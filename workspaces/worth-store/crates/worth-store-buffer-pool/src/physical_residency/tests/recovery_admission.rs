//! The original Recovery ceiling governs every same-pool reservation.

use std::sync::Barrier;

use super::*;

const RECOVERY: PhysicalOperationAllocationScope = PhysicalOperationAllocationScope::Recovery;

#[test]
fn overlapping_recovery_admission_and_growth_preserve_denied_backing() {
    let pool = pool(151);
    let baseline = pool.counters().admitted_bytes();
    let mut existing = pool.begin_operation(RECOVERY, nonzero_bytes(20)).unwrap();
    pool.restrict_recovery_operation_bytes(60).unwrap();
    let retained = pool.begin_operation(RECOVERY, nonzero_bytes(40)).unwrap();
    let before = pool.allocation_events().snapshot();
    assert_eq!(
        pool.begin_operation(RECOVERY, NonZeroU64::MIN).unwrap_err(),
        pressure(&pool, 1, 60, 60)
    );
    assert_no_reservation_change(before, pool.allocation_events().snapshot());
    let before = pool.allocation_events().snapshot();
    assert_eq!(existing.try_resize(21), Err(pressure(&pool, 1, 60, 60)));
    assert_eq!(existing.bytes(), 20);
    assert_eq!(retained.bytes(), 40);
    assert_no_reservation_change(before, pool.allocation_events().snapshot());
    assert_eq!(pool.counters().active_operation_bytes_for(RECOVERY), 60);
    drop(retained);
    existing.try_resize(21).unwrap();
    assert_eq!(pool.counters().active_operation_bytes_for(RECOVERY), 21);
    drop(existing);
    assert_eq!(pool.counters().admitted_bytes(), baseline);
    assert!(!pool.close().requires_inspection());
    let observer = pool.allocation_events();
    drop(pool);
    assert_eq!(
        observer
            .snapshot()
            .for_dimension(PhysicalResidencyDimension::TotalBytes)
            .active_units(),
        0
    );
}

#[test]
fn concurrent_recovery_admissions_have_one_winner_without_ceiling_overspend() {
    let pool = pool(152);
    pool.restrict_recovery_operation_bytes(60).unwrap();
    let before = pool.allocation_events().snapshot();
    let start = Barrier::new(3);
    let attempted = Barrier::new(3);
    let observed = Barrier::new(3);
    let (results, active, after) = std::thread::scope(|scope| {
        let mut tasks = Vec::new();
        for facade in [pool.clone(), pool.clone()] {
            let (start, attempted, observed) = (&start, &attempted, &observed);
            tasks.push(scope.spawn(move || {
                start.wait();
                let result = facade.begin_operation(RECOVERY, nonzero_bytes(40));
                attempted.wait();
                observed.wait();
                result
            }));
        }
        start.wait();
        attempted.wait();
        let active = pool.counters().active_operation_bytes_for(RECOVERY);
        let after = pool.allocation_events().snapshot();
        observed.wait();
        let results = tasks
            .into_iter()
            .map(|task| task.join().unwrap())
            .collect::<Vec<_>>();
        (results, active, after)
    });
    assert_eq!(active, 40);
    assert_one_admission(before, after, 40);
    assert_eq!(results.iter().filter(|result| result.is_ok()).count(), 1);
    let mut winner = None;
    for result in results {
        match result {
            Ok(grant) => winner = Some(grant),
            Err(denial) => assert_eq!(denial, pressure(&pool, 40, 40, 60)),
        }
    }
    assert_eq!(winner.as_ref().unwrap().bytes(), 40);
    drop(winner);
    let retry = pool.begin_operation(RECOVERY, nonzero_bytes(60)).unwrap();
    assert_eq!(pool.counters().active_operation_bytes_for(RECOVERY), 60);
    drop(retry);
    assert_eq!(pool.counters().active_operation_bytes(), 0);
    assert!(!pool.close().requires_inspection());
}

#[test]
fn concurrent_recovery_growth_keeps_loser_unchanged_and_retryable() {
    let pool = pool(153);
    pool.restrict_recovery_operation_bytes(60).unwrap();
    let grants = [
        pool.begin_operation(RECOVERY, nonzero_bytes(10)).unwrap(),
        pool.begin_operation(RECOVERY, nonzero_bytes(10)).unwrap(),
    ];
    let before = pool.allocation_events().snapshot();
    let start = Barrier::new(3);
    let attempted = Barrier::new(3);
    let observed = Barrier::new(3);
    let (results, active, after) = std::thread::scope(|scope| {
        let mut tasks = Vec::new();
        for (mut grant, facade) in grants.into_iter().zip([pool.clone(), pool.clone()]) {
            let (start, attempted, observed) = (&start, &attempted, &observed);
            tasks.push(scope.spawn(move || {
                start.wait();
                let result = grant.try_resize(50);
                attempted.wait();
                observed.wait();
                (grant, result, facade)
            }));
        }
        start.wait();
        attempted.wait();
        let active = pool.counters().active_operation_bytes_for(RECOVERY);
        let after = pool.allocation_events().snapshot();
        observed.wait();
        let results = tasks
            .into_iter()
            .map(|task| task.join().unwrap())
            .collect::<Vec<_>>();
        (results, active, after)
    });
    assert_eq!(active, 60);
    assert_one_admission(before, after, 40);
    assert_eq!(
        results
            .iter()
            .filter(|(_, result, _)| result.is_ok())
            .count(),
        1
    );
    let mut winner = None;
    let mut loser = None;
    for (grant, result, facade) in results {
        assert_eq!(grant.observation().pool(), facade.incarnation());
        match result {
            Ok(()) => {
                assert_eq!(grant.bytes(), 50);
                winner = Some(grant);
            }
            Err(denial) => {
                assert_eq!(denial, pressure(&pool, 40, 60, 60));
                assert_eq!(grant.bytes(), 10);
                loser = Some(grant);
            }
        }
    }
    drop(winner);
    assert_eq!(pool.counters().active_operation_bytes_for(RECOVERY), 10);
    let mut loser = loser.unwrap();
    loser.try_resize(50).unwrap();
    assert_eq!(pool.counters().active_operation_bytes_for(RECOVERY), 50);
    drop(loser);
    assert_eq!(pool.counters().active_operation_bytes(), 0);
    assert!(!pool.close().requires_inspection());
}

#[test]
fn restriction_cannot_raise_or_tighten_below_live_recovery() {
    let pool = pool(154);
    pool.restrict_recovery_operation_bytes(60).unwrap();
    let held = pool.begin_operation(RECOVERY, nonzero_bytes(40)).unwrap();
    let before = pool.allocation_events().snapshot();
    assert_eq!(
        pool.restrict_recovery_operation_bytes(39),
        Err(pressure(&pool, 0, 40, 39))
    );
    assert_no_reservation_change(before, pool.allocation_events().snapshot());
    let still_admitted = pool.begin_operation(RECOVERY, nonzero_bytes(20)).unwrap();
    drop(still_admitted);
    pool.restrict_recovery_operation_bytes(50).unwrap();
    pool.restrict_recovery_operation_bytes(80).unwrap();
    assert_eq!(
        pool.begin_operation(RECOVERY, nonzero_bytes(11))
            .unwrap_err(),
        pressure(&pool, 11, 40, 50)
    );
    let exact = pool.begin_operation(RECOVERY, nonzero_bytes(10)).unwrap();
    drop(exact);
    drop(held);
    assert_eq!(pool.counters().active_operation_bytes(), 0);
    assert!(!pool.close().requires_inspection());
}

#[test]
fn zero_recovery_ceiling_preserves_other_scope_limits_and_closed_posture() {
    let pool = pool(155);
    pool.restrict_recovery_operation_bytes(0).unwrap();
    pool.restrict_recovery_operation_bytes(100).unwrap();
    let before = pool.allocation_events().snapshot();
    assert_eq!(
        pool.begin_operation(RECOVERY, NonZeroU64::MIN).unwrap_err(),
        pressure(&pool, 1, 0, 0)
    );
    assert_no_reservation_change(before, pool.allocation_events().snapshot());
    let maintenance = pool
        .begin_operation(
            PhysicalOperationAllocationScope::Maintenance,
            nonzero_bytes(100),
        )
        .unwrap();
    let denial = pool
        .begin_operation(
            PhysicalOperationAllocationScope::Maintenance,
            NonZeroU64::MIN,
        )
        .unwrap_err();
    let PhysicalResidencyDenial::Pressure(denial) = denial else {
        panic!("Maintenance retains native policy limits")
    };
    assert_eq!(
        denial.dimension(),
        PhysicalResidencyDimension::OperationScope(PhysicalOperationAllocationScope::Maintenance)
    );
    assert_eq!(
        (denial.requested(), denial.current(), denial.limit()),
        (1, 100, 100)
    );
    drop(maintenance);
    assert_eq!(pool.counters().active_operation_bytes(), 0);
    assert!(!pool.close().requires_inspection());
    let before = pool.allocation_events().snapshot();
    assert_eq!(
        pool.restrict_recovery_operation_bytes(0),
        Err(PhysicalResidencyDenial::PoolClosed)
    );
    assert_eq!(
        pool.begin_operation(RECOVERY, NonZeroU64::MIN).unwrap_err(),
        PhysicalResidencyDenial::PoolClosed
    );
    assert_no_reservation_change(before, pool.allocation_events().snapshot());
}

fn pool(identity: u8) -> PhysicalResidencyPool {
    PhysicalResidencyPool::open(store(identity), limits(1024, 2, 1, 100, 4)).unwrap()
}

fn pressure(
    pool: &PhysicalResidencyPool,
    requested: u64,
    current: u64,
    limit: u64,
) -> PhysicalResidencyDenial {
    PhysicalResidencyDenial::Pressure(PhysicalResidencyPressureDenial::new(
        pool.store_identity(),
        pool.incarnation(),
        PhysicalResidencyPressureDemand {
            dimension: PhysicalResidencyDimension::OperationScope(RECOVERY),
            scope: RECOVERY,
            requested,
            current,
            limit,
        },
    ))
}

fn assert_no_reservation_change(
    before: PhysicalResidencyAllocationEventSnapshot,
    after: PhysicalResidencyAllocationEventSnapshot,
) {
    for dimension in [
        PhysicalResidencyDimension::OperationScope(RECOVERY),
        PhysicalResidencyDimension::OperationBytes,
        PhysicalResidencyDimension::TotalBytes,
    ] {
        let before = before.for_dimension(dimension);
        let after = after.for_dimension(dimension);
        assert_eq!(after.admissions(), before.admissions());
        assert_eq!(after.admitted_units(), before.admitted_units());
        assert_eq!(after.releases(), before.releases());
        assert_eq!(after.released_units(), before.released_units());
    }
}

fn assert_one_admission(
    before: PhysicalResidencyAllocationEventSnapshot,
    after: PhysicalResidencyAllocationEventSnapshot,
    delta: u64,
) {
    for dimension in [
        PhysicalResidencyDimension::OperationScope(RECOVERY),
        PhysicalResidencyDimension::OperationBytes,
        PhysicalResidencyDimension::TotalBytes,
    ] {
        let before = before.for_dimension(dimension);
        let after = after.for_dimension(dimension);
        assert_eq!(after.admissions(), before.admissions() + 1);
        assert_eq!(after.admitted_units(), before.admitted_units() + delta);
        assert_eq!(after.releases(), before.releases());
        assert_eq!(after.released_units(), before.released_units());
    }
}
