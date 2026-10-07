//! Native ownership of the current walk's actual backing and prepared peak.

use super::Denial;
use crate::physical_runtime::{LifecycleGeneration, PhysicalRecoveryReadAllocation};
use worth_store_buffer_pool::{OperationAllocationGrant, PhysicalResidencyIncarnation};
use worth_store_physical_format::store_namespace::StableStoreIdentity;

pub(super) struct HeadWalkBacking {
    grant: Option<OperationAllocationGrant>,
    store: StableStoreIdentity,
    pool: PhysicalResidencyIncarnation,
    origin_generation: LifecycleGeneration,
    used: u64,
}

impl HeadWalkBacking {
    pub(super) fn new(window: &PhysicalRecoveryReadAllocation<'_>) -> Result<Self, Denial> {
        let origin_generation = window.recovery_origin_generation().ok_or(Denial::Resident(
            crate::physical_runtime::PhysicalRecoveryRejoinResidentDenial::MissingResidentAdmission,
        ))?;
        Ok(Self {
            grant: None,
            store: window.store_identity(),
            pool: window.pool_identity(),
            origin_generation,
            used: 0,
        })
    }

    pub(super) fn matching_owner(&self, window: &PhysicalRecoveryReadAllocation<'_>) -> bool {
        self.store == window.store_identity()
            && self.pool == window.pool_identity()
            && Some(self.origin_generation) == window.recovery_origin_generation()
    }

    pub(super) fn prepare(
        &mut self,
        window: &PhysicalRecoveryReadAllocation<'_>,
        required: u64,
    ) -> Result<(), Denial> {
        if !self.matching_owner(window) {
            return Err(Denial::RootBinding);
        }
        if required <= self.charged_bytes() {
            return Ok(());
        }
        match &mut self.grant {
            Some(grant) => grant
                .try_resize(required)
                .map_err(|cause| Denial::Resident(window.map_allocation_denial(cause))),
            None => {
                self.grant = window.reserve_owned(required).map_err(Denial::Resident)?;
                Ok(())
            }
        }
    }

    pub(super) fn retain(&mut self, bytes: u64) -> Result<(), Denial> {
        let next = self.used.checked_add(bytes).ok_or(Denial::BoundExceeded)?;
        if next > self.charged_bytes() {
            return Err(Denial::BoundExceeded);
        }
        self.used = next;
        Ok(())
    }

    pub(super) fn used(&self) -> u64 {
        self.used
    }

    /// The caller has already disposed all storage above this actual capacity.
    pub(super) fn settle(&mut self, retained: u64) -> Result<(), Denial> {
        if retained > self.used || retained > self.charged_bytes() {
            return Err(Denial::BoundExceeded);
        }
        if let Some(grant) = &mut self.grant {
            grant.try_resize(retained).map_err(|cause| {
                Denial::Resident(crate::physical_runtime::PhysicalRecoveryRejoinResidentDenial::OperationAllocation(
                    crate::physical_runtime::PhysicalScopedAllocationFailure::from_denial(cause, self.origin_generation)
                ))
            })?;
        }
        self.used = retained;
        Ok(())
    }

    pub(super) fn charged_bytes(&self) -> u64 {
        self.grant
            .as_ref()
            .map_or(0, OperationAllocationGrant::bytes)
    }
}

pub(super) fn vector_bytes<T>(values: &Vec<T>) -> Result<u64, Denial> {
    slot_bytes::<T>(values.capacity())
}

pub(super) fn slot_bytes<T>(count: usize) -> Result<u64, Denial> {
    count
        .checked_mul(std::mem::size_of::<T>())
        .filter(|bytes| *bytes <= isize::MAX as usize)
        .and_then(|bytes| u64::try_from(bytes).ok())
        .ok_or(Denial::BoundExceeded)
}
