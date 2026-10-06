//! WAL-only observation of an already qualified live owner. No owned finish exists.

use super::media_backing::{BorrowedMediaBacking, DiscoveryMediaBacking};
use super::{
    FilesystemObservation, ObservedWalArtifact, RecoveryDiscoveryAllocationFailure,
    RecoveryDiscoveryCounters, RecoveryFilesystemQualificationError, RecoveryWalReadStorage,
};
use crate::filesystem_media::QualifiedFilesystemMedia;
use std::num::NonZeroU64;
use std::sync::atomic::Ordering;

pub struct BorrowedWalFilesystemObservation<'media> {
    discovery: FilesystemObservation<BorrowedMediaBacking<'media>>,
}

impl QualifiedFilesystemMedia {
    /// Borrows this qualified owner's read capability only. It cannot produce
    /// admitted recovery media, staging/publication authority, or another owner.
    pub fn bounded_wal_observation(
        &self,
        maximum_entries: u64,
        maximum_bytes: u64,
    ) -> Result<BorrowedWalFilesystemObservation<'_>, RecoveryFilesystemQualificationError> {
        if maximum_entries == 0 {
            return Err(RecoveryFilesystemQualificationError::InvalidDiscoveryLimit);
        }
        let incarnation = self
            .wal_observation_incarnations
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |prior| {
                prior.checked_add(1)
            })
            .map_err(|_| RecoveryFilesystemQualificationError::InvalidDiscoveryLimit)?
            .checked_add(1)
            .expect("checked owner-issued incarnation");
        Ok(BorrowedWalFilesystemObservation {
            discovery: FilesystemObservation {
                parts: BorrowedMediaBacking { media: self },
                remaining_entries: maximum_entries,
                maximum_entries,
                remaining_bytes: maximum_bytes,
                maximum_bytes,
                discovery_incarnation: incarnation,
                wal_observations_issued: 0,
                counters: RecoveryDiscoveryCounters::default(),
            },
        })
    }
}

impl BorrowedWalFilesystemObservation<'_> {
    pub fn store_identity(
        &self,
    ) -> worth_store_physical_format::store_namespace::StableStoreIdentity {
        self.discovery.parts.store_identity()
    }
    pub fn wal_listing_storage_is_qualified(&self) -> bool {
        self.discovery.wal_listing_storage_is_qualified()
    }
    pub fn wal_path_storage_is_qualified(&self) -> bool {
        self.discovery.wal_path_storage_is_qualified()
    }
    pub fn read_wal_artifacts_with_storage<S: RecoveryWalReadStorage>(
        &mut self,
        maximum_segments: NonZeroU64,
        byte_limit: u64,
        storage: &mut S,
    ) -> Result<Vec<ObservedWalArtifact>, RecoveryDiscoveryAllocationFailure<S::Denial>> {
        self.discovery
            .read_wal_artifacts_with_storage(maximum_segments, byte_limit, storage)
    }

    pub fn read_selected_wal_artifacts_with_storage<S: RecoveryWalReadStorage>(
        &mut self,
        maximum_segments: NonZeroU64,
        byte_limit: u64,
        selection: &dyn super::RecoveryWalReadSelection,
        storage: &mut S,
    ) -> Result<
        super::RecoverySelectedWalReadOutcome<Vec<ObservedWalArtifact>>,
        RecoveryDiscoveryAllocationFailure<S::Denial>,
    > {
        self.discovery.read_selected_wal_artifacts_with_storage(
            maximum_segments,
            byte_limit,
            selection,
            storage,
        )
    }
}
