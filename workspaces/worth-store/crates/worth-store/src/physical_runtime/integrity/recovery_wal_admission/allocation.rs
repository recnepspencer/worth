//! One native Recovery issuer for admitted WAL bytes and shared rosters.

use crate::physical_runtime::{
    instance::PhysicalResidencyOwner, LifecycleGeneration, PhysicalRecoveryAllocationAdmission,
    PhysicalRecoveryCoordination, PhysicalRecoveryRejoinResidentAdmissionDenial,
    PhysicalRecoveryRejoinResidentDenial, PhysicalScopedAllocationFailure,
};
use std::{alloc::Layout, num::NonZeroU64, sync::atomic::AtomicUsize};
use worth_store_buffer_pool::{
    OperationAllocationGrant, PhysicalOperationAllocationScope, PhysicalResidencyDenial,
    PhysicalResidencyDimension,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RecoveryWalAllocationDenial {
    Ownership(PhysicalRecoveryRejoinResidentAdmissionDenial),
    Backing {
        requested: u64,
        cause: PhysicalRecoveryRejoinResidentDenial,
    },
    SizeOverflow,
    AllocatorExceededReservation {
        requested: u64,
        actual: u64,
    },
}

use RecoveryWalAllocationDenial as Denial;

/// Native backing for Runtime's WAL inventory storage. This reservation owns
/// its pool binding independently of Coordination; it exposes no native grant.
#[derive(Debug)]
pub struct PhysicalRecoveryWalInventoryBacking {
    grant: OperationAllocationGrant,
    original: PhysicalRecoveryAllocationAdmission,
    generation: LifecycleGeneration,
}

impl PhysicalRecoveryCoordination {
    /// Reserve before allocating WAL inventory storage, from the same pool
    /// and original Recovery ceiling used by admitted WAL frames and segments.
    pub fn admit_wal_inventory_backing(
        &self,
        bytes: NonZeroU64,
    ) -> Result<PhysicalRecoveryWalInventoryBacking, Denial> {
        let allocation = WalAllocation::from_coordination(self)?;
        Ok(PhysicalRecoveryWalInventoryBacking {
            grant: allocation.reserve(bytes.get())?,
            original: allocation.original,
            generation: allocation.generation,
        })
    }
}

impl PhysicalRecoveryWalInventoryBacking {
    /// Fund the total old-plus-prospective live capacity before allocation.
    /// Smaller requests keep the existing reservation unchanged.
    pub fn grow_total(&mut self, required_total: u64) -> Result<(), Denial> {
        if required_total <= self.charged_bytes() {
            return Ok(());
        }
        self.resize(required_total)
    }

    /// Release capacity only after the corresponding storage has been disposed.
    /// Settlement cannot admit a capacity discovered after allocation.
    pub fn settle_after_disposal(&mut self, retained_bytes: u64) -> Result<(), Denial> {
        let charged = self.charged_bytes();
        if retained_bytes > charged {
            return Err(Denial::AllocatorExceededReservation {
                requested: charged,
                actual: retained_bytes,
            });
        }
        self.resize(retained_bytes)
    }

    pub fn charged_bytes(&self) -> u64 {
        self.grant.bytes()
    }

    fn resize(&mut self, requested: u64) -> Result<(), Denial> {
        self.grant
            .try_resize(requested)
            .map_err(|cause| Denial::Backing {
                requested,
                cause: map_denial(cause, self.original, self.generation),
            })
    }
}

pub(super) struct WalAllocation<'coordination> {
    owner: &'coordination PhysicalResidencyOwner,
    original: PhysicalRecoveryAllocationAdmission,
    generation: LifecycleGeneration,
}

impl<'coordination> WalAllocation<'coordination> {
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

    pub(super) fn reserve(&self, requested: u64) -> Result<OperationAllocationGrant, Denial> {
        let bytes = NonZeroU64::new(requested).ok_or(Denial::SizeOverflow)?;
        self.owner
            .ports()
            .begin_operation(PhysicalOperationAllocationScope::Recovery, bytes)
            .map_err(|cause| Denial::Backing {
                requested,
                cause: map_denial(cause, self.original, self.generation),
            })
    }

    pub(super) fn resize(
        &self,
        grant: &mut OperationAllocationGrant,
        requested: u64,
    ) -> Result<(), Denial> {
        grant
            .try_resize(requested)
            .map_err(|cause| Denial::Backing {
                requested,
                cause: map_denial(cause, self.original, self.generation),
            })
    }

    pub(super) fn owns(&self, grant: &OperationAllocationGrant) -> bool {
        let observation = grant.observation();
        observation.store() == self.owner.ports().store_identity()
            && observation.pool() == self.owner.ports().allocation_events().snapshot().pool()
            && observation.scope() == PhysicalOperationAllocationScope::Recovery
    }
}

fn map_denial(
    denial: PhysicalResidencyDenial,
    original: PhysicalRecoveryAllocationAdmission,
    generation: LifecycleGeneration,
) -> PhysicalRecoveryRejoinResidentDenial {
    if let PhysicalResidencyDenial::Pressure(pressure) = denial {
        if pressure.dimension()
            == PhysicalResidencyDimension::OperationScope(
                PhysicalOperationAllocationScope::Recovery,
            )
            && pressure.limit() == original.byte_limit()
        {
            return match pressure.current().checked_add(pressure.requested()) {
                Some(required) => PhysicalRecoveryRejoinResidentDenial::BudgetExceeded {
                    required,
                    admitted: original.byte_limit(),
                },
                None => PhysicalRecoveryRejoinResidentDenial::SizeOverflow {
                    admitted: original.byte_limit(),
                },
            };
        }
    }
    PhysicalRecoveryRejoinResidentDenial::OperationAllocation(
        PhysicalScopedAllocationFailure::from_denial(denial, generation),
    )
}

pub(super) fn arc_bytes<T>() -> Result<u64, Denial> {
    let (layout, _) = Layout::new::<[AtomicUsize; 2]>()
        .extend(Layout::new::<T>())
        .map_err(|_| Denial::SizeOverflow)?;
    u64::try_from(layout.pad_to_align().size()).map_err(|_| Denial::SizeOverflow)
}

pub(super) fn vector_bytes<T>(capacity: usize) -> Result<u64, Denial> {
    capacity
        .checked_mul(std::mem::size_of::<T>())
        .filter(|bytes| *bytes <= isize::MAX as usize)
        .and_then(|bytes| u64::try_from(bytes).ok())
        .ok_or(Denial::SizeOverflow)
}

pub(super) fn allocator_denial(requested: u64, cause: std::collections::TryReserveError) -> Denial {
    Denial::Backing {
        requested,
        cause: PhysicalRecoveryRejoinResidentDenial::Allocation { requested, cause },
    }
}

#[cfg(test)]
mod tests;
