//! Capture windows reserve against genuine co-live native operation grants.

use super::*;

#[path = "allocation_window/quota_pressure.rs"]
mod quota_pressure;

#[test]
fn one_basis_aggregate_headroom_with_retained_recovery_captures() {
    let pool = PhysicalResidencyPool::open(store(131), limits(128, 1, 1, 2048, 1)).unwrap();
    let write = candidate_batches_allocation(&pool, &[1]);
    let key = PhysicalFrameKey::new(store(131), coordinate(1, 16));
    let dirty = materialize(&pool, &write, key, 0x51);
    drop(write);
    let basis = capture_bytes(1);
    let recovery = pool
        .begin_operation(
            PhysicalOperationAllocationScope::Recovery,
            nonzero_bytes(2048 - basis),
        )
        .unwrap();
    let session = pool.begin_dirty_generation_capture().unwrap();
    let allocation = pool
        .begin_dirty_generation_capture_allocation(&session, nonzero_bytes(2048))
        .unwrap();
    assert_eq!(allocation.bytes(), basis);
    assert_eq!(pool.counters().active_operation_bytes(), 2048);
    let PhysicalDirtyGenerationCaptureStep::Complete { slice, .. } = pool
        .capture_next_dirty_generation_slice(session, allocation)
        .unwrap()
    else {
        panic!("the genuine one-slot dirty session must complete");
    };
    assert_eq!(slice.frames().len(), 1);
    assert_eq!(slice.frames()[0].frame(), key);
    assert_eq!(slice.metadata_bytes(), basis);
    assert_eq!(slice.admitted_bytes(), basis);
    drop(slice);
    assert_eq!(pool.counters().active_operation_bytes(), recovery.bytes());
    drop(recovery);
    assert_eq!(pool.counters().active_operation_bytes(), 0);
    dirty.discard_candidate().unwrap();
    assert!(!pool.close().requires_inspection());
}

#[test]
fn less_than_one_basis_headroom_reports_native_operation_pressure_without_admission() {
    let identity = store(132);
    let pool = PhysicalResidencyPool::open(identity, limits(128, 1, 1, 2048, 1)).unwrap();
    let basis = capture_bytes(1);
    let current = 2048 - basis + 1;
    let recovery = pool
        .begin_operation(
            PhysicalOperationAllocationScope::Recovery,
            nonzero_bytes(current),
        )
        .unwrap();
    let session = pool.begin_dirty_generation_capture().unwrap();
    let before = pool.allocation_events().snapshot();
    assert_eq!(
        pool.begin_dirty_generation_capture_allocation(&session, nonzero_bytes(2048))
            .unwrap_err(),
        PhysicalResidencyDenial::Pressure(PhysicalResidencyPressureDenial::new(
            identity,
            pool.incarnation(),
            PhysicalResidencyPressureDemand {
                dimension: PhysicalResidencyDimension::OperationBytes,
                scope: PhysicalOperationAllocationScope::Maintenance,
                requested: basis,
                current,
                limit: 2048,
            },
        ))
    );
    assert_eq!(pool.counters().active_operation_bytes(), current);
    for dimension in [
        PhysicalResidencyDimension::OperationBytes,
        PhysicalResidencyDimension::TotalBytes,
    ] {
        assert_eq!(
            pool.allocation_events()
                .snapshot()
                .for_dimension(dimension)
                .admitted_units(),
            before.for_dimension(dimension).admitted_units()
        );
    }
    drop(recovery);
    let allocation = pool
        .begin_dirty_generation_capture_allocation(&session, nonzero_bytes(2048))
        .unwrap();
    drop(allocation);
    assert!(!pool.close().requires_inspection());
}

#[test]
fn foreign_incarnation_session_denies_before_native_window_reservation() {
    let identity = store(133);
    let pool = PhysicalResidencyPool::open(identity, limits(128, 1, 1, 2048, 1)).unwrap();
    let foreign = PhysicalResidencyPool::open(identity, limits(128, 1, 1, 2048, 1)).unwrap();
    let session = foreign.begin_dirty_generation_capture().unwrap();
    let before = pool.allocation_events().snapshot();
    assert_eq!(
        pool.begin_dirty_generation_capture_allocation(&session, nonzero_bytes(2048))
            .unwrap_err(),
        PhysicalResidencyDenial::DirtyGenerationCaptureSessionMismatch
    );
    assert_eq!(pool.counters().active_operation_bytes(), 0);
    assert_eq!(
        pool.allocation_events()
            .snapshot()
            .for_dimension(PhysicalResidencyDimension::OperationBytes),
        before.for_dimension(PhysicalResidencyDimension::OperationBytes)
    );
    assert!(!pool.close().requires_inspection());
    assert!(!foreign.close().requires_inspection());
}

#[test]
fn useful_session_slots_cap_the_policy_maximum_before_allocation() {
    let pool = PhysicalResidencyPool::open(store(134), limits(128, 2, 2, 4096, 2)).unwrap();
    let session = pool.begin_dirty_generation_capture().unwrap();
    let allocation = pool
        .begin_dirty_generation_capture_allocation(&session, nonzero_bytes(4096))
        .unwrap();
    assert_eq!(allocation.bytes(), capture_bytes(2));
    assert!(allocation.bytes() < 4096);
    drop(allocation);
    assert_eq!(pool.counters().active_operation_bytes(), 0);
    assert!(!pool.close().requires_inspection());
}

#[test]
fn policy_below_one_basis_denies_before_native_reservation() {
    let pool = PhysicalResidencyPool::open(store(135), limits(128, 1, 1, 2048, 1)).unwrap();
    let session = pool.begin_dirty_generation_capture().unwrap();
    let before = pool.allocation_events().snapshot();
    assert_eq!(
        pool.begin_dirty_generation_capture_allocation(&session, NonZeroU64::MIN)
            .unwrap_err(),
        PhysicalResidencyDenial::DirtyGenerationCaptureBudgetExceeded {
            required: capture_bytes(1),
            admitted: 1
        }
    );
    assert_eq!(pool.counters().active_operation_bytes(), 0);
    assert_eq!(
        pool.allocation_events()
            .snapshot()
            .for_dimension(PhysicalResidencyDimension::OperationBytes),
        before.for_dimension(PhysicalResidencyDimension::OperationBytes)
    );
    assert!(!pool.close().requires_inspection());
}
