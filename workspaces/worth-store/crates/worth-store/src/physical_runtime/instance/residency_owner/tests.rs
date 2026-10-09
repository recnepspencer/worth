use std::num::{NonZeroU32, NonZeroU64};

use worth_store_buffer_pool::{
    PhysicalOperationAllocationScope, PhysicalResidencyDenial, PhysicalResidencyDimension,
};
use worth_store_physical_format::store_namespace::{
    ProposedStoreIdentity, StoreNamespaceIdentityRecord, StoreNamespaceVersion,
};

use super::PhysicalResidencyOwner;
use crate::physical_runtime::record_serving::{
    AdmittedPhysicalRecordFormat, AdmittedPhysicalRecordResidencyPolicy,
    PhysicalRecordFormatDeclaration, PhysicalRecordResidencyPolicy, PhysicalSpeculativeWorkKind,
};

#[test]
fn abandoned_owner_closes_surviving_facades_and_final_release_reconciles() {
    let owner = PhysicalResidencyOwner::admit(store(111), admitted_policy()).unwrap();
    let ports = owner.ports().clone();
    let observer = ports.allocation_events();
    drop(owner);

    assert_eq!(
        ports
            .begin_operation(
                PhysicalOperationAllocationScope::ForegroundRead,
                NonZeroU64::MIN,
            )
            .unwrap_err(),
        PhysicalResidencyDenial::PoolClosed,
    );
    drop(ports);
    for dimension in [
        PhysicalResidencyDimension::MetadataBytes,
        PhysicalResidencyDimension::TotalBytes,
    ] {
        assert_eq!(
            observer.snapshot().for_dimension(dimension).active_units(),
            0,
        );
    }
}

#[test]
fn moved_owner_preserves_pool_and_live_grant_until_terminal_close() {
    let owner = PhysicalResidencyOwner::admit(store(117), admitted_policy()).unwrap();
    let ports = owner.ports().clone();
    let observer = ports.allocation_events();
    let pool = observer.snapshot().pool();
    let grant = ports
        .begin_operation(PhysicalOperationAllocationScope::Recovery, NonZeroU64::MIN)
        .unwrap();
    let moved = owner;
    moved
        .validate_recovered_policy(store(117), admitted_policy())
        .unwrap();
    assert_eq!(moved.ports().allocation_events().snapshot().pool(), pool);
    assert_eq!(moved.ports().counters().active_operation_bytes(), 1);
    drop(grant);
    let shutdown = moved.close();
    assert!(!shutdown.requires_inspection());
    assert_eq!(
        ports
            .begin_operation(PhysicalOperationAllocationScope::Recovery, NonZeroU64::MIN)
            .unwrap_err(),
        PhysicalResidencyDenial::PoolClosed
    );
    drop(ports);
    assert_eq!(
        observer
            .snapshot()
            .for_dimension(PhysicalResidencyDimension::TotalBytes)
            .active_units(),
        0
    );
}

#[test]
fn recovered_policy_validation_rejects_same_store_envelope_drift_without_mutation() {
    let owner = PhysicalResidencyOwner::admit(store(118), admitted_policy()).unwrap();
    let before = owner.ports().allocation_events().snapshot();
    assert_eq!(owner.validate_recovered_policy(store(118), admitted_policy_with_recovery_scope(true)), Err(crate::physical_runtime::record_serving::RecordBootstrapDenial::RecoveredResidencyPolicyMismatch));
    assert_eq!(owner.admitted_policy(), admitted_policy());
    assert_eq!(owner.ports().allocation_events().snapshot(), before);
}

#[test]
fn recovered_policy_validation_rejects_store_drift_before_transfer() {
    let owner = PhysicalResidencyOwner::admit(store(119), admitted_policy()).unwrap();
    assert_eq!(owner.validate_recovered_policy(store(120), admitted_policy()), Err(crate::physical_runtime::record_serving::RecordBootstrapDenial::RecoveredResidencyStoreMismatch));
    owner
        .validate_recovered_policy(store(119), admitted_policy())
        .unwrap();
}

#[test]
fn explicit_close_consumes_the_only_lifecycle_owner() {
    let owner = PhysicalResidencyOwner::admit(store(112), admitted_policy()).unwrap();
    let ports = owner.ports().clone();
    let shutdown = owner.close();
    assert!(!shutdown.requires_inspection());
    assert_eq!(
        ports
            .begin_operation(
                PhysicalOperationAllocationScope::ForegroundRead,
                NonZeroU64::MIN,
            )
            .unwrap_err(),
        PhysicalResidencyDenial::PoolClosed,
    );
}

#[test]
fn explicit_close_classifies_live_facade_allocation_as_residue() {
    let owner = PhysicalResidencyOwner::admit(store(113), admitted_policy()).unwrap();
    let ports = owner.ports().clone();
    let allocation = ports
        .begin_operation(
            PhysicalOperationAllocationScope::ForegroundRead,
            NonZeroU64::MIN,
        )
        .unwrap();

    let shutdown = owner.close();
    assert!(shutdown.requires_inspection());
    assert!(shutdown.has_cancellable_work_residue());
    assert_eq!(shutdown.counters().active_operation_bytes(), 1);

    drop(allocation);
    assert_eq!(ports.counters().active_operation_bytes(), 0);
}

#[test]
fn recovered_allocation_keeps_original_ceiling_and_current_pool_residual() {
    let identity = store(114);
    let original_owner = PhysicalResidencyOwner::admit(identity, admitted_policy()).unwrap();
    let original = original_owner.recovery_allocation_admission();
    let mut owner =
        PhysicalResidencyOwner::admit(identity, admitted_policy_with_recovery_scope(true)).unwrap();
    let current_available = owner.available_recovery_operation_bytes();
    let original_limit = original.byte_limit();
    assert!(original_limit > 1 && original_limit < current_available);
    owner
        .restrict_recovery_allocation(original, recovery_generation())
        .unwrap();
    assert_eq!(owner.recovery_allocation_admission(), original);
    assert_eq!(owner.available_recovery_operation_bytes(), original_limit);

    let in_use = original_limit / 2;
    let grant = owner
        .ports()
        .begin_operation(
            PhysicalOperationAllocationScope::Recovery,
            NonZeroU64::new(in_use).unwrap(),
        )
        .unwrap();
    assert_eq!(
        owner.available_recovery_operation_bytes(),
        original_limit - in_use
    );
    drop(grant);
    assert_eq!(owner.available_recovery_operation_bytes(), original_limit);
}

#[test]
fn recovered_allocation_rejects_foreign_store_and_cannot_expand_current_pool() {
    let identity = store(115);
    let original_owner =
        PhysicalResidencyOwner::admit(identity, admitted_policy_with_recovery_scope(true)).unwrap();
    let original = original_owner.recovery_allocation_admission();
    let foreign_owner = PhysicalResidencyOwner::admit(store(116), admitted_policy()).unwrap();
    let mut owner = PhysicalResidencyOwner::admit(identity, admitted_policy()).unwrap();
    let current_available = owner.available_recovery_operation_bytes();
    let current_admission = owner.recovery_allocation_admission();
    assert_eq!(
        owner.restrict_recovery_allocation(
            foreign_owner.recovery_allocation_admission(),
            recovery_generation()
        ),
        Err(PhysicalResidencyDenial::WrongStore)
    );
    assert_eq!(owner.recovery_allocation_admission(), current_admission);
    assert!(original.byte_limit() > current_available);
    owner
        .restrict_recovery_allocation(original, recovery_generation())
        .unwrap();
    assert_eq!(owner.recovery_allocation_admission(), original);
    assert_eq!(
        owner.available_recovery_operation_bytes(),
        current_available
    );
}

#[test]
fn recovery_restriction_is_atomic_with_its_carried_original_basis() {
    use crate::physical_runtime::PhysicalRecoveryAllocationAdmission;

    let identity = store(119);
    let mut owner = PhysicalResidencyOwner::admit(identity, admitted_policy()).unwrap();
    let original = PhysicalRecoveryAllocationAdmission::new(identity, 64);
    owner
        .restrict_recovery_allocation(original, recovery_generation())
        .unwrap();
    let ports = owner.ports().clone();
    let live = ports
        .begin_operation(PhysicalOperationAllocationScope::Recovery, nonzero(40))
        .unwrap();

    let denial = owner
        .restrict_recovery_allocation(
            PhysicalRecoveryAllocationAdmission::new(identity, 32),
            recovery_generation(),
        )
        .unwrap_err();
    let PhysicalResidencyDenial::Pressure(pressure) = denial else {
        panic!("restriction below a live native charge must preserve pressure");
    };
    assert_eq!(pressure.requested(), 0);
    assert_eq!(pressure.current(), 40);
    assert_eq!(pressure.limit(), 32);
    assert_eq!(owner.recovery_allocation_admission(), original);
    assert_eq!(owner.available_recovery_operation_bytes(), 24);

    owner
        .restrict_recovery_allocation(
            PhysicalRecoveryAllocationAdmission::new(identity, 128),
            recovery_generation(),
        )
        .unwrap();
    assert_eq!(owner.recovery_allocation_admission(), original);
    let PhysicalResidencyDenial::Pressure(pressure) = ports
        .begin_operation(PhysicalOperationAllocationScope::Recovery, nonzero(25))
        .unwrap_err()
    else {
        panic!("a cloned issuer cannot widen the original ceiling");
    };
    assert_eq!(pressure.current(), 40);
    assert_eq!(pressure.requested(), 25);
    assert_eq!(pressure.limit(), 64);
    drop(live);
    let mut retry = ports
        .begin_operation(PhysicalOperationAllocationScope::Recovery, nonzero(64))
        .unwrap();
    assert!(retry.try_resize(65).is_err());
    assert_eq!(retry.bytes(), 64);
    drop(retry);
    assert_eq!(ports.counters().active_operation_bytes(), 0);
}

fn store(byte: u8) -> worth_store_physical_format::store_namespace::StableStoreIdentity {
    StoreNamespaceIdentityRecord::new(
        StoreNamespaceVersion::CURRENT,
        ProposedStoreIdentity::from_nonzero_bytes([byte; 16]).unwrap(),
    )
    .published_identity()
}

fn admitted_policy() -> AdmittedPhysicalRecordResidencyPolicy {
    admitted_policy_with_recovery_scope(false)
}

fn admitted_policy_with_recovery_scope(
    expanded_recovery_scope: bool,
) -> AdmittedPhysicalRecordResidencyPolicy {
    use PhysicalOperationAllocationScope as Scope;
    use PhysicalSpeculativeWorkKind as Kind;

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
        .operation_bytes(nonzero(if expanded_recovery_scope {
            page * 2
        } else {
            page
        }));
    for scope in [
        Scope::ForegroundRead,
        Scope::ForegroundWrite,
        Scope::Recovery,
        Scope::Scrub,
        Scope::Maintenance,
        Scope::Verification,
        Scope::Blob,
    ] {
        let bytes = if scope == Scope::Recovery && expanded_recovery_scope {
            page * 2
        } else {
            page
        };
        builder = builder.scope_bytes(scope, nonzero(bytes));
    }
    for kind in [Kind::ReadAhead, Kind::Prefetch, Kind::WriteBehind] {
        builder = builder.speculative_frames(kind, NonZeroU32::new(2).unwrap());
    }
    builder.admit(format).into_result().unwrap()
}

fn nonzero(value: u64) -> NonZeroU64 {
    NonZeroU64::new(value).unwrap()
}

fn recovery_generation() -> crate::physical_runtime::LifecycleGeneration {
    crate::physical_runtime::LifecycleGeneration::from_reopened(NonZeroU64::new(7).unwrap())
}

#[test]
fn recovery_origin_is_carried_and_cannot_be_rebound_by_another_lifecycle() {
    let mut owner = PhysicalResidencyOwner::admit(store(123), admitted_policy()).unwrap();
    let original = owner.recovery_allocation_admission();
    assert_eq!(owner.recovery_origin_generation(), None);
    owner
        .restrict_recovery_allocation(original, recovery_generation())
        .unwrap();
    let before = owner.ports().allocation_events().snapshot();
    let other =
        crate::physical_runtime::LifecycleGeneration::from_reopened(NonZeroU64::new(8).unwrap());
    assert_eq!(
        owner.restrict_recovery_allocation(original, other),
        Err(PhysicalResidencyDenial::AllocationGrantMismatch)
    );
    assert_eq!(owner.ports().allocation_events().snapshot(), before);
    let moved = owner;
    assert_eq!(
        moved.recovery_origin_generation(),
        Some(recovery_generation())
    );
    assert_eq!(moved.recovery_allocation_admission(), original);
    assert_eq!(
        moved.ports().allocation_events().snapshot().pool(),
        before.pool()
    );
}
