//! Addressed record reads borrowed from one qualified live media owner.
//! This read-only surface cannot finish, promote, or extract that owner.

use super::media_backing::BorrowedMediaBacking;
use super::{
    DiscoveryMediaBacking, FilesystemObservation, ObservedRecoveryArtifact,
    RecoveryDiscoveryAllocationFailure, RecoveryDiscoveryCounters,
    RecoveryFilesystemQualificationError,
};
use crate::filesystem_media::{ArtifactTreeReadAllocator, QualifiedFilesystemMedia};
use worth_store_physical_format::RecordArtifactFile;

pub struct BorrowedRecordFilesystemObservation<'media> {
    observation: FilesystemObservation<BorrowedMediaBacking<'media>>,
}

impl QualifiedFilesystemMedia {
    /// Inherits this owner's confined read capability without creating an
    /// admitted recovery session or any staging/publication capability.
    pub fn bounded_record_observation(
        &self,
        maximum_entries: u64,
        maximum_bytes: u64,
    ) -> Result<BorrowedRecordFilesystemObservation<'_>, RecoveryFilesystemQualificationError> {
        if maximum_entries == 0 || maximum_bytes == 0 {
            return Err(RecoveryFilesystemQualificationError::InvalidDiscoveryLimit);
        }
        Ok(BorrowedRecordFilesystemObservation {
            observation: FilesystemObservation {
                parts: BorrowedMediaBacking { media: self },
                remaining_entries: maximum_entries,
                remaining_bytes: maximum_bytes,
                maximum_bytes,
                // Record evidence has no WAL observation identity to issue.
                discovery_incarnation: 0,
                wal_observations_issued: 0,
                counters: RecoveryDiscoveryCounters::default(),
            },
        })
    }
}

impl BorrowedRecordFilesystemObservation<'_> {
    pub fn store_identity(
        &self,
    ) -> worth_store_physical_format::store_namespace::StableStoreIdentity {
        self.observation.parts.store_identity()
    }

    pub fn path_storage_is_qualified(&self) -> bool {
        self.observation
            .parts
            .artifact_tree()
            .path_storage_is_qualified()
    }

    pub fn counters(&self) -> RecoveryDiscoveryCounters {
        self.observation.counters
    }

    /// Rechecks the live checkpoint through the same confined, allocated
    /// artifact reader. This observation cannot select or certify a checkpoint.
    pub fn read_current_checkpoint_with_storage<S: ArtifactTreeReadAllocator>(
        &mut self,
        byte_limit: u64,
        storage: &mut S,
    ) -> Result<ObservedRecoveryArtifact, RecoveryDiscoveryAllocationFailure<S::Denial>> {
        self.observation
            .read_current_checkpoint_with_storage(byte_limit, storage)
    }

    pub fn read_record_artifact_with_storage<S: ArtifactTreeReadAllocator>(
        &mut self,
        address: RecordArtifactFile,
        byte_limit: u64,
        storage: &mut S,
    ) -> Result<ObservedRecoveryArtifact, RecoveryDiscoveryAllocationFailure<S::Denial>> {
        self.observation
            .read_record_artifact_with_storage(address, byte_limit, storage)
    }

    pub fn read_record_artifact_range_with_storage<S: ArtifactTreeReadAllocator>(
        &mut self,
        address: RecordArtifactFile,
        offset: u64,
        length: u32,
        byte_limit: u64,
        storage: &mut S,
    ) -> Result<ObservedRecoveryArtifact, RecoveryDiscoveryAllocationFailure<S::Denial>> {
        self.observation
            .read_record_artifact_range_with_storage(address, offset, length, byte_limit, storage)
    }
}

#[cfg(test)]
mod tests;
