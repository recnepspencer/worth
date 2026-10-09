use std::num::{NonZeroU32, NonZeroU64};

use worth_store_buffer_pool::{
    PhysicalOperationAllocationScope as Scope, PhysicalResidencyDimension,
};
use worth_store_physical_format::store_namespace::{
    ProposedStoreIdentity, StoreNamespaceIdentityRecord, StoreNamespaceVersion,
};

use super::*;
use crate::physical_runtime::record_serving::{
    AdmittedPhysicalRecordFormat, PhysicalRecordFormatDeclaration, PhysicalRecordResidencyPolicy,
    PhysicalResidencyRetryPosture, PhysicalSpeculativeWorkKind,
};

#[test]
fn zero_source_window_funds_growth_and_releases_exact_native_charge() {
    let mut owner = owner();
    let ports = owner.ports().clone();
    let observer = ports.allocation_events();
    let original = owner.recovery_allocation_admission();
    let before = observer.snapshot();
    let mut window = PhysicalRecoveryReadAllocation::new(&mut owner, original, generation());
    assert_eq!(window.charged_bytes(), 0);
    window.reserve_total(0).unwrap();
    assert_eq!(observer.snapshot(), before);
    window.reserve_total(16).unwrap();
    assert_eq!(window.charged_bytes(), 16);
    assert_eq!(
        ports.counters().active_operation_bytes_for(Scope::Recovery),
        16
    );
    window.reserve_total(64).unwrap();
    assert_eq!(window.charged_bytes(), 64);
    window.reserve_total(1).unwrap();
    assert_eq!(window.charged_bytes(), 64);
    assert_eq!(
        ports.counters().active_operation_bytes_for(Scope::Recovery),
        64
    );
    drop(window);
    assert_eq!(ports.counters().active_operation_bytes(), 0);
    drop(ports);
    drop(owner);
    assert_eq!(
        observer
            .snapshot()
            .for_dimension(PhysicalResidencyDimension::TotalBytes)
            .active_units(),
        0
    );
}

#[test]
fn original_ceiling_includes_other_live_recovery_before_native_growth() {
    let mut owner = owner();
    let ports = owner.ports().clone();
    let original = PhysicalRecoveryAllocationAdmission::new(ports.store_identity(), 64);
    owner
        .restrict_recovery_allocation(original, generation())
        .unwrap();
    let other = ports.begin_operation(Scope::Recovery, nonzero(40)).unwrap();
    let mut window = PhysicalRecoveryReadAllocation::new(&mut owner, original, generation());
    window.reserve_total(24).unwrap();
    let before = ports.allocation_events().snapshot();
    assert_eq!(
        window.reserve_total(25),
        Err(PhysicalRecoveryRejoinResidentDenial::BudgetExceeded {
            required: 65,
            admitted: 64,
        })
    );
    assert_eq!(window.charged_bytes(), 24);
    let after = ports.allocation_events().snapshot();
    for dimension in [
        PhysicalResidencyDimension::OperationScope(Scope::Recovery),
        PhysicalResidencyDimension::OperationBytes,
        PhysicalResidencyDimension::TotalBytes,
    ] {
        assert_eq!(
            after.for_dimension(dimension).admitted_units(),
            before.for_dimension(dimension).admitted_units(),
        );
        assert_eq!(
            after.for_dimension(dimension).active_units(),
            before.for_dimension(dimension).active_units(),
        );
    }
    assert_eq!(
        after
            .for_dimension(PhysicalResidencyDimension::OperationScope(Scope::Recovery))
            .denials(),
        before
            .for_dimension(PhysicalResidencyDimension::OperationScope(Scope::Recovery))
            .denials()
            + 1,
    );
    assert_eq!(
        ports.counters().active_operation_bytes_for(Scope::Recovery),
        64
    );
    drop(window);
    assert_eq!(
        ports.counters().active_operation_bytes_for(Scope::Recovery),
        40
    );
    drop(other);
    assert_eq!(ports.counters().active_operation_bytes(), 0);
}

#[test]
fn native_aggregate_pressure_preserves_zero_and_existing_source_charge() {
    let mut owner = owner();
    let ports = owner.ports().clone();
    let original = owner.recovery_allocation_admission();
    let limit = owner.admitted_policy().operation_bytes();
    let mut other = ports
        .begin_operation(Scope::Maintenance, nonzero(limit - 1))
        .unwrap();
    let mut window = PhysicalRecoveryReadAllocation::new(&mut owner, original, generation());
    let before_first_denial = ports.allocation_events().snapshot();
    assert_pressure(window.reserve_total(2).unwrap_err(), 2, limit - 1, limit);
    let after_first_denial = ports.allocation_events().snapshot();
    for dimension in [
        PhysicalResidencyDimension::OperationBytes,
        PhysicalResidencyDimension::OperationScope(Scope::Recovery),
        PhysicalResidencyDimension::TotalBytes,
    ] {
        let before = before_first_denial.for_dimension(dimension);
        let after = after_first_denial.for_dimension(dimension);
        assert_eq!(after.admitted_units(), before.admitted_units());
        assert_eq!(after.admissions(), before.admissions());
    }
    assert_eq!(window.charged_bytes(), 0);
    assert_eq!(ports.counters().active_operation_bytes(), limit - 1);
    other.try_resize(limit - 17).unwrap();
    window.reserve_total(16).unwrap();
    let before_growth_denial = ports.allocation_events().snapshot();
    assert_pressure(window.reserve_total(18).unwrap_err(), 2, limit - 1, limit);
    let after_growth_denial = ports.allocation_events().snapshot();
    for dimension in [
        PhysicalResidencyDimension::OperationBytes,
        PhysicalResidencyDimension::OperationScope(Scope::Recovery),
        PhysicalResidencyDimension::TotalBytes,
    ] {
        let before = before_growth_denial.for_dimension(dimension);
        let after = after_growth_denial.for_dimension(dimension);
        assert_eq!(after.admitted_units(), before.admitted_units());
        assert_eq!(after.admissions(), before.admissions());
    }
    assert_eq!(window.charged_bytes(), 16);
    assert_eq!(ports.counters().active_operation_bytes(), limit - 1);
    drop(other);
    window.reserve_total(18).unwrap();
    assert_eq!(ports.counters().active_operation_bytes(), 18);
    let observer = ports.allocation_events();
    drop(window);
    assert_eq!(ports.counters().active_operation_bytes(), 0);
    drop(ports);
    drop(owner);
    assert_eq!(
        observer
            .snapshot()
            .for_dimension(PhysicalResidencyDimension::TotalBytes)
            .active_units(),
        0
    );
}

fn assert_pressure(
    denial: PhysicalRecoveryRejoinResidentDenial,
    requested: u64,
    current: u64,
    limit: u64,
) {
    let PhysicalRecoveryRejoinResidentDenial::OperationAllocation(failure) = denial else {
        panic!("source backing must preserve the native allocation failure");
    };
    let pressure = failure.pressure().unwrap();
    assert_eq!(pressure.scope(), Scope::Recovery);
    assert_eq!(
        pressure.dimension(),
        PhysicalResidencyDimension::OperationBytes
    );
    assert_eq!(pressure.requested(), requested);
    assert_eq!(pressure.admitted(), current);
    assert_eq!(pressure.limit(), limit);
    assert_eq!(pressure.store_generation(), generation());
    assert_eq!(
        pressure.retry_posture(),
        PhysicalResidencyRetryPosture::AfterAllocationRelease
    );
    assert!(!pressure.effect_may_have_started());
}

fn owner() -> PhysicalResidencyOwner {
    let store = StoreNamespaceIdentityRecord::new(
        StoreNamespaceVersion::CURRENT,
        ProposedStoreIdentity::from_nonzero_bytes([131; 16]).unwrap(),
    )
    .published_identity();
    let format = AdmittedPhysicalRecordFormat::admit(
        PhysicalRecordFormatDeclaration::builder().admit().unwrap(),
    );
    let page = u64::from(format.declaration().page_size().bytes());
    let mut builder = PhysicalRecordResidencyPolicy::builder()
        .total_bytes(nonzero(page * 8))
        .resident_bytes(nonzero(page * 4))
        .metadata_bytes(nonzero(page))
        .frame_entries(NonZeroU32::new(4).unwrap())
        .pinned_frames(NonZeroU32::new(4).unwrap())
        .pin_leases(NonZeroU32::new(4).unwrap())
        .dirty_frames(NonZeroU32::new(2).unwrap())
        .dirty_replacement_bytes(nonzero(page))
        .operation_bytes(nonzero(page));
    for scope in [
        Scope::ForegroundRead,
        Scope::ForegroundWrite,
        Scope::Recovery,
        Scope::Scrub,
        Scope::Maintenance,
        Scope::Verification,
        Scope::Blob,
    ] {
        builder = builder.scope_bytes(scope, nonzero(page));
    }
    for kind in [
        PhysicalSpeculativeWorkKind::ReadAhead,
        PhysicalSpeculativeWorkKind::Prefetch,
        PhysicalSpeculativeWorkKind::WriteBehind,
    ] {
        builder = builder.speculative_frames(kind, NonZeroU32::new(2).unwrap());
    }
    PhysicalResidencyOwner::admit(store, builder.admit(format).into_result().unwrap()).unwrap()
}

fn generation() -> LifecycleGeneration {
    LifecycleGeneration::from_reopened(NonZeroU64::new(7).unwrap())
}

fn nonzero(bytes: u64) -> NonZeroU64 {
    NonZeroU64::new(bytes).unwrap()
}
