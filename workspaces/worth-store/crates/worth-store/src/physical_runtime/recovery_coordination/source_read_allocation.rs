//! Native transient backing for one selector/root source observation.
//! This does not certify or retain reconstructed C8 custody.

use std::num::NonZeroU64;

use worth_store_buffer_pool::{
    OperationAllocationGrant, PhysicalOperationAllocationScope, PhysicalResidencyDenial,
    PhysicalResidencyDimension,
};

use crate::physical_runtime::{
    instance::PhysicalResidencyOwner, LifecycleGeneration, PhysicalRecoveryAllocationAdmission,
    PhysicalRecoveryRejoinResidentDenial, PhysicalScopedAllocationFailure,
};

use super::{PhysicalRecoveryCoordination, PhysicalRecoveryRejoinResidentAdmissionDenial};

/// Exclusive Coordination-borrowed backing for a transient source read.
///
/// Reserve the simultaneous selector/root buffers and integrity scratch before
/// allocating them. Drop those buffers before releasing this reservation.
/// The borrow prevents consuming Coordination while this charge remains live.
pub struct PhysicalRecoveryReadAllocation<'coordination> {
    grant: Option<OperationAllocationGrant>,
    pub(super) owner: &'coordination PhysicalResidencyOwner,
    original: PhysicalRecoveryAllocationAdmission,
    generation: LifecycleGeneration,
}

impl PhysicalRecoveryCoordination {
    /// Starts a zero-charge reservation in this Coordination owner's pool.
    pub fn begin_source_read_allocation(
        &mut self,
    ) -> Result<PhysicalRecoveryReadAllocation<'_>, PhysicalRecoveryRejoinResidentAdmissionDenial>
    {
        let (owner, original, generation) = self.source_read_allocation_basis()?;
        Ok(PhysicalRecoveryReadAllocation::new(
            owner, original, generation,
        ))
    }
}

impl<'coordination> PhysicalRecoveryReadAllocation<'coordination> {
    /// Independent rejoin borrows the same owner without blocking C9 admission.
    /// Every returned buffer still owns its native charge; no ceiling is minted.
    pub(in crate::physical_runtime) fn for_coordination(
        coordination: &'coordination PhysicalRecoveryCoordination,
    ) -> Result<Self, PhysicalRecoveryRejoinResidentAdmissionDenial> {
        let (owner, original, generation) = coordination.sampling_allocation_basis()?;
        Ok(Self::new(owner, original, generation))
    }

    /// Serving uses the actual moved recovery owner and its carried ceiling.
    /// The public Coordination entry remains exclusively borrowed.
    pub(in crate::physical_runtime) fn for_serving(
        owner: &'coordination PhysicalResidencyOwner,
        generation: LifecycleGeneration,
    ) -> Self {
        Self::new(owner, owner.recovery_allocation_admission(), generation)
    }

    fn new(
        owner: &'coordination PhysicalResidencyOwner,
        original: PhysicalRecoveryAllocationAdmission,
        generation: LifecycleGeneration,
    ) -> Self {
        Self {
            grant: None,
            owner,
            original,
            generation,
        }
    }

    /// Funds the required total simultaneous backing. Smaller requests retain
    /// the current charge; this surface cannot shrink or expose a native grant.
    pub fn reserve_total(
        &mut self,
        required_total: u64,
    ) -> Result<(), PhysicalRecoveryRejoinResidentDenial> {
        let current = self.charged_bytes();
        if required_total <= current {
            return Ok(());
        }
        let scope = PhysicalOperationAllocationScope::Recovery;
        let result = match &mut self.grant {
            Some(grant) => grant.try_resize(required_total),
            None => self
                .owner
                .ports()
                .begin_operation(
                    scope,
                    NonZeroU64::new(required_total).expect("growth from zero is positive"),
                )
                .map(|grant| self.grant = Some(grant)),
        };
        result.map_err(|denial| self.map_allocation_denial(denial))
    }

    pub(in crate::physical_runtime) fn reserve_owned(
        &self,
        required: u64,
    ) -> Result<Option<OperationAllocationGrant>, PhysicalRecoveryRejoinResidentDenial> {
        let Some(bytes) = NonZeroU64::new(required) else {
            return Ok(None);
        };
        self.owner
            .ports()
            .begin_operation(PhysicalOperationAllocationScope::Recovery, bytes)
            .map(Some)
            .map_err(|denial| self.map_allocation_denial(denial))
    }

    pub(in crate::physical_runtime) fn map_allocation_denial(
        &self,
        denial: PhysicalResidencyDenial,
    ) -> PhysicalRecoveryRejoinResidentDenial {
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

    /// Retains validated certificate records while all source observations are
    /// still funded; the native pool accounts their construction overlap.
    pub fn admit_shared_checkpoint(
        &mut self,
        assembly: worth_store_physical_integrity::ValidatedCheckpointStreamAssembly<'_, '_>,
    ) -> Result<super::SharedRecoveryCheckpoint, super::SharedCheckpointAdmissionDenial> {
        if assembly.facts().source().identity().store_identity() != self.original.store_identity() {
            return Err(super::SharedCheckpointAdmissionDenial::StoreMismatch);
        }
        super::SharedRecoveryCheckpoint::prepare(self.owner, self.generation, assembly)
    }

    /// Exact currently reserved native Recovery bytes.
    pub fn charged_bytes(&self) -> u64 {
        self.grant
            .as_ref()
            .map_or(0, OperationAllocationGrant::bytes)
    }

    pub(in crate::physical_runtime) fn recovery_byte_limit(&self) -> u64 {
        self.original.byte_limit()
    }

    pub(in crate::physical_runtime) fn store_identity(
        &self,
    ) -> worth_store_physical_format::store_namespace::StableStoreIdentity {
        self.owner.ports().store_identity()
    }

    pub(in crate::physical_runtime) fn pool_identity(
        &self,
    ) -> worth_store_buffer_pool::PhysicalResidencyIncarnation {
        self.owner.ports().allocation_events().snapshot().pool()
    }

    pub(in crate::physical_runtime) fn generation(&self) -> LifecycleGeneration {
        self.generation
    }

    /// The generation of the registered recovery issuer, carried by the actual
    /// pool owner. A new Serving runtime has a different observation lifecycle.
    pub(in crate::physical_runtime) fn recovery_origin_generation(
        &self,
    ) -> Option<LifecycleGeneration> {
        self.owner.recovery_origin_generation()
    }

    pub(in crate::physical_runtime) fn owns_checkpoint(
        &self,
        checkpoint: &super::SharedRecoveryCheckpoint,
    ) -> bool {
        checkpoint.matches_owner(self.owner)
    }

    /// Prepares online binding interpretation only from this pool's C9-admitted
    /// retained checkpoint. All evidence and lookup storage is pre-funded.
    pub fn begin_checkpoint_binding_rebuild(
        &mut self,
        checkpoint: &super::SharedRecoveryCheckpoint,
        maximum_operations: u64,
    ) -> Result<
        crate::physical_runtime::StoreRecoveryCheckpointBindingRebuilder,
        crate::physical_runtime::StoreRecoveryCheckpointBindingAllocationDenial,
    > {
        crate::physical_runtime::StoreRecoveryCheckpointBindingRebuilder::prepare(
            self,
            checkpoint,
            maximum_operations,
        )
    }
}

#[cfg(test)]
mod tests;
