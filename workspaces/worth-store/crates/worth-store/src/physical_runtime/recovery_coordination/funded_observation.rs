//! An actual C4 read buffer retains its own native Recovery reservation.

use worth_store_buffer_pool::OperationAllocationGrant;
use worth_store_physical_backend::{
    ArtifactTreeListingAllocationBoundary, ArtifactTreePathAllocationBoundary,
    BorrowedRecordFilesystemObservation, BoundedRecoveryFilesystemDiscovery,
    ObservedRecoveryArtifact, RecoveryDiscoveryAllocationFailure, RecoveryDiscoveryArtifact,
};
use worth_store_physical_format::RecordArtifactFile;

use super::PhysicalRecoveryReadAllocation;
use crate::physical_runtime::PhysicalRecoveryRejoinResidentDenial;

mod storage;

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
    pub(in crate::physical_runtime) fn read_serving_record(
        &mut self,
        observation: &mut BorrowedRecordFilesystemObservation<'_>,
        address: RecordArtifactFile,
        byte_limit: u64,
    ) -> Result<
        FundedRecoveryObservation,
        RecoveryDiscoveryAllocationFailure<PhysicalRecoveryObservationAllocationDenial>,
    > {
        self.read_serving_observation(observation, address, 0, None, byte_limit)
    }

    pub(in crate::physical_runtime) fn read_serving_record_range(
        &mut self,
        observation: &mut BorrowedRecordFilesystemObservation<'_>,
        address: RecordArtifactFile,
        offset: u64,
        length: u32,
        byte_limit: u64,
    ) -> Result<
        FundedRecoveryObservation,
        RecoveryDiscoveryAllocationFailure<PhysicalRecoveryObservationAllocationDenial>,
    > {
        self.read_serving_observation(observation, address, offset, Some(length), byte_limit)
    }

    fn read_serving_observation(
        &mut self,
        observation: &mut BorrowedRecordFilesystemObservation<'_>,
        address: RecordArtifactFile,
        offset: u64,
        length: Option<u32>,
        byte_limit: u64,
    ) -> Result<
        FundedRecoveryObservation,
        RecoveryDiscoveryAllocationFailure<PhysicalRecoveryObservationAllocationDenial>,
    > {
        use PhysicalRecoveryObservationAllocationDenial as Denial;
        let failure = |cause| RecoveryDiscoveryAllocationFailure::Allocation {
            artifact: RecoveryDiscoveryArtifact::Record(address),
            offset,
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
        let observed = match length {
            Some(length) => observation.read_record_artifact_range_with_storage(
                address,
                offset,
                length,
                byte_limit,
                &mut storage,
            ),
            None => {
                observation.read_record_artifact_with_storage(address, byte_limit, &mut storage)
            }
        }?;
        Ok(storage.finish(observed))
    }

    pub fn read_checkpoint(
        &mut self,
        discovery: &mut BoundedRecoveryFilesystemDiscovery,
        byte_limit: u64,
    ) -> Result<
        FundedRecoveryObservation,
        RecoveryDiscoveryAllocationFailure<PhysicalRecoveryObservationAllocationDenial>,
    > {
        self.read_observation(discovery, byte_limit, SourceRead::Checkpoint)
    }

    pub fn read_checkpoint_source_root(
        &mut self,
        discovery: &mut BoundedRecoveryFilesystemDiscovery,
        generation: u64,
        byte_limit: u64,
    ) -> Result<
        FundedRecoveryObservation,
        RecoveryDiscoveryAllocationFailure<PhysicalRecoveryObservationAllocationDenial>,
    > {
        self.read_observation(
            discovery,
            byte_limit,
            SourceRead::Record(RecordArtifactFile::RootManifest { generation }),
        )
    }

    fn read_observation(
        &mut self,
        discovery: &mut BoundedRecoveryFilesystemDiscovery,
        byte_limit: u64,
        source: SourceRead,
    ) -> Result<
        FundedRecoveryObservation,
        RecoveryDiscoveryAllocationFailure<PhysicalRecoveryObservationAllocationDenial>,
    > {
        use PhysicalRecoveryObservationAllocationDenial as Denial;
        if discovery.store_identity() != self.owner.ports().store_identity() {
            return Err(RecoveryDiscoveryAllocationFailure::Allocation {
                artifact: source.artifact(),
                offset: 0,
                requested: 0,
                cause: Denial::StoreMismatch,
            });
        }
        let mut storage = storage::NativeObservationStorage::new(self);
        let observed = match source {
            SourceRead::Checkpoint => {
                discovery.read_current_checkpoint_with_storage(byte_limit, &mut storage)
            }
            SourceRead::Record(artifact) => {
                discovery.read_record_artifact_with_storage(artifact, byte_limit, &mut storage)
            }
        }?;
        Ok(storage.finish(observed))
    }
}

enum SourceRead {
    Checkpoint,
    Record(RecordArtifactFile),
}

impl SourceRead {
    fn artifact(&self) -> RecoveryDiscoveryArtifact {
        match *self {
            Self::Checkpoint => RecoveryDiscoveryArtifact::CurrentCheckpoint,
            Self::Record(artifact) => RecoveryDiscoveryArtifact::Record(artifact),
        }
    }
}

#[cfg(test)]
mod tests;

#[cfg(test)]
#[path = "funded_observation/reread_tests.rs"]
mod reread_tests;
