use crate::filesystem_media::{
    ArtifactTreeFailure, ArtifactTreeFailureKind, FilesystemMediaOwner, MediaOperationRole,
};
use cap_fs_ext::DirExt;
use cap_std::fs::Dir;

pub(in crate::filesystem_media) fn open_directory(
    owner: &FilesystemMediaOwner,
    parent: &Dir,
    name: &str,
) -> Result<Dir, ArtifactTreeFailure> {
    open_directory_with_backing(owner, parent, name, ())
}

pub(in crate::filesystem_media) fn open_directory_with_backing<B>(
    owner: &FilesystemMediaOwner,
    parent: &Dir,
    name: &str,
    backing: B,
) -> Result<Dir, ArtifactTreeFailure> {
    let attempt = super::begin(owner, MediaOperationRole::OpenDirectory, 0);
    if let Some(error) = attempt.fail_before_error() {
        drop(backing);
        attempt.denied();
        return Err(super::denied(&error));
    }
    let result = parent.open_dir_nofollow(name);
    // Cap scratch is gone on return. PauseAfter must not retain its grant.
    drop(backing);
    match result {
        Ok(directory) => {
            attempt.completed(0);
            Ok(directory)
        }
        Err(error) => {
            attempt.denied();
            let kind = if error.kind() == std::io::ErrorKind::NotFound {
                ArtifactTreeFailureKind::Absent
            } else {
                ArtifactTreeFailureKind::DeniedBeforeEffect
            };
            Err(ArtifactTreeFailure::io(kind, &error))
        }
    }
}

pub(in crate::filesystem_media) fn open_optional_directory(
    owner: &FilesystemMediaOwner,
    parent: &Dir,
    name: &str,
) -> Result<Option<Dir>, ArtifactTreeFailure> {
    match open_directory(owner, parent, name) {
        Ok(directory) => Ok(Some(directory)),
        Err(failure) if failure.kind() == ArtifactTreeFailureKind::Absent => Ok(None),
        Err(failure) => Err(failure),
    }
}
