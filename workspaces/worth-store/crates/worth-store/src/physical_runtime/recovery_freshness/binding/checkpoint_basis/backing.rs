//! Native storage backing and authentic zero-allocation pool provenance.

use worth_store_buffer_pool::{OperationAllocationGrant, PhysicalResidencyIncarnation};

use crate::physical_runtime::{
    instance::PhysicalResidencyOwner, LifecycleGeneration, PhysicalRecoveryReadAllocation,
    PhysicalRecoveryRejoinResidentDenial, PhysicalScopedAllocationFailure,
};

use super::StoreRecoveryCheckpointBindingAllocationDenial as Denial;

pub(super) enum CheckpointBindingBacking {
    Inline(PhysicalResidencyIncarnation),
    Reserved {
        grant: OperationAllocationGrant,
        generation: LifecycleGeneration,
    },
}

impl CheckpointBindingBacking {
    pub(super) fn admit(
        window: &PhysicalRecoveryReadAllocation<'_>,
        requested: u64,
    ) -> Result<Self, Denial> {
        match window
            .reserve_owned(requested)
            .map_err(|cause| Denial::Backing { requested, cause })?
        {
            Some(grant) => Ok(Self::Reserved {
                grant,
                generation: window.generation(),
            }),
            None => Ok(Self::Inline(window.pool_identity())),
        }
    }

    pub(super) fn grant(&self) -> Option<&OperationAllocationGrant> {
        match self {
            Self::Reserved { grant, .. } => Some(grant),
            Self::Inline(_) => None,
        }
    }

    pub(super) fn charged_bytes(&self) -> u64 {
        self.grant().map_or(0, OperationAllocationGrant::bytes)
    }

    fn pool(&self) -> PhysicalResidencyIncarnation {
        match self {
            Self::Inline(pool) => *pool,
            Self::Reserved { grant, .. } => grant.observation().pool(),
        }
    }

    pub(super) fn matches_window(&self, window: &PhysicalRecoveryReadAllocation<'_>) -> bool {
        self.pool() == window.pool_identity()
    }

    pub(super) fn matches_owner(&self, owner: &PhysicalResidencyOwner) -> bool {
        self.pool() == owner.ports().allocation_events().snapshot().pool()
    }

    /// The caller must first dispose lookup storage (or all failed evidence).
    /// Only retained Vec capacity remains when this reservation is reduced.
    pub(super) fn retain(&mut self, bytes: u64) -> Result<(), Denial> {
        match self {
            Self::Inline(_) if bytes == 0 => Ok(()),
            Self::Inline(_) => Err(Denial::BackingMismatch),
            Self::Reserved { grant, generation } => {
                let pool = grant.observation().pool();
                if bytes > grant.bytes() {
                    return Err(Denial::BackingMismatch);
                }
                grant.try_resize(bytes).map_err(|cause| Denial::Backing {
                    requested: bytes,
                    cause: PhysicalRecoveryRejoinResidentDenial::OperationAllocation(
                        PhysicalScopedAllocationFailure::from_denial(cause, *generation),
                    ),
                })?;
                if bytes == 0 {
                    *self = Self::Inline(pool);
                }
                Ok(())
            }
        }
    }
}
