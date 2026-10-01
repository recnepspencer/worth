use super::*;

#[test]
fn one_pool_issued_grant_resizes_exact_scope_and_global_bytes() {
    let identity = store(41);
    let scope = PhysicalOperationAllocationScope::Recovery;
    let pool = PhysicalResidencyPool::open(identity, limits(1024, 2, 1, 100, 4)).unwrap();
    let baseline = pool.counters().admitted_bytes();
    let mut grant = pool.begin_operation(scope, nonzero_bytes(60)).unwrap();
    assert_eq!(grant.try_resize(60), Ok(()));
    assert_eq!(grant.try_resize(25), Ok(()));
    assert_eq!(grant.bytes(), 25);
    assert_eq!(pool.counters().active_operation_bytes_for(scope), 25);
    assert_eq!(pool.counters().active_operation_bytes(), 25);
    assert_eq!(pool.counters().admitted_bytes(), baseline + 25);

    assert_eq!(grant.try_resize(80), Ok(()));
    assert_eq!(grant.bytes(), 80);
    assert_eq!(grant.observation().store(), identity);
    assert_eq!(grant.observation().pool(), pool.incarnation());
    assert_eq!(pool.counters().active_operation_bytes_for(scope), 80);
    assert_eq!(pool.counters().active_operation_bytes(), 80);
    assert_eq!(pool.counters().admitted_bytes(), baseline + 80);

    let foreign = PhysicalResidencyPool::open(identity, limits(1024, 2, 1, 100, 4)).unwrap();
    let key = PhysicalFrameKey::new(identity, coordinate(1, 20));
    assert_eq!(
        foreign.access_frame(&grant, key).unwrap_err(),
        PhysicalResidencyDenial::AllocationGrantMismatch
    );
    assert_eq!(foreign.counters().active_operation_bytes(), 0);

    grant.try_resize(0).unwrap();
    assert_eq!(pool.counters().active_operation_bytes(), 0);
    grant.try_resize(25).unwrap();
    assert_eq!(pool.counters().active_operation_bytes(), 25);
    drop(grant);
    assert_eq!(pool.counters().admitted_bytes(), baseline);
    assert!(!pool.close().requires_inspection());
}

#[test]
fn failed_growth_preserves_reservation_and_below_use_shrink_is_typed() {
    let identity = store(42);
    let scope = PhysicalOperationAllocationScope::Recovery;
    let pool = PhysicalResidencyPool::open(identity, limits(1024, 2, 1, 100, 4)).unwrap();
    let mut grant = pool.begin_operation(scope, nonzero_bytes(60)).unwrap();
    let other = pool
        .begin_operation(
            PhysicalOperationAllocationScope::ForegroundRead,
            nonzero_bytes(30),
        )
        .unwrap();
    let before = pool.counters();
    assert_eq!(
        grant.try_resize(75),
        Err(PhysicalResidencyDenial::Pressure(
            PhysicalResidencyPressureDenial::new(
                identity,
                pool.incarnation(),
                PhysicalResidencyPressureDemand {
                    dimension: PhysicalResidencyDimension::OperationBytes,
                    scope,
                    requested: 15,
                    current: 90,
                    limit: 100,
                },
            ),
        ))
    );
    assert_eq!(grant.bytes(), 60);
    assert_eq!(pool.counters().active_operation_bytes_for(scope), 60);
    assert_eq!(pool.counters().active_operation_bytes(), 90);
    assert_eq!(pool.counters().admitted_bytes(), before.admitted_bytes());
    assert_eq!(pool.counters().denials(), before.denials() + 1);

    // Safe named uses borrow the grant; this exercises the defensive atomic
    // check against an inconsistent holder without minting a new grant.
    grant
        .active_use_bytes
        .store(41, std::sync::atomic::Ordering::Release);
    assert_eq!(
        grant.try_resize(40),
        Err(
            PhysicalResidencyDenial::AllocationGrantResizeBelowActiveUse {
                requested: 40,
                active: 41,
            }
        )
    );
    assert_eq!(grant.bytes(), 60);
    grant
        .active_use_bytes
        .store(0, std::sync::atomic::Ordering::Release);
    drop(other);
    drop(grant);
    assert_eq!(pool.counters().active_operation_bytes(), 0);
}

#[test]
fn closed_pool_allows_cleanup_shrink_but_never_new_growth() {
    let identity = store(43);
    let scope = PhysicalOperationAllocationScope::Recovery;
    let pool = PhysicalResidencyPool::open(identity, limits(1024, 2, 1, 100, 4)).unwrap();
    let mut grant = pool.begin_operation(scope, nonzero_bytes(40)).unwrap();
    assert!(pool.close().requires_inspection());
    assert_eq!(grant.try_resize(40), Ok(()));
    assert_eq!(grant.try_resize(0), Ok(()));
    assert_eq!(pool.counters().active_operation_bytes_for(scope), 0);
    assert_eq!(grant.try_resize(0), Ok(()));
    assert_eq!(
        grant.try_resize(1),
        Err(PhysicalResidencyDenial::PoolClosed)
    );
    assert_eq!(grant.bytes(), 0);
    drop(grant);
    assert!(!pool.close().requires_inspection());
}

fn maximum_byte_limits() -> PhysicalResidencyLimits {
    use PhysicalOperationAllocationScope as Scope;
    use PhysicalSpeculativeWorkKind as Speculation;

    PhysicalResidencyLimits::builder()
        .total_bytes(nonzero_bytes(u64::MAX))
        .resident_bytes(nonzero_bytes(1024))
        .metadata_bytes(nonzero_bytes(TEST_FRAME_METADATA_BYTES))
        .frame_entries(nonzero_count(4))
        .pinned_frames(nonzero_count(2))
        .pin_leases(nonzero_count(2))
        .dirty_frames(nonzero_count(1))
        .dirty_replacement_bytes(nonzero_bytes(1024))
        .operation_bytes(nonzero_bytes(u64::MAX))
        .scope_bytes(Scope::ForegroundRead, nonzero_bytes(u64::MAX))
        .scope_bytes(Scope::ForegroundWrite, nonzero_bytes(u64::MAX))
        .scope_bytes(Scope::Recovery, nonzero_bytes(u64::MAX))
        .scope_bytes(Scope::Scrub, nonzero_bytes(u64::MAX))
        .scope_bytes(Scope::Maintenance, nonzero_bytes(u64::MAX))
        .scope_bytes(Scope::Verification, nonzero_bytes(u64::MAX))
        .scope_bytes(Scope::Blob, nonzero_bytes(u64::MAX))
        .speculative_frames(Speculation::Prefetch, nonzero_count(2))
        .speculative_frames(Speculation::ReadAhead, nonzero_count(2))
        .speculative_frames(Speculation::WriteBehind, nonzero_count(1))
        .admit(NonZeroU64::MIN)
        .unwrap()
}

#[test]
fn resize_denies_operation_counter_overflow_without_changing_reservations() {
    let identity = store(44);
    // Maintenance can use the full admitted maximum, including progress headroom.
    let scope = PhysicalOperationAllocationScope::Maintenance;
    let pool = PhysicalResidencyPool::open(identity, maximum_byte_limits()).unwrap();
    let mut grant = pool.begin_operation(scope, nonzero_bytes(2)).unwrap();
    let other = pool
        .begin_operation(
            PhysicalOperationAllocationScope::ForegroundRead,
            nonzero_bytes(2),
        )
        .unwrap();
    let before = pool.counters();
    assert_eq!(
        grant.try_resize(u64::MAX),
        Err(PhysicalResidencyDenial::Pressure(
            PhysicalResidencyPressureDenial::new(
                identity,
                pool.incarnation(),
                PhysicalResidencyPressureDemand {
                    dimension: PhysicalResidencyDimension::OperationBytes,
                    scope,
                    requested: u64::MAX - 2,
                    current: 4,
                    limit: u64::MAX,
                },
            ),
        ))
    );
    assert_eq!(grant.bytes(), 2);
    assert_eq!(pool.counters().active_operation_bytes_for(scope), 2);
    assert_eq!(pool.counters().active_operation_bytes(), 4);
    assert_eq!(pool.counters().admitted_bytes(), before.admitted_bytes());
    drop(other);
    drop(grant);
    assert_eq!(pool.counters().active_operation_bytes(), 0);
    assert!(!pool.close().requires_inspection());
}

#[test]
fn resize_denies_total_counter_overflow_after_scope_and_operation_fit() {
    let identity = store(45);
    let scope = PhysicalOperationAllocationScope::Maintenance;
    let pool = PhysicalResidencyPool::open(identity, maximum_byte_limits()).unwrap();
    let mut grant = pool.begin_operation(scope, nonzero_bytes(2)).unwrap();
    let before = pool.counters();
    assert_eq!(
        grant.try_resize(u64::MAX),
        Err(PhysicalResidencyDenial::Pressure(
            PhysicalResidencyPressureDenial::new(
                identity,
                pool.incarnation(),
                PhysicalResidencyPressureDemand {
                    dimension: PhysicalResidencyDimension::TotalBytes,
                    scope,
                    requested: u64::MAX - 2,
                    current: before.admitted_bytes(),
                    limit: u64::MAX,
                },
            ),
        ))
    );
    assert_eq!(grant.bytes(), 2);
    assert_eq!(pool.counters().active_operation_bytes_for(scope), 2);
    assert_eq!(pool.counters().active_operation_bytes(), 2);
    assert_eq!(pool.counters().admitted_bytes(), before.admitted_bytes());
    drop(grant);
    assert_eq!(pool.counters().active_operation_bytes(), 0);
    assert!(!pool.close().requires_inspection());
}
