//! Native backing for sampling scratch and its retained result.

use std::num::NonZeroU64;

use worth_store_buffer_pool::{
    OperationAllocationGrant, PhysicalOperationAllocationScope, PhysicalResidencyDenial,
    PhysicalResidencyDimension,
};

use crate::physical_runtime::{
    instance::PhysicalResidencyOwner, LifecycleGeneration, PhysicalRecoveryAllocationAdmission,
    PhysicalRecoveryCoordination, PhysicalRecoveryRejoinResidentAdmissionDenial,
    PhysicalRecoveryRejoinResidentDenial, PhysicalScopedAllocationFailure,
    StoreRecoveryCheckpointBindingBasis,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StoreRecoveryBindingSampleAllocationDenial {
    SizeOverflow,
    BackingMismatch,
    Ownership(PhysicalRecoveryRejoinResidentAdmissionDenial),
    Backing {
        requested: u64,
        cause: PhysicalRecoveryRejoinResidentDenial,
    },
    AllocatorExceededReservation {
        requested: u64,
        actual: u64,
    },
    LocalLimit {
        required: u64,
        admitted: u64,
    },
}

impl StoreRecoveryBindingSampleAllocationDenial {
    pub const fn requested_bytes(&self) -> u64 {
        match self {
            Self::Backing { requested, .. }
            | Self::AllocatorExceededReservation { requested, .. } => *requested,
            Self::LocalLimit { required, .. } => *required,
            Self::SizeOverflow | Self::BackingMismatch | Self::Ownership(_) => 0,
        }
    }
}

use StoreRecoveryBindingSampleAllocationDenial as Denial;

pub(super) struct SamplingAllocation<'coordination> {
    owner: &'coordination PhysicalResidencyOwner,
    original: PhysicalRecoveryAllocationAdmission,
    generation: LifecycleGeneration,
}

impl<'coordination> SamplingAllocation<'coordination> {
    pub(super) fn from_coordination(
        coordination: &'coordination PhysicalRecoveryCoordination,
    ) -> Result<Self, Denial> {
        let (owner, original, generation) = coordination
            .sampling_allocation_basis()
            .map_err(Denial::Ownership)?;
        Ok(Self {
            owner,
            original,
            generation,
        })
    }

    pub(super) fn matches_checkpoint_basis(
        &self,
        basis: &StoreRecoveryCheckpointBindingBasis,
    ) -> bool {
        basis.matches_owner(self.owner)
    }

    pub(super) fn reserve(&self, requested: u64) -> Result<SamplingBacking, Denial> {
        let Some(bytes) = NonZeroU64::new(requested) else {
            return Ok(SamplingBacking::Inline);
        };
        let grant = self
            .owner
            .ports()
            .begin_operation(PhysicalOperationAllocationScope::Recovery, bytes)
            .map_err(|cause| Denial::Backing {
                requested,
                cause: self.map_denial(cause),
            })?;
        Ok(SamplingBacking::Reserved {
            grant,
            generation: self.generation,
        })
    }

    fn map_denial(&self, denial: PhysicalResidencyDenial) -> PhysicalRecoveryRejoinResidentDenial {
        let scope = PhysicalOperationAllocationScope::Recovery;
        if let PhysicalResidencyDenial::Pressure(pressure) = denial {
            if pressure.dimension() == PhysicalResidencyDimension::OperationScope(scope)
                && pressure.limit() == self.original.byte_limit()
            {
                return match pressure.current().checked_add(pressure.requested()) {
                    Some(required) => PhysicalRecoveryRejoinResidentDenial::BudgetExceeded {
                        required,
                        admitted: self.original.byte_limit(),
                    },
                    None => PhysicalRecoveryRejoinResidentDenial::SizeOverflow {
                        admitted: self.original.byte_limit(),
                    },
                };
            }
        }
        PhysicalRecoveryRejoinResidentDenial::OperationAllocation(
            PhysicalScopedAllocationFailure::from_denial(denial, self.generation),
        )
    }
}

#[derive(Debug)]
pub(in crate::physical_runtime::recovery_freshness::binding) enum SamplingBacking {
    Inline,
    Reserved {
        grant: OperationAllocationGrant,
        generation: LifecycleGeneration,
    },
}

impl SamplingBacking {
    pub(super) fn grant(&self) -> Option<&OperationAllocationGrant> {
        match self {
            Self::Inline => None,
            Self::Reserved { grant, .. } => Some(grant),
        }
    }

    pub(in crate::physical_runtime::recovery_freshness::binding) fn bytes(&self) -> u64 {
        self.grant().map_or(0, OperationAllocationGrant::bytes)
    }

    /// Scratch must already be disposed; only result capacities remain live.
    pub(super) fn shrink_to(&mut self, retained: u64) -> Result<(), Denial> {
        match self {
            Self::Inline if retained == 0 => Ok(()),
            Self::Inline => Err(Denial::BackingMismatch),
            Self::Reserved { grant, generation } => {
                if retained > grant.bytes() {
                    return Err(Denial::BackingMismatch);
                }
                grant
                    .try_resize(retained)
                    .map_err(|cause| Denial::Backing {
                        requested: retained,
                        cause: PhysicalRecoveryRejoinResidentDenial::OperationAllocation(
                            PhysicalScopedAllocationFailure::from_denial(cause, *generation),
                        ),
                    })?;
                if retained == 0 {
                    *self = Self::Inline;
                }
                Ok(())
            }
        }
    }
}

impl From<super::super::storage_allocation::BindingStorageAllocationDenial> for Denial {
    fn from(value: super::super::storage_allocation::BindingStorageAllocationDenial) -> Self {
        use super::super::storage_allocation::BindingStorageAllocationDenial as Storage;
        match value {
            Storage::SizeOverflow => Self::SizeOverflow,
            Storage::BackingMismatch => Self::BackingMismatch,
            Storage::Backing { requested, cause } => Self::Backing { requested, cause },
            Storage::AllocatorExceededReservation { requested, actual } => {
                Self::AllocatorExceededReservation { requested, actual }
            }
        }
    }
}
