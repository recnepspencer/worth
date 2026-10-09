//! Certificate bytes and their native Recovery reservation share one lifetime.

use std::{collections::TryReserveError, num::NonZeroU64, sync::Arc};

use worth_store_buffer_pool::{OperationAllocationGrant, PhysicalOperationAllocationScope};
use worth_store_physical_integrity::{
    ValidatedCheckpointStreamAssembly, VerifiedCheckpointFacts, VerifiedCheckpointStream,
    VerifiedCheckpointStreamAssemblyDenial,
};

use crate::physical_runtime::{
    instance::PhysicalResidencyOwner, LifecycleGeneration, PhysicalRecoveryRejoinResidentDenial,
    PhysicalScopedAllocationFailure,
};

use super::{PhysicalRecoveryCoordination, PhysicalRecoveryRejoinResidentAdmissionDenial};

/// Clones retain the same certificate bytes and the same native reservation.
/// The stream is borrowed only; its owning allocation cannot escape this owner.
#[derive(Debug, Clone)]
pub struct SharedRecoveryCheckpoint {
    storage: Arc<RecoveryCheckpointStorage>,
}

#[derive(Debug)]
struct RecoveryCheckpointStorage {
    stream: VerifiedCheckpointStream,
    // Fields dispose in declaration order: bytes precede their reservation.
    grant: OperationAllocationGrant,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SharedCheckpointAdmissionDenial {
    ResidentAdmission(PhysicalRecoveryRejoinResidentAdmissionDenial),
    Allocation(PhysicalRecoveryRejoinResidentDenial),
    Assembly(VerifiedCheckpointStreamAssemblyDenial),
    StoreMismatch,
    SizeOverflow,
    Allocator(TryReserveError),
}

impl PhysicalRecoveryCoordination {
    /// Copies only allocation-free, Integrity-validated certificate records,
    /// after their complete construction peak is reserved in this actual pool.
    pub fn admit_shared_checkpoint(
        &mut self,
        assembly: ValidatedCheckpointStreamAssembly<'_, '_>,
    ) -> Result<SharedRecoveryCheckpoint, SharedCheckpointAdmissionDenial> {
        let store = self.store;
        if assembly.facts().source().identity().store_identity() != store {
            return Err(SharedCheckpointAdmissionDenial::StoreMismatch);
        }
        let (owner, _, generation) = self
            .source_read_allocation_basis()
            .map_err(SharedCheckpointAdmissionDenial::ResidentAdmission)?;
        SharedRecoveryCheckpoint::prepare(owner, generation, assembly)
    }
}

impl SharedRecoveryCheckpoint {
    pub fn stream(&self) -> &VerifiedCheckpointStream {
        &self.storage.stream
    }

    pub fn facts(&self) -> VerifiedCheckpointFacts {
        self.stream().facts()
    }

    /// Actual shared heap backing; each observer refers to this same storage.
    pub fn owned_heap_bytes(&self) -> Option<u64> {
        self.stream()
            .owned_heap_bytes()?
            .checked_add(storage_bytes()?)
    }

    pub(in crate::physical_runtime) fn matches_owner(
        &self,
        owner: &PhysicalResidencyOwner,
    ) -> bool {
        let observation = self.storage.grant.observation();
        observation.scope() == PhysicalOperationAllocationScope::Recovery
            && observation.store() == self.facts().source().identity().store_identity()
            && observation.pool() == owner.ports().allocation_events().snapshot().pool()
    }

    pub(super) fn prepare(
        owner: &PhysicalResidencyOwner,
        generation: LifecycleGeneration,
        assembly: ValidatedCheckpointStreamAssembly<'_, '_>,
    ) -> Result<Self, SharedCheckpointAdmissionDenial> {
        use SharedCheckpointAdmissionDenial as Denial;
        let records = assembly.certificates();
        let payload = records
            .iter()
            .try_fold(0_u64, |bytes, (_, record)| {
                bytes.checked_add(u64::try_from(record.len()).ok()?)
            })
            .ok_or(Denial::SizeOverflow)?;
        let roster = u64::try_from(records.len())
            .ok()
            .and_then(|count| count.checked_mul(std::mem::size_of::<Box<[u8]>>() as u64))
            .ok_or(Denial::SizeOverflow)?;
        let retained = payload
            .checked_add(roster)
            .and_then(|bytes| bytes.checked_add(storage_bytes()?))
            .ok_or(Denial::SizeOverflow)?;
        // Vec-to-Box conversion may overlap old and prospective payload/roster
        // storage. Reserve that peak before the first try_reserve_exact.
        let peak = retained
            .checked_add(payload)
            .and_then(|bytes| bytes.checked_add(roster))
            .ok_or(Denial::SizeOverflow)?;
        let map_native = |denial| {
            Denial::Allocation(PhysicalRecoveryRejoinResidentDenial::OperationAllocation(
                PhysicalScopedAllocationFailure::from_denial(denial, generation),
            ))
        };
        let mut grant = owner
            .ports()
            .begin_operation(
                PhysicalOperationAllocationScope::Recovery,
                NonZeroU64::new(peak).ok_or(Denial::SizeOverflow)?,
            )
            .map_err(map_native)?;
        let mut certificates = Vec::new();
        certificates
            .try_reserve_exact(records.len())
            .map_err(Denial::Allocator)?;
        for (_, bytes) in records {
            let mut copy = Vec::new();
            copy.try_reserve_exact(bytes.len())
                .map_err(Denial::Allocator)?;
            copy.extend_from_slice(bytes);
            certificates.push(copy.into_boxed_slice());
        }
        let stream = assembly
            .retain_prepared_certificates(certificates.into_boxed_slice())
            .map_err(Denial::Assembly)?;
        // No intermediate Vec backing remains; Arc storage was pre-funded.
        grant.try_resize(retained).map_err(map_native)?;
        Ok(Self {
            storage: Arc::new(RecoveryCheckpointStorage { stream, grant }),
        })
    }
}

fn storage_bytes() -> Option<u64> {
    u64::try_from(std::mem::size_of::<RecoveryCheckpointStorage>())
        .ok()?
        .checked_add(2 * std::mem::size_of::<usize>() as u64)
}

#[cfg(test)]
mod tests;
