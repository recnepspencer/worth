//! Unrepresentable declared bounds and totals are refused before admission.
use super::*;
use crate::inspection::mvcc::partition_allocations::admitted_bytes;
use crate::inspection::mvcc::sharing::RelationalBranchSharingInspectionDenial as Denial;
use crate::storage::substrate::{StorageAllocationExtent, StorageAllocationObservation};

fn carrier(bytes: u64) -> StorageAllocationObservation {
    StorageAllocationObservation {
        id: 1,
        bytes,
        unique: false,
        extent: StorageAllocationExtent::Checked,
    }
}

#[test]
fn unrepresentable_carrier_bound_is_refused_before_it_becomes_evidence() {
    let mut representable = carrier(64);
    representable.add_checked_nested_bound(Some(4096));
    assert_eq!(admitted_bytes(representable), Ok(4160));

    // Either the nested bound itself or its sum with the layout overflows.
    for bound in [None, Some(u64::MAX)] {
        let mut unrepresentable = carrier(64);
        unrepresentable.add_checked_nested_bound(bound);
        assert_eq!(unrepresentable.extent, StorageAllocationExtent::Overflow);
        assert_eq!(
            admitted_bytes(unrepresentable),
            Err(Denial::AuthoritativeByteBoundOverflow)
        );
    }
}

#[test]
fn unrepresentable_total_of_per_carrier_bounds_is_refused_not_saturated() {
    let locator = |owner| {
        RelationalAuthoritativeAllocationLocator::new(
            1,
            RelationalAuthoritativeAllocationKind::PartitionPayload,
            owner,
            owner,
            None,
        )
    };
    let mut accounting = RelationalAuthoritativeAllocationAccounting::default();
    accounting.insert(locator(1), u64::MAX - 1);
    accounting.insert(locator(2), 1);
    assert_eq!(accounting.unique_bytes(), Ok(u64::MAX));
    accounting.insert(locator(3), 1);
    assert_eq!(
        accounting.unique_bytes(),
        Err(Denial::AuthoritativeByteBoundOverflow)
    );

    accounting.logical_partition_bytes = u64::MAX;
    assert_eq!(accounting.logical_bytes(), Ok(u64::MAX));
    accounting.logical_root_metadata_bytes = 1;
    assert_eq!(
        accounting.logical_bytes(),
        Err(Denial::AuthoritativeByteBoundOverflow)
    );
}
