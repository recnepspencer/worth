//! Typed record addresses constructed only inside admitted path backing.

use super::record_directory;
use crate::filesystem_media::{
    backed_directory, backed_file, ArtifactTreeAllocatedReadFailure, ArtifactTreeBackedPath,
    ArtifactTreeDirectory, ArtifactTreeFile, ArtifactTreePathAllocator,
};
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

pub(in crate::recovery_media::discovery) fn backed_checkpoint_artifact<
    S: ArtifactTreePathAllocator,
>(
    storage: &mut S,
) -> Result<
    ArtifactTreeBackedPath<ArtifactTreeFile, S::PathBacking>,
    ArtifactTreeAllocatedReadFailure<S::Denial>,
> {
    backed_file(
        &ArtifactTreeDirectory::families(),
        "checkpoint.current",
        storage,
    )
}
