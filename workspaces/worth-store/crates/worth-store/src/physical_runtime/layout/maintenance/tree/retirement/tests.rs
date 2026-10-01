use super::*;
use std::num::{NonZeroU32, NonZeroU64};

use worth_store_buffer_pool::{
    PhysicalOperationAllocationScope, PhysicalResidencyLimits, PhysicalSpeculativeWorkKind,
};
use worth_store_physical_format::store_namespace::{
    ProposedStoreIdentity, StoreNamespaceIdentityRecord, StoreNamespaceVersion,
};

use crate::physical_runtime::{
    record_serving::{
        PhysicalScopedAllocationAdmission, PreparedDerivedDirectoryBasis, RecordFramePorts,
    },
    LifecycleGeneration, RuntimeIdentity,
};

fn charged_directory_ports() -> (
    RecordFramePorts,
    worth_store_physical_format::store_namespace::StableStoreIdentity,
) {
    let store = StoreNamespaceIdentityRecord::new(
        StoreNamespaceVersion::CURRENT,
        ProposedStoreIdentity::from_nonzero_bytes([141; 16]).unwrap(),
    )
    .published_identity();
    let bytes = NonZeroU64::new(4096).unwrap();
    // This owner-lifetime test charges one KiB for a one-record Vec and its
    // shared backing. It does not stand in for production retirement geometry.
    let operation = NonZeroU64::new(4096).unwrap();
    let frames = NonZeroU32::new(4).unwrap();
    let mut limits = PhysicalResidencyLimits::builder()
        .total_bytes(NonZeroU64::new(12_288 + operation.get()).unwrap())
        .resident_bytes(bytes)
        .metadata_bytes(bytes)
        .frame_entries(frames)
        .pinned_frames(frames)
        .pin_leases(frames)
        .dirty_frames(frames)
        .dirty_replacement_bytes(bytes)
        .operation_bytes(operation);
    for scope in [
        PhysicalOperationAllocationScope::ForegroundRead,
        PhysicalOperationAllocationScope::ForegroundWrite,
        PhysicalOperationAllocationScope::Recovery,
        PhysicalOperationAllocationScope::Scrub,
        PhysicalOperationAllocationScope::Maintenance,
        PhysicalOperationAllocationScope::Verification,
        PhysicalOperationAllocationScope::Blob,
    ] {
        limits = limits.scope_bytes(scope, operation);
    }
    let limits = limits
        .speculative_frames(PhysicalSpeculativeWorkKind::Prefetch, frames)
        .speculative_frames(PhysicalSpeculativeWorkKind::ReadAhead, frames)
        .speculative_frames(PhysicalSpeculativeWorkKind::WriteBehind, frames)
        .admit(NonZeroU64::MIN)
        .unwrap();
    (RecordFramePorts::bounded(store, limits).unwrap(), store)
}

#[test]
fn retirement_union_checks_unique_count_before_insertion() {
    let first = PersistedRecordIdentity::new([1; 16], 1).unwrap();
    let second = PersistedRecordIdentity::new([2; 16], 1).unwrap();
    let third = PersistedRecordIdentity::new([3; 16], 1).unwrap();
    let budget = RetirementRecordBudget::for_test(2);
    let mut dropped = BTreeSet::new();

    extend_retirement(&mut dropped, &[first, second], budget).unwrap();
    extend_retirement(&mut dropped, &[second, first], budget).unwrap();
    assert_eq!(dropped.len(), 2);
    assert!(matches!(
        extend_retirement(&mut dropped, &[third], budget),
        Err(PhysicalLayoutMaintenanceFailure::RetirementLimit)
    ));
    assert!(!dropped.contains(&third));
}

#[test]
fn prepared_directory_clones_hold_one_real_maintenance_charge_until_last_drop() {
    let (ports, store) = charged_directory_ports();
    let runtime = RuntimeIdentity::from_reopened(NonZeroU64::new(11).unwrap());
    let generation = LifecycleGeneration::from_reopened(NonZeroU64::new(3).unwrap());
    let admission = PhysicalScopedAllocationAdmission::new(&ports, runtime, generation);
    let charged_bytes = NonZeroU64::new(1024).unwrap();
    let allocation = admission.admit_maintenance(charged_bytes).unwrap();
    let record = PersistedRecordIdentity::new([17; 16], 1).unwrap();
    let admitted = AdmittedDirectoryRetirement {
        records: vec![record],
        allocation: Some(allocation),
    };
    let basis = PreparedDerivedDirectoryBasis::from_admitted(
        None, None, None, admitted, store, runtime, generation,
    )
    .expect("the charged records bind the issuing Store and lifecycle");
    assert_eq!(basis.replaced_nodes(), &[record]);
    let clone = basis.clone();
    assert_eq!(
        ports
            .counters()
            .active_operation_bytes_for(PhysicalOperationAllocationScope::Maintenance),
        charged_bytes.get()
    );
    drop(basis);
    assert_eq!(
        ports
            .counters()
            .active_operation_bytes_for(PhysicalOperationAllocationScope::Maintenance),
        charged_bytes.get()
    );
    drop(clone);
    assert_eq!(
        ports
            .counters()
            .active_operation_bytes_for(PhysicalOperationAllocationScope::Maintenance),
        0
    );
}

#[test]
fn foreign_runtime_refusal_releases_the_original_maintenance_charge() {
    let (ports, store) = charged_directory_ports();
    let runtime = RuntimeIdentity::from_reopened(NonZeroU64::new(21).unwrap());
    let other_runtime = RuntimeIdentity::from_reopened(NonZeroU64::new(22).unwrap());
    let generation = LifecycleGeneration::from_reopened(NonZeroU64::new(3).unwrap());
    let admission = PhysicalScopedAllocationAdmission::new(&ports, runtime, generation);
    let allocation = admission
        .admit_maintenance(NonZeroU64::new(1024).unwrap())
        .unwrap();
    let admitted = AdmittedDirectoryRetirement {
        records: vec![PersistedRecordIdentity::new([23; 16], 1).unwrap()],
        allocation: Some(allocation),
    };
    assert!(PreparedDerivedDirectoryBasis::from_admitted(
        None,
        None,
        None,
        admitted,
        store,
        other_runtime,
        generation,
    )
    .is_none());
    assert_eq!(
        ports
            .counters()
            .active_operation_bytes_for(PhysicalOperationAllocationScope::Maintenance),
        0
    );
}
