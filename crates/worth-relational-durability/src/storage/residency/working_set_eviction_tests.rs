//! S1/M1 and S3: real eviction under an open root, and genuine saturation.
use super::working_set_fixture::*;
use std::num::NonZeroUsize;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use worth_relational_contracts::internal_port::identity::data::record_ids::EntityId;
use worth_relational_storage::internal_port::storage::residency::lease::Head;
use worth_relational_storage::internal_port::storage::residency::retention::RelationalRegionRetention;
use worth_relational_storage::internal_port::storage::residency::rolling_window::RollingResidencyWindow;
use worth_relational_storage::internal_port::storage::residency::working_set::{
    ResidencyPoolKind, WorkingSetAccountant,
};

/// One full rolling Head scan of `root`, checking every served record.
fn scan(root: &[RelationalRegionRetention], basis: &Basis, entity: EntityId) -> usize {
    let host = worth_harness::facade::execution_fixture::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host);
    let regions = root.iter().map(|region| (basis.cold.partition_id, region));
    let mut window =
        RollingResidencyWindow::<Head, _>::new(regions, NonZeroUsize::new(1).unwrap(), LIMIT);
    let mut visited = 0;
    while let Some(step) = window.advance(execution) {
        let (_, lease) = step.expect("a one-region window fits the working set");
        assert_eq!(
            lease.head_view().entity(entity).unwrap().generation(),
            entity.generation_value()
        );
        assert!(window.held() <= 1);
        visited += 1;
    }
    visited
}

#[test]
fn an_open_root_scanned_twice_with_concurrent_edits_stays_within_its_working_set() {
    let bases = bases();
    let basis = &bases.original;
    let (head, full) = image_bytes(basis);
    // One scan window (Head), one edit borrow (Head + Full) and one in-flight
    // promotion of a just-scanned cell to Full fit; the root never does.
    let ceiling = 3 * head + 2 * full;
    let accountant = WorkingSetAccountant::new(budget(ceiling, ceiling));
    let count = usize::try_from(4 * ceiling / head + 1).unwrap();
    // The original root stays open, with every cell retained, for both scans.
    let root = bound_root(basis, count, &accountant);
    assert!(
        count as u64 * head > 4 * ceiling,
        "the root is 4x the working set"
    );
    let reads_before = basis.source.reads.load(Ordering::SeqCst);
    assert_eq!(scan(&root, basis, bases.entity), count);
    let reads_after_first = basis.source.reads.load(Ordering::SeqCst);
    assert!(
        reads_after_first - reads_before >= count,
        "scan 1 loads every cell"
    );

    let stop = AtomicBool::new(false);
    let edits = AtomicUsize::new(0);
    std::thread::scope(|scope| {
        scope.spawn(|| {
            let host = worth_harness::facade::execution_fixture::serial_request();
            let execution = &worth_execution::ExecutionRequest::serial(&host);
            while !stop.load(Ordering::Acquire) {
                for region in root.iter().step_by(3) {
                    match region.borrow_full(execution, LIMIT) {
                        Ok(lease) => {
                            assert_basis(&lease, basis);
                            // An owned successor edition: unpersisted and never charged.
                            let successor =
                                RelationalRegionRetention::resident(lease.into_owned_partition());
                            assert!(successor.accepted_full_partition().is_some());
                            edits.fetch_add(1, Ordering::AcqRel);
                        }
                        Err(failure) if memory_exhausted(&failure) => std::thread::yield_now(),
                        Err(failure) => panic!("edit demand failed: {failure:?}"),
                    }
                }
            }
        });
        // Stop the editor even when scan 2 fails, so the scope always joins.
        struct StopOnDrop<'flag>(&'flag AtomicBool);
        impl Drop for StopOnDrop<'_> {
            fn drop(&mut self) {
                self.0.store(true, Ordering::Release);
            }
        }
        let _stop = StopOnDrop(&stop);
        assert_eq!(scan(&root, basis, bases.entity), count);
    });
    assert!(edits.load(Ordering::Acquire) > 0, "edits ran during scan 2");
    assert!(
        basis.source.reads.load(Ordering::SeqCst) - reads_after_first >= count,
        "scan 2 rewarmed evicted cells from cold storage"
    );
    let pressure = accountant.pressure();
    assert!(pressure.evictions_completed as usize >= count);
    assert!(pressure.head_resident_bytes <= ceiling);

    // With every borrow released, the whole allowance can be reclaimed: each
    // evicted image's charge lives in its allocation, so this succeeds only if
    // no strong `Arc<ResidentRegion>` survives in a root slot or raw cache.
    let everything = accountant
        .reserve(ResidencyPoolKind::Head, ceiling, std::ptr::null())
        .expect("no evicted image survives its last borrow");
    for region in &root {
        let observed = region.cell().working_set_observation();
        assert!(!observed.resident && !observed.raw_head_cached && !observed.raw_full_cached);
        assert_eq!((observed.leases, observed.state_bytes), (0, 0));
    }
    drop(everything);
    assert_eq!(accountant.charged_bytes(ResidencyPoolKind::Head), 0);
}

#[test]
fn borrows_pinning_the_whole_working_set_refuse_new_demand_until_released() {
    let bases = bases();
    let basis = &bases.original;
    let (head, full) = image_bytes(basis);
    let ceiling = 3 * head + head / 2;
    let accountant = WorkingSetAccountant::new(budget(ceiling, 2 * (head + full)));
    let root = bound_root(basis, 5, &accountant);
    let host = worth_harness::facade::execution_fixture::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host);
    let held = root[..3]
        .iter()
        .map(|region| region.borrow_head(execution, LIMIT).unwrap())
        .collect::<Vec<_>>();
    let reads = basis.source.reads.load(Ordering::SeqCst);

    let read = root[3].borrow_head(execution, LIMIT).unwrap_err();
    assert!(memory_exhausted(&read), "a new read refuses: {read:?}");
    let edit = root[0].borrow_full(execution, LIMIT).unwrap_err();
    assert!(
        memory_exhausted(&edit),
        "an edit's Full demand refuses: {edit:?}"
    );
    assert_eq!(
        basis.source.reads.load(Ordering::SeqCst),
        reads,
        "saturation refuses before any cold read; nothing was skipped or served"
    );
    let refused = root[3].cell().working_set_observation();
    assert!(!refused.resident && !refused.raw_head_cached && refused.leases == 0);
    assert!(
        root[3].cell().check_damage().is_ok(),
        "exhaustion is retryable"
    );
    assert!(accountant.charged_bytes(ResidencyPoolKind::Head) <= ceiling);

    // Resident borrowed images still serve; a hit needs no new memory.
    let hit = root[1].borrow_head(execution, LIMIT).unwrap();
    assert!(hit.head_view().entity(bases.entity).is_some());
    // Historical reads draw on their own allowance and cannot be starved.
    let historical = bound_cell(basis, &accountant, ResidencyPoolKind::Historical);
    assert_basis(
        &historical.demand_full(execution, LIMIT, None).unwrap(),
        basis,
    );

    drop((held, hit));
    let recovered = root[3].borrow_full(execution, LIMIT).unwrap();
    assert_basis(&recovered, basis);
    assert!(basis.source.reads.load(Ordering::SeqCst) > reads);
    // P2-1: promoting Head -> Full clears the superseded raw_head cache immediately.
    let obs = root[3].cell().working_set_observation();
    assert!(obs.resident && !obs.raw_head_cached && obs.raw_full_cached);
    assert_eq!(obs.raw_cache_bytes, 0);
}

#[test]
fn oversized_request_and_in_flight_saturation_refuse_without_wiping_cache() {
    let bases = bases();
    let basis = &bases.original;
    let (head, full) = image_bytes(basis);
    let ceiling = 2 * head;
    let accountant = WorkingSetAccountant::new(budget(ceiling, ceiling));
    let root = bound_root(basis, 2, &accountant);
    let host = worth_harness::facade::execution_fixture::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host);

    // Warm cell 0 and drop its borrow so it is evictable.
    let warm = root[0].borrow_head(execution, LIMIT).unwrap();
    drop(warm);
    let evictions_before = accountant.pressure().evictions_completed;
    assert!(root[0].cell().working_set_observation().resident);

    // P1-4a: a single request larger than ceiling refuses immediately without sweeping.
    let err = accountant
        .reserve(ResidencyPoolKind::Head, ceiling + 1, std::ptr::null())
        .unwrap_err();
    assert_eq!(err.requested, ceiling + 1);
    assert_eq!(accountant.pressure().evictions_completed, evictions_before);
    assert!(root[0].cell().working_set_observation().resident);

    // Saturating max_in_flight_fill_bytes refuses cold loads before reading.
    let fill_cap = accountant.budget().max_in_flight_fill_bytes;
    let _fill_hold = accountant.reserve_fill(fill_cap).unwrap();
    let reads_before = basis.source.reads.load(Ordering::SeqCst);
    let refused = root[1].borrow_head(execution, LIMIT).unwrap_err();
    assert!(memory_exhausted(&refused));
    assert_eq!(basis.source.reads.load(Ordering::SeqCst), reads_before);
    assert!(full >= head);
}

#[test]
fn rolling_window_with_k_gt_one_sheds_cleanly_and_small_overhead_does_not_poison_estimates() {
    use worth_relational_storage::internal_port::storage::residency::working_set::{
        ResidencyBudget, ResidencyLevelKind,
    };

    let bases = bases();
    let basis = &bases.original;
    let (head, _) = image_bytes(basis);
    // Only 1 Head image fits, yet K = 3: window must self-shed and complete the full scan.
    let ceiling = head + head / 2;
    let accountant = WorkingSetAccountant::new(budget(ceiling, ceiling));
    let root = bound_root(basis, 4, &accountant);
    let host = worth_harness::facade::execution_fixture::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host);
    let regions = root.iter().map(|r| (basis.cold.partition_id, r));
    let mut window =
        RollingResidencyWindow::<Head, _>::new(regions, NonZeroUsize::new(3).unwrap(), LIMIT);
    let mut visited = 0;
    while let Some(step) = window.advance(execution) {
        let (_, lease) = step.expect("rolling window sheds older borrows to fit next cell");
        assert!(lease.head_view().entity(bases.entity).is_some());
        visited += 1;
    }
    assert_eq!(visited, 4);

    // P1-4b: after measuring real small partitions, a 1 MiB encoded estimate must stay
    // near linear (fixed overhead + ~1 MiB), not 64x inflated.
    let est_1mib = accountant.estimate(ResidencyLevelKind::Head, 1024 * 1024);
    assert!(
        est_1mib < 4 * 1024 * 1024,
        "1 MiB estimate must not be poisoned by small-partition fixed overhead: {est_1mib}"
    );
    let tiny = WorkingSetAccountant::new(ResidencyBudget {
        head_working_set_bytes: head / 2,
        historical_working_set_bytes: head / 2,
        max_in_flight_fill_bytes: 1024 * 1024,
        streaming_window_bytes: 64 * 1024,
    });
    let too_big = bound_root(basis, 1, &tiny);
    let regions = too_big.iter().map(|r| (basis.cold.partition_id, r));
    let mut small_window =
        RollingResidencyWindow::<Head, _>::new(regions, NonZeroUsize::new(2).unwrap(), LIMIT);
    let err = small_window.advance(execution).unwrap().unwrap_err();
    assert!(memory_exhausted(&err));
}
