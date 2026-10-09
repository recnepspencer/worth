//! Storage refusals shared by native-backed binding evidence owners.

use crate::physical_runtime::PhysicalRecoveryRejoinResidentDenial;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum BindingStorageAllocationDenial {
    SizeOverflow,
    BackingMismatch,
    Backing {
        requested: u64,
        cause: PhysicalRecoveryRejoinResidentDenial,
    },
    AllocatorExceededReservation {
        requested: u64,
        actual: u64,
    },
}

impl From<BindingStorageAllocationDenial>
    for super::StoreRecoveryCheckpointBindingAllocationDenial
{
    fn from(denial: BindingStorageAllocationDenial) -> Self {
        match denial {
            BindingStorageAllocationDenial::SizeOverflow => Self::SizeOverflow,
            BindingStorageAllocationDenial::BackingMismatch => Self::BackingMismatch,
            BindingStorageAllocationDenial::Backing { requested, cause } => {
                Self::Backing { requested, cause }
            }
            BindingStorageAllocationDenial::AllocatorExceededReservation { requested, actual } => {
                Self::AllocatorExceededReservation { requested, actual }
            }
        }
    }
}
