use super::*;
use std::num::NonZeroU64;

use worth_store_buffer_pool::PhysicalOperationAllocationScope;

use crate::physical_runtime::{
    record_serving::{PhysicalScopedAllocationAdmission, PreparedDerivedDirectoryBasis},
    LifecycleGeneration, RuntimeIdentity,
};

#[test]
fn prepared_directory_clones_hold_one_real_maintenance_charge_until_last_drop() {
    let (ports, store, _, _) = super::super::allocation_test_ports::ports(4096);
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
    let (ports, store, _, _) = super::super::allocation_test_ports::ports(4096);
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
