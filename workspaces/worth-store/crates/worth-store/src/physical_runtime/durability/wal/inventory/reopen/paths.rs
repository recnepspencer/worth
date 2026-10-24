use worth_store_physical_backend::{ArtifactTreeDirectory, ArtifactTreeFile};
use worth_store_wal::WalSegmentArtifactIdentity;

use super::PhysicalWalOpenFailure;

pub(super) fn wal_directory() -> ArtifactTreeDirectory {
    ArtifactTreeDirectory::families()
        .child("wal")
        .expect("the Store-owned WAL directory is portable")
}

/// Creates the WAL directory of a store that has never appended.
pub(super) fn ensure_wal_directory(
    tree: &worth_store_physical_backend::ArtifactTreeMedia<'_>,
    directory: &ArtifactTreeDirectory,
) -> Result<(), PhysicalWalOpenFailure> {
    if !tree
        .directory_exists(directory)
        .map_err(PhysicalWalOpenFailure::Media)?
    {
        tree.create_directory(directory)
            .map_err(PhysicalWalOpenFailure::Media)?;
    }
    Ok(())
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
