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
}
