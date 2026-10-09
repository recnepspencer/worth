//! Independent scope and total capture windows, with real live competitors.

use super::*;

#[test]
fn maintenance_scope_window_uses_protected_headroom_and_preserves_native_pressure() {
    use PhysicalOperationAllocationScope::{Maintenance, Recovery};
    let identity = store(136);
    let basis = capture_bytes(1);
    let scope_limit = basis * 4;
    let pool = PhysicalResidencyPool::open(
        identity,
        window_limits(16_384, 4096, scope_limit, scope_limit, basis),
    )
    .unwrap();
    let recovery = pool
        .begin_operation(Recovery, nonzero_bytes(scope_limit - basis))
        .unwrap();
    assert_eq!(
        pool.begin_operation(Recovery, nonzero_bytes(basis))
            .unwrap_err(),
        PhysicalResidencyDenial::Pressure(PhysicalResidencyPressureDenial::new(
            identity,
            pool.incarnation(),
            PhysicalResidencyPressureDemand {
                dimension: PhysicalResidencyDimension::OperationScope(Recovery),
                scope: Recovery,
                requested: basis,
                current: scope_limit - basis,
                limit: scope_limit - basis,
            },
        ))
    );
    let mut competing = pool
        .begin_operation(Maintenance, nonzero_bytes(scope_limit - basis))
        .unwrap();
    let session = pool.begin_dirty_generation_capture().unwrap();
    let allocation = pool
        .begin_dirty_generation_capture_allocation(&session, nonzero_bytes(4096))
        .unwrap();
    assert_eq!(allocation.bytes(), basis);
    assert_eq!(
        pool.counters().active_operation_bytes_for(Maintenance),
        scope_limit
    );
    assert!(pool.counters().active_operation_bytes() < 4096 - basis);
    assert!(pool.counters().admitted_bytes() < 16_384 - basis);
    drop(allocation);
    competing.try_resize(scope_limit - basis + 1).unwrap();
    let before = pool.allocation_events().snapshot();
    assert_eq!(
        pool.begin_dirty_generation_capture_allocation(&session, nonzero_bytes(4096))
            .unwrap_err(),
        PhysicalResidencyDenial::Pressure(PhysicalResidencyPressureDenial::new(
            identity,
            pool.incarnation(),
            PhysicalResidencyPressureDemand {
                dimension: PhysicalResidencyDimension::OperationScope(Maintenance),
                scope: Maintenance,
                requested: basis,
                current: scope_limit - basis + 1,
                limit: scope_limit,
            },
        ))
    );
    assert_eq!(
        pool.counters().active_operation_bytes_for(Maintenance),
        competing.bytes()
    );
    assert_eq!(
        pool.allocation_events()
            .snapshot()
            .for_dimension(PhysicalResidencyDimension::OperationBytes)
            .admitted_units(),
        before
            .for_dimension(PhysicalResidencyDimension::OperationBytes)
            .admitted_units()
    );
    drop(competing);
    drop(recovery);
    assert_eq!(pool.counters().active_operation_bytes(), 0);
    let observer = pool.allocation_events();
    assert!(!pool.close().requires_inspection());
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
fn total_window_counts_metadata_resident_and_recovery_before_exact_pressure() {
    use PhysicalOperationAllocationScope::{Maintenance, Recovery};
    let identity = store(137);
    let total_limit = 16_384;
    let basis = capture_bytes(1);
    let pool = PhysicalResidencyPool::open(
        identity,
        window_limits(total_limit, total_limit, total_limit, total_limit, 0),
    )
    .unwrap();
    let write = candidate_batches_allocation(&pool, &[1]);
    let key = PhysicalFrameKey::new(identity, coordinate(1, 16));
    let dirty = materialize(&pool, &write, key, 0x53);
    drop(write);
    let baseline = pool.counters();
    assert!(baseline.metadata_bytes() > 0);
    assert_eq!(baseline.resident_bytes(), 16);
    assert_eq!(baseline.active_operation_bytes(), 0);
    assert_eq!(baseline.admitted_bytes(), baseline.metadata_bytes() + 16);
    let mut recovery = pool
        .begin_operation(
            Recovery,
            nonzero_bytes(total_limit - baseline.metadata_bytes() - 16 - basis),
        )
        .unwrap();
    let session = pool.begin_dirty_generation_capture().unwrap();
    let allocation = pool
        .begin_dirty_generation_capture_allocation(&session, nonzero_bytes(total_limit))
        .unwrap();
    assert_eq!(allocation.bytes(), basis);
    assert_eq!(pool.counters().admitted_bytes(), total_limit);
    assert!(pool.counters().active_operation_bytes() < total_limit - basis);
    let step = pool
        .capture_next_dirty_generation_slice(session, allocation)
        .unwrap();
    let slice = match step {
        PhysicalDirtyGenerationCaptureStep::More { slice, .. }
        | PhysicalDirtyGenerationCaptureStep::Complete { slice, .. } => slice,
    };
    assert_eq!(slice.frames().len(), 1);
    assert_eq!(slice.frames()[0].frame(), key);
    assert_eq!(slice.metadata_bytes(), basis);
    drop(slice);
    recovery.try_resize(recovery.bytes() + 1).unwrap();
    let current = total_limit - basis + 1;
    assert_eq!(pool.counters().admitted_bytes(), current);
    let session = pool.begin_dirty_generation_capture().unwrap();
    let before = pool.allocation_events().snapshot();
    assert_eq!(
        pool.begin_dirty_generation_capture_allocation(&session, nonzero_bytes(total_limit))
            .unwrap_err(),
        PhysicalResidencyDenial::Pressure(PhysicalResidencyPressureDenial::new(
            identity,
            pool.incarnation(),
            PhysicalResidencyPressureDemand {
                dimension: PhysicalResidencyDimension::TotalBytes,
                scope: Maintenance,
                requested: basis,
                current,
                limit: total_limit,
            },
        ))
    );
    assert_eq!(pool.counters().admitted_bytes(), current);
    assert_eq!(
        pool.allocation_events()
            .snapshot()
            .for_dimension(PhysicalResidencyDimension::TotalBytes)
            .admitted_units(),
        before
            .for_dimension(PhysicalResidencyDimension::TotalBytes)
            .admitted_units()
    );
    drop(recovery);
    assert_eq!(pool.counters().active_operation_bytes(), 0);
    dirty.discard_candidate().unwrap();
    let observer = pool.allocation_events();
    assert!(!pool.close().requires_inspection());
    drop(pool);
    assert_eq!(
        observer
            .snapshot()
            .for_dimension(PhysicalResidencyDimension::TotalBytes)
            .active_units(),
        0
    );
}

fn window_limits(
    total: u64,
    operation: u64,
    maintenance: u64,
    recovery: u64,
    headroom: u64,
) -> PhysicalResidencyLimits {
    use PhysicalOperationAllocationScope as Scope;
    use PhysicalSpeculativeWorkKind as Kind;
    let mut declaration = PhysicalResidencyLimits::builder()
        .total_bytes(nonzero_bytes(total))
        .resident_bytes(nonzero_bytes(128))
        .metadata_bytes(nonzero_bytes(TEST_FRAME_METADATA_BYTES))
        .frame_entries(nonzero_count(4))
        .pinned_frames(nonzero_count(2))
        .pin_leases(nonzero_count(2))
        .dirty_frames(nonzero_count(2))
        .dirty_replacement_bytes(nonzero_bytes(128))
        .operation_bytes(nonzero_bytes(operation))
        .progress_headroom_bytes(headroom);
    for scope in [
        Scope::ForegroundRead,
        Scope::ForegroundWrite,
        Scope::Recovery,
        Scope::Scrub,
        Scope::Maintenance,
        Scope::Verification,
        Scope::Blob,
    ] {
        let bytes = match scope {
            Scope::Maintenance => maintenance,
            Scope::Recovery => recovery,
            _ => operation,
        };
        declaration = declaration.scope_bytes(scope, nonzero_bytes(bytes));
    }
    for kind in [Kind::ReadAhead, Kind::Prefetch, Kind::WriteBehind] {
        declaration = declaration.speculative_frames(kind, nonzero_count(2));
    }
    declaration.admit(NonZeroU64::MIN).unwrap()
}
