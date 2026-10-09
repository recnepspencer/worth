//! An actual C4 read buffer retains its own native Recovery reservation.

use worth_foundational::LimitDimension;
use worth_proof::{DenialTransitionOutcome, TransitionOutcome};
use worth_store_buffer_pool::OperationAllocationGrant;
use worth_store_physical_backend::{
    AllocatedReadFailure, ArtifactCeiling, ArtifactTreeListingAllocationBoundary,
    ArtifactTreePathAllocationBoundary, BorrowedRecordFilesystemObservation,
    BoundedRecoveryFilesystemDiscovery, ObservedRecoveryArtifact, PageAddress, ReadGrant,
    ReadRefusal, RecoveryDiscoveryAllocationFailure, RecoveryDiscoveryArtifact, StreamArtifact,
    UnchargedRead,
};
use worth_store_physical_format::{PhysicalRecordFormatDeclaration, RecordArtifactFile};

use super::PhysicalRecoveryReadAllocation;
use crate::physical_runtime::PhysicalRecoveryRejoinResidentDenial;

mod storage;

/// A funded whole read: the observation it funded, a budget's refusal, or what
/// no budget fixes.
pub type FundedReadOutcome<D> = DenialTransitionOutcome<
    FundedRecoveryObservation,
    ReadRefusal<D>,
    AllocatedReadFailure<PhysicalRecoveryObservationAllocationDenial>,
>;

#[derive(Debug)]
pub struct FundedRecoveryObservation {
    // The bytes are disposed before their independently owned reservation.
    observed: ObservedRecoveryArtifact,
    backing: ObservationBacking,
}

#[derive(Debug)]
enum ObservationBacking {
    Inline,
    Reserved(OperationAllocationGrant),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PhysicalRecoveryObservationAllocationDenial {
    StoreMismatch,
    WalObservationUnavailable(worth_store_physical_backend::RecoveryFilesystemQualificationError),
    Residency(PhysicalRecoveryRejoinResidentDenial),
    ListingResidency {
        boundary: ArtifactTreeListingAllocationBoundary,
        cause: PhysicalRecoveryRejoinResidentDenial,
    },
    UnqualifiedListingStorage,
    PathResidency {
        boundary: ArtifactTreePathAllocationBoundary,
        cause: PhysicalRecoveryRejoinResidentDenial,
    },
    UnqualifiedPathStorage,
    AllocatorExceededReservation {
        requested: u64,
        actual: u64,
    },
}

impl FundedRecoveryObservation {
    pub fn observed(&self) -> &ObservedRecoveryArtifact {
        &self.observed
    }

    /// Only authenticated absence can leave the owning allocation boundary.
    pub fn into_absent(self) -> Result<ObservedRecoveryArtifact, Self> {
        if self.observed.bytes().is_some() {
            return Err(self);
        }
        Ok(self.observed)
    }

    pub fn owned_heap_bytes(&self) -> Option<u64> {
        self.observed.owned_heap_bytes()
    }

    pub fn charged_bytes(&self) -> u64 {
        match &self.backing {
            ObservationBacking::Inline => 0,
            ObservationBacking::Reserved(grant) => grant.bytes(),
        }
    }
}

impl PhysicalRecoveryReadAllocation<'_> {
    /// Re-reads the checkpoint stream its selected claim declares `declared`
    /// bytes long: a longer stream is damage with its real length.
    pub(in crate::physical_runtime) fn read_serving_checkpoint(
        &mut self,
        observation: &mut BorrowedRecordFilesystemObservation<'_>,
        declared: u64,
    ) -> Result<
        FundedRecoveryObservation,
        RecoveryDiscoveryAllocationFailure<PhysicalRecoveryObservationAllocationDenial>,
    > {
        let ceiling = ArtifactCeiling::declared(StreamArtifact::CurrentCheckpoint, declared);
        self.read_serving_record(observation, ceiling)
    }

    /// Re-reads one whole artifact under its ceiling.
    pub(in crate::physical_runtime) fn read_serving_record(
        &mut self,
        observation: &mut BorrowedRecordFilesystemObservation<'_>,
        ceiling: ArtifactCeiling,
    ) -> Result<
        FundedRecoveryObservation,
        RecoveryDiscoveryAllocationFailure<PhysicalRecoveryObservationAllocationDenial>,
    > {
        let context = ceiling.artifact();
        self.read_serving_observation(observation, context, |observation, storage| {
            observation
                .read_with_storage(ceiling, ReadGrant::ceiling_only(), storage)
                .observed()
        })
    }

    pub(in crate::physical_runtime) fn read_serving_record_range(
        &mut self,
        observation: &mut BorrowedRecordFilesystemObservation<'_>,
        address: RecordArtifactFile,
        offset: u64,
        length: u32,
    ) -> Result<
        FundedRecoveryObservation,
        RecoveryDiscoveryAllocationFailure<PhysicalRecoveryObservationAllocationDenial>,
    > {
        let context = RecoveryDiscoveryArtifact::Record(address);
        self.read_serving_observation(observation, context, |observation, storage| {
            observation
                .read_record_artifact_range_with_storage(
                    address,
                    offset,
                    length,
                    ReadGrant::ceiling_only(),
                    storage,
                )
                .observed()
        })
    }

    fn read_serving_observation(
        &mut self,
        observation: &mut BorrowedRecordFilesystemObservation<'_>,
        context: RecoveryDiscoveryArtifact,
        read: impl FnOnce(
            &mut BorrowedRecordFilesystemObservation<'_>,
            &mut storage::NativeObservationStorage<'_, '_>,
        ) -> Result<
            ObservedRecoveryArtifact,
            RecoveryDiscoveryAllocationFailure<PhysicalRecoveryObservationAllocationDenial>,
        >,
    ) -> Result<
        FundedRecoveryObservation,
        RecoveryDiscoveryAllocationFailure<PhysicalRecoveryObservationAllocationDenial>,
    > {
        use PhysicalRecoveryObservationAllocationDenial as Denial;
        let failure = |cause| RecoveryDiscoveryAllocationFailure::Allocation {
            artifact: context,
            offset: 0,
            requested: 0,
            cause,
        };
        if observation.store_identity() != self.store_identity() {
            return Err(failure(Denial::StoreMismatch));
        }
        if !observation.path_storage_is_qualified() {
            return Err(failure(Denial::UnqualifiedPathStorage));
        }
        let mut storage = storage::NativeObservationStorage::new(self);
        let observed = read(observation, &mut storage)?;
        Ok(storage.finish(observed))
    }

    /// The checkpoint stream, whose length no fact declares, under `grant`.
    pub fn read_checkpoint<D: LimitDimension>(
        &mut self,
        discovery: &mut BoundedRecoveryFilesystemDiscovery,
        grant: ReadGrant<D>,
    ) -> FundedReadOutcome<D> {
        self.read_whole(discovery, checkpoint_stream(), grant)
    }

    /// The checkpoint's source root: one page of `format`, under `grant`.
    pub fn read_checkpoint_source_root<D: LimitDimension>(
        &mut self,
        discovery: &mut BoundedRecoveryFilesystemDiscovery,
        format: PhysicalRecordFormatDeclaration,
        generation: u64,
        grant: ReadGrant<D>,
    ) -> FundedReadOutcome<D> {
        let ceiling = ArtifactCeiling::page(format, PageAddress::RootManifest { generation });
        self.read_whole(discovery, ceiling, grant)
    }

    pub(in crate::physical_runtime) fn read_record(
        &mut self,
        discovery: &mut BoundedRecoveryFilesystemDiscovery,
        ceiling: ArtifactCeiling,
    ) -> Result<
        FundedRecoveryObservation,
        RecoveryDiscoveryAllocationFailure<PhysicalRecoveryObservationAllocationDenial>,
    > {
        self.read_whole(discovery, ceiling, ReadGrant::ceiling_only())
            .observed()
    }

    fn read_whole<D: LimitDimension>(
        &mut self,
        discovery: &mut BoundedRecoveryFilesystemDiscovery,
        ceiling: ArtifactCeiling,
        grant: ReadGrant<D>,
    ) -> FundedReadOutcome<D> {
        if discovery.store_identity() != self.owner.ports().store_identity() {
            return TransitionOutcome::Failed(AllocatedReadFailure::Allocation {
                artifact: ceiling.artifact(),
                offset: 0,
                requested: 0,
                cause: PhysicalRecoveryObservationAllocationDenial::StoreMismatch,
            });
        }
        let mut storage = storage::NativeObservationStorage::new(self);
        discovery
            .read_with_storage(ceiling, grant, &mut storage)
            .map_success(|observed| storage.finish(observed))
    }
}

/// The checkpoint stream: nothing declares its length.
const fn checkpoint_stream() -> ArtifactCeiling {
    ArtifactCeiling::undeclared(StreamArtifact::CurrentCheckpoint)
}

/// The checkpoint stream read whole and uncharged, as a test observes it.
#[cfg(test)]
pub(in crate::physical_runtime) fn observe_checkpoint_for_test(
    discovery: &mut BoundedRecoveryFilesystemDiscovery,
) -> Result<ObservedRecoveryArtifact, worth_store_physical_backend::RecoveryDiscoveryFailure> {
    discovery
        .read(checkpoint_stream(), ReadGrant::ceiling_only())
        .observed()
}

#[cfg(test)]
mod tests;

#[cfg(test)]
#[path = "funded_observation/reread_tests.rs"]
mod reread_tests;
