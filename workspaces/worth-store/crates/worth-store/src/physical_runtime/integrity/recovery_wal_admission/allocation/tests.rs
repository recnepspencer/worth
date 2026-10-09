//! Owning inventory reservations use the qualified Coordination's native pool.

use super::*;
use crate::physical_runtime::AdmittedRecoveryFilesystemMedia;
use worth_store_buffer_pool::{
    PhysicalOperationAllocationScope as Scope, PhysicalResidencyAllocationEventSnapshot,
    PhysicalResidencyDimension as Dimension,
};

fn fixture() -> (
    tempfile::TempDir,
    AdmittedRecoveryFilesystemMedia,
    PhysicalRecoveryCoordination,
) {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path().join("store");
    super::super::media_generation_tests::initialize(&root);
    let (media, coordination) =
        super::super::media_generation_tests::recovery_media_and_coordination(&root);
    (directory, media, coordination)
}

fn assert_no_admission(
    before: PhysicalResidencyAllocationEventSnapshot,
    after: PhysicalResidencyAllocationEventSnapshot,
) {
    for dimension in [
        Dimension::OperationBytes,
        Dimension::OperationScope(Scope::Recovery),
        Dimension::TotalBytes,
    ] {
        assert_eq!(
            before.for_dimension(dimension).admissions(),
            after.for_dimension(dimension).admissions()
        );
        assert_eq!(
            before.for_dimension(dimension).admitted_units(),
            after.for_dimension(dimension).admitted_units()
        );
    }
}

#[test]
fn inventory_backing_funds_overlap_settles_and_survives_coordination_disposal() {
    let (_directory, media, coordination) = fixture();
    let (owner, _, _) = coordination.sampling_allocation_basis().unwrap();
    let ports = owner.ports().clone();
    let observer = ports.allocation_events();
    let mut backing = coordination
        .admit_wal_inventory_backing(NonZeroU64::new(64).unwrap())
        .unwrap();
    let mut old = Vec::<u8>::new();
    old.try_reserve_exact(64).unwrap();
    assert_eq!(old.capacity(), 64);
    old.extend_from_slice(b"funded-inventory");
    backing.grow_total(64 + 128).unwrap();
    let mut next = Vec::<u8>::new();
    next.try_reserve_exact(128).unwrap();
    assert_eq!(next.capacity(), 128);
    next.extend_from_slice(&old);
    assert_eq!(
        ports.counters().active_operation_bytes_for(Scope::Recovery),
        (old.capacity() + next.capacity()) as u64
    );
    drop(old);
    backing
        .settle_after_disposal(next.capacity() as u64)
        .unwrap();
    assert_eq!(backing.charged_bytes(), 128);
    let before = observer.snapshot();
    backing.grow_total(0).unwrap();
    backing.grow_total(128).unwrap();
    assert_no_admission(before, observer.snapshot());
    assert_eq!(backing.charged_bytes(), 128);
    drop(coordination);
    drop(media);
    assert_eq!(next, b"funded-inventory");
    assert_eq!(
        ports.counters().active_operation_bytes_for(Scope::Recovery),
        next.capacity() as u64
    );
    let before = observer.snapshot();
    let Denial::Backing {
        cause: PhysicalRecoveryRejoinResidentDenial::OperationAllocation(failure),
        ..
    } = backing.grow_total(129).unwrap_err()
    else {
        panic!("closed native pool must deny further growth");
    };
    assert_eq!(
        failure.reason(),
        crate::physical_runtime::PhysicalRecordResidencyFailureReason::PoolClosed
    );
    assert_no_admission(before, observer.snapshot());
    assert_eq!(backing.charged_bytes(), 128);
    drop(next);
    backing.settle_after_disposal(0).unwrap();
    assert_eq!(ports.counters().active_operation_bytes(), 0);
    drop(backing);
    drop(ports);
    assert_eq!(
        observer
            .snapshot()
            .for_dimension(Dimension::TotalBytes)
            .active_units(),
        0
    );
}

#[test]
fn original_ceiling_denies_initial_and_growth_without_admission_or_late_settlement() {
    let (_directory, _media, coordination) = fixture();
    let (owner, original, _) = coordination.sampling_allocation_basis().unwrap();
    let ports = owner.ports().clone();
    let observer = ports.allocation_events();
    let before = observer.snapshot();
    assert!(matches!(
        coordination.admit_wal_inventory_backing(
            NonZeroU64::new(original.byte_limit() + 1).unwrap()
        ),
        Err(Denial::Backing {
            cause: PhysicalRecoveryRejoinResidentDenial::BudgetExceeded { required, admitted },
            ..
        }) if required == original.byte_limit() + 1 && admitted == original.byte_limit()
    ));
    assert_no_admission(before, observer.snapshot());
    let mut backing = coordination
        .admit_wal_inventory_backing(NonZeroU64::new(64).unwrap())
        .unwrap();
    let held = ports
        .begin_operation(
            Scope::Recovery,
            NonZeroU64::new(original.byte_limit() - 65).unwrap(),
        )
        .unwrap();
    let before = observer.snapshot();
    assert!(matches!(
        backing.grow_total(66),
        Err(Denial::Backing {
            requested: 66,
            cause: PhysicalRecoveryRejoinResidentDenial::BudgetExceeded { required, admitted },
        }) if required == original.byte_limit() + 1 && admitted == original.byte_limit()
    ));
    assert_no_admission(before, observer.snapshot());
    assert_eq!(backing.charged_bytes(), 64);
    assert_eq!(
        backing.settle_after_disposal(65),
        Err(Denial::AllocatorExceededReservation {
            requested: 64,
            actual: 65
        })
    );
    assert_no_admission(before, observer.snapshot());
    assert_eq!(backing.charged_bytes(), 64);
    drop(held);
    backing.grow_total(66).unwrap();
    assert_eq!(
        ports.counters().active_operation_bytes_for(Scope::Recovery),
        66
    );
    drop(backing);
    assert_eq!(ports.counters().active_operation_bytes(), 0);
}

#[test]
fn aggregate_pressure_preserves_native_cause_charge_and_admission_counters() {
    let (_directory, _media, coordination) = fixture();
    let (owner, _, generation) = coordination.sampling_allocation_basis().unwrap();
    let ports = owner.ports().clone();
    let policy = owner.admitted_policy();
    // Recovery must leave the declared protected progress bytes available,
    // even though the competing Maintenance grant may spend that headroom.
    let limit = policy
        .operation_bytes()
        .checked_sub(policy.limits().progress_headroom_bytes())
        .unwrap();
    let mut backing = coordination
        .admit_wal_inventory_backing(NonZeroU64::new(64).unwrap())
        .unwrap();
    let held = ports
        .begin_operation(Scope::Maintenance, NonZeroU64::new(limit - 65).unwrap())
        .unwrap();
    let before = ports.allocation_events().snapshot();
    let Denial::Backing {
        requested: 66,
        cause: PhysicalRecoveryRejoinResidentDenial::OperationAllocation(failure),
    } = backing.grow_total(66).unwrap_err()
    else {
        panic!("native aggregate pressure must remain typed");
    };
    let pressure = failure.pressure().unwrap();
    assert_eq!(pressure.dimension(), Dimension::OperationBytes);
    assert_eq!(pressure.admitted(), limit - 1);
    assert_eq!(pressure.requested(), 2);
    assert_eq!(pressure.limit(), limit);
    assert_eq!(pressure.store_generation(), generation);
    assert!(!pressure.effect_may_have_started());
    assert_no_admission(before, ports.allocation_events().snapshot());
    assert_eq!(backing.charged_bytes(), 64);
    drop(held);
    backing.grow_total(66).unwrap();
    assert_eq!(
        ports.counters().active_operation_bytes_for(Scope::Recovery),
        66
    );
    drop(backing);
    assert_eq!(ports.counters().active_operation_bytes(), 0);
}
