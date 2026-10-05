//! Sealed mechanical dispatch over two real C4 observation surfaces.

use std::num::NonZeroU64;

use worth_store_physical_backend::{
    BorrowedWalFilesystemObservation, BoundedRecoveryFilesystemDiscovery, ObservedWalArtifact,
    RecoveryDiscoveryAllocationFailure, RecoverySelectedWalReadOutcome as Outcome,
    RecoveryWalReadSelection, RecoveryWalReadStorage,
};
use worth_store_physical_format::store_namespace::StableStoreIdentity;

pub(super) enum WalReadSource<'read, 'media> {
    Recovery(&'read mut BoundedRecoveryFilesystemDiscovery),
    Serving(BorrowedWalFilesystemObservation<'media>),
}

impl WalReadSource<'_, '_> {
    pub(super) fn store_identity(&self) -> StableStoreIdentity {
        match self {
            Self::Recovery(source) => source.store_identity(),
            Self::Serving(source) => source.store_identity(),
        }
    }
    pub(super) fn listing_is_qualified(&self) -> bool {
        match self {
            Self::Recovery(source) => source.wal_listing_storage_is_qualified(),
            Self::Serving(source) => source.wal_listing_storage_is_qualified(),
        }
    }
    pub(super) fn path_is_qualified(&self) -> bool {
        match self {
            Self::Recovery(source) => source.wal_path_storage_is_qualified(),
            Self::Serving(source) => source.wal_path_storage_is_qualified(),
        }
    }
    pub(super) fn read<S: RecoveryWalReadStorage>(
        &mut self,
        maximum_segments: NonZeroU64,
        byte_limit: u64,
        storage: &mut S,
        selection: Option<&dyn RecoveryWalReadSelection>,
    ) -> Result<Outcome<Vec<ObservedWalArtifact>>, RecoveryDiscoveryAllocationFailure<S::Denial>>
    {
        match (self, selection) {
            (Self::Recovery(source), None) => source
                .read_wal_artifacts_with_storage(maximum_segments, byte_limit, storage)
                .map(Outcome::Observed),
            (Self::Serving(source), None) => source
                .read_wal_artifacts_with_storage(maximum_segments, byte_limit, storage)
                .map(Outcome::Observed),
            (Self::Recovery(source), Some(selection)) => source
                .read_selected_wal_artifacts_with_storage(
                    maximum_segments,
                    byte_limit,
                    selection,
                    storage,
                ),
            (Self::Serving(source), Some(selection)) => source
                .read_selected_wal_artifacts_with_storage(
                    maximum_segments,
                    byte_limit,
                    selection,
                    storage,
                ),
        }
    }
}
