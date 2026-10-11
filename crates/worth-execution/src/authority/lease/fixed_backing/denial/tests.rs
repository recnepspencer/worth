use super::{
    ExecutionAllocationDenial, ExecutionAllocationDenialClass as Class,
    ExecutionAllocationDenialKind as Raw, ExecutionPhysicalAllocationDenialKind as Physical,
};
use crate::{LeaseDenial, MapKernelStop, WorkCeilingDenial};

#[test]
fn all_raw_allocation_kinds_have_one_exact_classification() {
    let lease = LeaseDenial::ChargedBytesOverflow;
    let cases = [
        (Raw::Layout, Class::Physical(Physical::Layout)),
        (
            Raw::Lease(lease),
            Class::Request(WorkCeilingDenial::Admission(lease)),
        ),
        (
            Raw::Cancelled,
            Class::Request(WorkCeilingDenial::Stopped(MapKernelStop::Cancelled)),
        ),
        (
            Raw::DeadlineElapsed,
            Class::Request(WorkCeilingDenial::Stopped(MapKernelStop::DeadlineElapsed)),
        ),
        (Raw::Allocator, Class::Physical(Physical::Allocator)),
        (
            Raw::CapacityMismatch,
            Class::Physical(Physical::CapacityMismatch),
        ),
        (
            Raw::WriteBeyondReserved,
            Class::Physical(Physical::WriteBeyondReserved),
        ),
        (
            Raw::IncompleteSeal,
            Class::Physical(Physical::IncompleteSeal),
        ),
    ];
    for (raw, expected) in cases {
        assert_eq!(raw.classify(), expected);
        assert_eq!(
            ExecutionAllocationDenial::new(raw, Some(17)).classify(),
            expected
        );
    }
}
