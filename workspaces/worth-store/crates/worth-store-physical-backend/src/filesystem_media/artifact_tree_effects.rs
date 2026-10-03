use std::io::Write;

use cap_std::fs::Dir;
mod directory_open;
#[cfg(feature = "recovery-runtime-owner")]
pub(super) use directory_open::open_directory_with_backing;
pub(super) use directory_open::{open_directory, open_optional_directory};

use super::{
    ArtifactTreeFailure, ArtifactTreeFailureKind, FilesystemMediaOwner, MediaOperationRole,
};

pub(super) fn directory_has_entries(
    owner: &FilesystemMediaOwner,
    directory: &Dir,
) -> Result<bool, ArtifactTreeFailure> {
    let attempt = begin(owner, MediaOperationRole::ListDirectory, 0);
    if let Some(error) = attempt.fail_before_error() {
        attempt.denied();
        return Err(denied(&error));
    }
    match directory.entries() {
        Ok(mut entries) => match entries.next() {
            None => {
                attempt.completed(0);
                Ok(false)
            }
            Some(Ok(_)) => {
                attempt.completed(0);
                Ok(true)
            }
            Some(Err(error)) => {
                attempt.denied();
                Err(ArtifactTreeFailure::io(
                    ArtifactTreeFailureKind::Damaged,
                    &error,
                ))
            }
        },
        Err(error) => {
            attempt.denied();
            Err(ArtifactTreeFailure::io(
                ArtifactTreeFailureKind::Damaged,
                &error,
            ))
        }
    }
}

pub(super) fn directory_has_other_entry(
    owner: &FilesystemMediaOwner,
    directory: &Dir,
    selected: &str,
) -> Result<bool, ArtifactTreeFailure> {
    let attempt = begin(owner, MediaOperationRole::ListDirectory, 0);
    if let Some(error) = attempt.fail_before_error() {
        attempt.denied();
        return Err(denied(&error));
    }
    let entries = match directory.entries() {
        Ok(entries) => entries,
        Err(error) => {
            attempt.denied();
            return Err(ArtifactTreeFailure::io(
                ArtifactTreeFailureKind::Damaged,
                &error,
            ));
        }
    };
    for entry in entries.take(2) {
        match entry {
            Ok(entry) if entry.file_name() != selected => {
                attempt.completed(0);
                return Ok(true);
            }
            Ok(_) => {}
            Err(error) => {
                attempt.denied();
                return Err(ArtifactTreeFailure::io(
                    ArtifactTreeFailureKind::Damaged,
                    &error,
                ));
            }
        }
    }
    attempt.completed(0);
    Ok(false)
}

pub(super) fn create_directory(
    owner: &FilesystemMediaOwner,
    parent: &Dir,
    name: &str,
) -> Result<(), ArtifactTreeFailure> {
    let attempt = begin(owner, MediaOperationRole::CreateDirectory, 0);
    if let Some(error) = attempt.fail_before_error() {
        attempt.denied();
        return Err(denied(&error));
    }
    match parent.create_dir(name) {
        Ok(()) => {
            attempt.completed(0);
            Ok(())
        }
        Err(error) => {
            attempt.denied();
            let kind = if error.kind() == std::io::ErrorKind::AlreadyExists {
                ArtifactTreeFailureKind::AlreadyExists
            } else {
                ArtifactTreeFailureKind::DeniedBeforeEffect
            };
            Err(ArtifactTreeFailure::io(kind, &error))
        }
    }
}

pub(super) fn artifact_file_length(
    owner: &FilesystemMediaOwner,
    file: &cap_std::fs::File,
) -> Result<u64, ArtifactTreeFailure> {
    let attempt = begin(owner, MediaOperationRole::ReadMetadata, 0);
    complete_artifact_file_length(attempt, file)
}

pub(super) fn identified_artifact_file_length(
    owner: &FilesystemMediaOwner,
    file: &cap_std::fs::File,
) -> Result<(super::MediaOperationIdentity, u64), ArtifactTreeFailure> {
    let Some((operation, attempt)) = begin_identified(owner, MediaOperationRole::ReadMetadata, 0)
    else {
        return Err(ArtifactTreeFailure::structural(
            ArtifactTreeFailureKind::DeniedBeforeEffect,
        ));
    };
    complete_artifact_file_length(attempt, file).map(|length| (operation, length))
}

fn complete_artifact_file_length(
    attempt: super::fault_interposition::MediaBoundaryAttempt<'_>,
    file: &cap_std::fs::File,
) -> Result<u64, ArtifactTreeFailure> {
    if let Some(error) = attempt.fail_before_error() {
        attempt.denied();
        return Err(denied(&error));
    }
    match file.metadata() {
        Ok(metadata) => {
            attempt.completed(0);
            Ok(metadata.len())
        }
        Err(error) => {
            attempt.denied();
            Err(denied(&error))
        }
    }
}

pub(super) fn write_all_interposed(
    owner: &FilesystemMediaOwner,
    file: &mut cap_std::fs::File,
    bytes: &[u8],
) -> Result<(), ArtifactTreeFailure> {
    let requested = bytes.len() as u64;
    let attempt = begin(owner, MediaOperationRole::PositionedWrite, requested);
    if let Some(error) = attempt.fail_before_error() {
        attempt.denied();
        return Err(denied(&error));
    }
    let limit = attempt.transfer_limit(requested) as usize;
    match file.write_all(&bytes[..limit]) {
        Ok(()) if limit != bytes.len() => {
            attempt.partial(limit as u64);
            Err(ArtifactTreeFailure::structural(
                ArtifactTreeFailureKind::PartialWrite {
                    completed_bytes: limit as u64,
                },
            ))
        }
        Ok(()) if attempt.effect_observation_is_indeterminate() => {
            attempt.indeterminate(requested);
            Err(indeterminate())
        }
        Ok(()) => {
            attempt.completed(requested);
            Ok(())
        }
        Err(error) => {
            attempt.indeterminate(0);
            Err(ArtifactTreeFailure::io(
                ArtifactTreeFailureKind::IndeterminateEffect,
                &error,
            ))
        }
    }
}

pub(super) fn synchronize_file(
    owner: &FilesystemMediaOwner,
    file: &cap_std::fs::File,
) -> Result<(), ArtifactTreeFailure> {
    let attempt = begin(owner, MediaOperationRole::SynchronizeFileState, 0);
    synchronize_file_with_attempt(owner, file, attempt)
}

pub(super) fn synchronize_file_for_operation(
    owner: &FilesystemMediaOwner,
    file: &cap_std::fs::File,
    operation: super::MediaOperationIdentity,
) -> Result<(), ArtifactTreeFailure> {
    let attempt = owner.boundary().begin_operation(
        MediaOperationRole::SynchronizeFileState,
        0,
        super::MediaOperationCoordinates::for_path(
            operation,
            super::MediaPathRole::ArtifactOwned,
            None,
        ),
    );
    synchronize_file_with_attempt(owner, file, attempt)
}

fn synchronize_file_with_attempt(
    owner: &FilesystemMediaOwner,
    file: &cap_std::fs::File,
    attempt: super::fault_interposition::MediaBoundaryAttempt<'_>,
) -> Result<(), ArtifactTreeFailure> {
    if let Some(error) = attempt
        .fail_before_error()
        .or_else(|| attempt.barrier_error())
    {
        attempt.denied();
        return Err(denied(&error));
    }
    match file.sync_all() {
        Ok(()) if attempt.effect_observation_is_indeterminate() => {
            attempt.indeterminate(0);
            Err(indeterminate())
        }
        Ok(()) => {
            owner.boundary().counters().file_sync();
            attempt.completed(0);
            Ok(())
        }
        Err(error) => {
            attempt.indeterminate(0);
            Err(ArtifactTreeFailure::io(
                ArtifactTreeFailureKind::IndeterminateEffect,
                &error,
            ))
        }
    }
}

pub(super) fn synchronize_directory(
    owner: &FilesystemMediaOwner,
    directory: &Dir,
) -> Result<(), ArtifactTreeFailure> {
    let attempt = begin(
        owner,
        MediaOperationRole::SynchronizeDirectoryPublication,
        0,
    );
    if let Some(error) = attempt
        .fail_before_error()
        .or_else(|| attempt.barrier_error())
    {
        attempt.denied();
        return Err(denied(&error));
    }
    match super::directory_synchronization::synchronize_directory_handle(directory) {
        Ok(()) if attempt.effect_observation_is_indeterminate() => {
            attempt.indeterminate(0);
            Err(indeterminate())
        }
        Ok(()) => {
            owner.boundary().counters().directory_sync();
            attempt.completed(0);
            Ok(())
        }
        Err(error) => {
            attempt.indeterminate(0);
            Err(ArtifactTreeFailure::io(
                ArtifactTreeFailureKind::IndeterminateEffect,
                &error,
            ))
        }
    }
}

pub(super) fn begin(
    owner: &FilesystemMediaOwner,
    role: MediaOperationRole,
    bytes: u64,
) -> super::fault_interposition::MediaBoundaryAttempt<'_> {
    owner.boundary().begin(role, bytes)
}

pub(super) fn begin_identified(
    owner: &FilesystemMediaOwner,
    role: MediaOperationRole,
    bytes: u64,
) -> Option<(
    super::MediaOperationIdentity,
    super::fault_interposition::MediaBoundaryAttempt<'_>,
)> {
    let operation = owner.issue_operation_identity()?;
    let attempt = owner.boundary().begin_operation(
        role,
        bytes,
        super::MediaOperationCoordinates::for_path(
            operation,
            super::MediaPathRole::ArtifactOwned,
            None,
        ),
    );
    Some((operation, attempt))
}

fn denied(error: &std::io::Error) -> ArtifactTreeFailure {
    ArtifactTreeFailure::io(ArtifactTreeFailureKind::DeniedBeforeEffect, error)
}

const fn indeterminate() -> ArtifactTreeFailure {
    ArtifactTreeFailure::structural(ArtifactTreeFailureKind::IndeterminateEffect)
}
