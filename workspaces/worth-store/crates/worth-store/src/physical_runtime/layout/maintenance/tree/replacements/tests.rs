use super::*;
use crate::physical_runtime::{
    record_serving::PhysicalScopedAllocationAdmission, PhysicalRecordResidencyFailureReason,
};
use worth_store_buffer_pool::PhysicalOperationAllocationScope;

#[test]
fn insertion_backing_survives_scratch_drop_and_growth_denies_without_changing_backing() {
    let (ports, _, runtime, generation) = super::super::allocation_test_ports::ports(4096);
    let admission = PhysicalScopedAllocationAdmission::new(&ports, runtime, generation);
    let allocation = admission
        .admit_maintenance(NonZeroU64::new(RETAINED_BACKING_BYTES).unwrap())
        .unwrap();
    let mut records = ChargedReplacementRecords::from_allocation(allocation);
    records.prepare_insertion().unwrap();
    let first = PersistedRecordIdentity::new([1; 16], 1).unwrap();
    records.append_prepared(ReplacementPath::one(first));
    let retained = retained_capacity_charge(usize::from(MAXIMUM_HEIGHT)).unwrap();
    let scratch = admission
        .admit_maintenance(NonZeroU64::new(512).unwrap())
        .unwrap();
    assert_eq!(
        ports
            .counters()
            .active_operation_bytes_for(PhysicalOperationAllocationScope::Maintenance),
        retained + 512
    );
    drop(scratch);
    assert_eq!(
        ports
            .counters()
            .active_operation_bytes_for(PhysicalOperationAllocationScope::Maintenance),
        retained
    );
    let expected_growth = retained_capacity_charge(usize::from(MAXIMUM_HEIGHT) * 2).unwrap();
    let collision = admission
        .admit_maintenance(NonZeroU64::new(4096 - retained - expected_growth + 1).unwrap())
        .unwrap();
    let old_capacity = records.records.capacity();
    let failure = match records.prepare_insertion() {
        Err(PhysicalLayoutMaintenanceFailure::WriterAllocation(failure)) => failure,
        _ => panic!("replacement growth must deny at the issuing pool envelope"),
    };
    assert_eq!(
        failure.reason(),
        PhysicalRecordResidencyFailureReason::PhysicalPressure
    );
    let pressure = failure.pressure().unwrap();
    assert_eq!(
        pressure.scope(),
        PhysicalOperationAllocationScope::Maintenance
    );
    assert_eq!(pressure.requested(), expected_growth);
    assert_eq!(pressure.admitted(), 4096 - expected_growth + 1);
    assert_eq!(records.records(), &[first]);
    assert_eq!(records.records.capacity(), old_capacity);
    assert_eq!(records.allocation.bytes(), retained);
    drop(collision);
    records.prepare_insertion().unwrap();
    assert_eq!(records.records.capacity(), usize::from(MAXIMUM_HEIGHT) * 2);
    assert_eq!(records.allocation.bytes(), expected_growth);
    drop(records);
    assert_eq!(
        ports
            .counters()
            .active_operation_bytes_for(PhysicalOperationAllocationScope::Maintenance),
        0
    );
}
