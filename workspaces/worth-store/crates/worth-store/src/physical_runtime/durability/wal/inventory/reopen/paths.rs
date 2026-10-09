use worth_store_physical_backend::{ArtifactTreeDirectory, ArtifactTreeFile};
use worth_store_wal::WalSegmentArtifactIdentity;

use super::PhysicalWalOpenFailure;

pub(super) fn wal_directory() -> ArtifactTreeDirectory {
    ArtifactTreeDirectory::families()
        .child("wal")
        .expect("the Store-owned WAL directory is portable")
}

pub(super) fn artifact(
    directory: &ArtifactTreeDirectory,
    identity: WalSegmentArtifactIdentity,
) -> ArtifactTreeFile {
    directory
        .file(&identity.file_name())
        .expect("canonical WAL artifact names are portable")
}

pub(super) fn map_listing_failure(
    failure: worth_store_physical_backend::ArtifactTreeFailure,
) -> PhysicalWalOpenFailure {
    if matches!(
        failure.kind(),
        worth_store_physical_backend::ArtifactTreeFailureKind::AccessLimitExceeded
    ) {
        PhysicalWalOpenFailure::InventoryLimitExceeded
    } else {
        PhysicalWalOpenFailure::Media(failure)
    }
}
