use super::*;
use crate::physical_runtime::record_serving::PhysicalScopedAllocationAdmission;
use worth_store_buffer_pool::PhysicalOperationAllocationScope;

#[test]
fn unique_union_growth_denies_before_inserting_an_unfunded_record() {
    let (ports, _, runtime, generation) = super::super::super::allocation_test_ports::ports(16384);
    let admission = PhysicalScopedAllocationAdmission::new(&ports, runtime, generation);
    let initial = SET_HEADROOM_BYTES + RETAINED_BACKING_BYTES;
    let allocation = admission
        .admit_maintenance(NonZeroU64::new(initial).unwrap())
        .unwrap();
    let mut records =
        ChargedRetirementRecords::from_allocation(RetirementRecordBudget::for_test(2), allocation);
    let first = PersistedRecordIdentity::new([1; 16], 1).unwrap();
    let second = PersistedRecordIdentity::new([2; 16], 1).unwrap();
    records.insert(first).unwrap();
    records.insert(first).unwrap();
    let prior_charge = records.allocation.bytes();
    let collision = admission
        .admit_maintenance(NonZeroU64::new(16384 - prior_charge - SET_RECORD_BYTES + 1).unwrap())
        .unwrap();
    let failure = match records.insert(second) {
        Err(PhysicalLayoutMaintenanceFailure::WriterAllocation(failure)) => failure,
        _ => panic!("unique union growth must deny before allocating a tree node"),
    };
    assert_eq!(failure.pressure().unwrap().requested(), SET_RECORD_BYTES);
    assert_eq!(records.len(), 1);
    assert!(!records.contains(&second));
    assert_eq!(records.allocation.bytes(), prior_charge);
    drop(collision);
    records.insert(second).unwrap();
    let third = PersistedRecordIdentity::new([3; 16], 1).unwrap();
    assert!(matches!(
        records.insert(third),
        Err(PhysicalLayoutMaintenanceFailure::RetirementLimit)
    ));
    assert!(!records.contains(&third));
    drop(records);
    assert_eq!(
        ports
            .counters()
            .active_operation_bytes_for(PhysicalOperationAllocationScope::Maintenance),
        0
    );
}

#[test]
fn union_copy_accounts_for_live_source_set_and_destination_before_retention() {
    let (ports, _, runtime, generation) = super::super::super::allocation_test_ports::ports(32768);
    let admission = PhysicalScopedAllocationAdmission::new(&ports, runtime, generation);
    let make = || {
        let allocation = admission
            .admit_maintenance(
                NonZeroU64::new(SET_HEADROOM_BYTES + RETAINED_BACKING_BYTES).unwrap(),
            )
            .unwrap();
        ChargedRetirementRecords::from_allocation(RetirementRecordBudget::for_test(4), allocation)
    };
    let first = PersistedRecordIdentity::new([1; 16], 1).unwrap();
    let mut source = make();
    source.insert(first).unwrap();
    let (source_records, source_charge) = source.into_retained_parts().unwrap();
    let mut union = make();
    union.extend(&source_records).unwrap();
    let retained = retained_capacity_charge(1).unwrap();
    let live_before_copy = source_charge.bytes() + union.allocation.bytes();
    let collision = admission
        .admit_maintenance(NonZeroU64::new(32768 - live_before_copy - retained + 1).unwrap())
        .unwrap();
    let failure = match union.into_retained_parts() {
        Err(PhysicalLayoutMaintenanceFailure::WriterAllocation(failure)) => failure,
        _ => panic!("copy must fund source, set and prospective Vec concurrently"),
    };
    assert_eq!(failure.pressure().unwrap().requested(), retained);
    assert_eq!(failure.pressure().unwrap().admitted(), 32768 - retained + 1);
    assert_eq!(source_records, vec![first]);
    assert_eq!(source_charge.bytes(), retained);
    drop(collision);
    let mut union = make();
    union.extend(&source_records).unwrap();
    let (union_records, union_charge) = union.into_retained_parts().unwrap();
    assert_eq!(union_records, source_records);
    assert_eq!(
        ports
            .counters()
            .active_operation_bytes_for(PhysicalOperationAllocationScope::Maintenance),
        retained * 2
    );
    drop((union_records, union_charge, source_records, source_charge));
    assert_eq!(
        ports
            .counters()
            .active_operation_bytes_for(PhysicalOperationAllocationScope::Maintenance),
        0
    );
}
