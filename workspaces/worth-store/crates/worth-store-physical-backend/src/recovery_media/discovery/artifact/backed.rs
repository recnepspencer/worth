//! Typed record addresses constructed only inside admitted path backing.

use super::record_directory;
use crate::filesystem_media::{
    backed_directory, backed_file, ArtifactTreeAllocatedReadFailure, ArtifactTreeBackedPath,
    ArtifactTreeDirectory, ArtifactTreeFile, ArtifactTreePathAllocator,
};
use crate::recovery_media::ceiling::{CeilingArtifact, StreamArtifact};
use worth_store_physical_format::RecordArtifactFile;

pub(in crate::recovery_media::discovery) fn backed_record_artifact<S: ArtifactTreePathAllocator>(
    address: RecordArtifactFile,
    storage: &mut S,
) -> Result<
    ArtifactTreeBackedPath<ArtifactTreeFile, S::PathBacking>,
    ArtifactTreeAllocatedReadFailure<S::Denial>,
> {
    let (root, components) = record_directory(address);
    let mut components = components.iter();
    let mut directory = backed_directory(
        &root,
        components.next().expect("record directory is nonempty"),
        storage,
    )?;
    for component in components {
        let next = backed_directory(directory.get(), component, storage)?;
        drop(directory);
        directory = next;
    }
    // Format owns spelling and its fixed stack representation. No temporary
    // filename String is created before the FileAddress admission callback.
    let name = address.canonical_file_name();
    let file = backed_file(directory.get(), name.as_str(), storage)?;
    drop(directory);
    Ok(file)
}

/// The tree file a ceiling names, backed by `storage`.
pub(in crate::recovery_media::discovery) fn backed_ceiling_artifact<
    S: ArtifactTreePathAllocator,
>(
    address: CeilingArtifact,
    storage: &mut S,
) -> Result<
    ArtifactTreeBackedPath<ArtifactTreeFile, S::PathBacking>,
    ArtifactTreeAllocatedReadFailure<S::Denial>,
> {
    match address {
        CeilingArtifact::Record(file) => backed_record_artifact(file, storage),
        CeilingArtifact::Stream(StreamArtifact::CurrentCheckpoint) => backed_file(
            &ArtifactTreeDirectory::families(),
            super::CHECKPOINT_FILE,
            storage,
        ),
    }
}
