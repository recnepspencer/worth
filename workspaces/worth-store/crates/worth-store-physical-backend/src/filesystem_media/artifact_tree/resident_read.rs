use std::io::{Read, Seek, SeekFrom};

use super::super::artifact_tree_effects::{artifact_file_length, begin};
use super::{ArtifactTreeFailure, ArtifactTreeFailureKind, ArtifactTreeFile, ArtifactTreeMedia};
use crate::filesystem_media::MediaOperationRole;

#[derive(Debug)]
pub(crate) enum ArtifactTreeAllocatedReadFailure<E> {
    Media(ArtifactTreeFailure),
    Allocation { requested: usize, cause: E },
    BufferLengthMismatch { requested: usize, observed: usize },
}

impl ArtifactTreeMedia<'_> {
    pub(crate) fn read_bounded_with_allocator<E>(
        &self,
        artifact: &ArtifactTreeFile,
        limit: u64,
        allocate: impl FnOnce(usize) -> Result<Vec<u8>, E>,
    ) -> Result<Vec<u8>, ArtifactTreeAllocatedReadFailure<E>> {
        let directory = self
            .open_directory(&artifact.directory)
            .map_err(ArtifactTreeAllocatedReadFailure::Media)?;
        let mut file = self
            .open_readable_file(&directory, &artifact.file_name)
            .map_err(ArtifactTreeAllocatedReadFailure::Media)?;
        let length = artifact_file_length(self.owner, &file)
            .map_err(ArtifactTreeAllocatedReadFailure::Media)?;
        if length > limit || length > usize::MAX as u64 {
            return Err(ArtifactTreeAllocatedReadFailure::Media(
                ArtifactTreeFailure::limit(length, limit),
            ));
        }
        let requested = length as usize;
        let mut bytes = allocate(requested)
            .map_err(|cause| ArtifactTreeAllocatedReadFailure::Allocation { requested, cause })?;
        require_exact_buffer_length(requested, &bytes)?;
        read_exact_owned(self, &mut file, &mut bytes)?;
        Ok(bytes)
    }

    pub(crate) fn read_exact_at_with_allocator<E>(
        &self,
        artifact: &ArtifactTreeFile,
        offset: u64,
        length: usize,
        allocate: impl FnOnce(usize) -> Result<Vec<u8>, E>,
    ) -> Result<Vec<u8>, ArtifactTreeAllocatedReadFailure<E>> {
        let directory = self
            .open_directory(&artifact.directory)
            .map_err(ArtifactTreeAllocatedReadFailure::Media)?;
        let mut file = self
            .open_readable_file(&directory, &artifact.file_name)
            .map_err(ArtifactTreeAllocatedReadFailure::Media)?;
        let end = offset.checked_add(length as u64).ok_or_else(|| {
            ArtifactTreeAllocatedReadFailure::Media(ArtifactTreeFailure::structural(
                ArtifactTreeFailureKind::AccessLimitExceeded,
            ))
        })?;
        if end
            > artifact_file_length(self.owner, &file)
                .map_err(ArtifactTreeAllocatedReadFailure::Media)?
        {
            return Err(ArtifactTreeAllocatedReadFailure::Media(
                ArtifactTreeFailure::structural(ArtifactTreeFailureKind::Damaged),
            ));
        }
        let mut bytes =
            allocate(length).map_err(|cause| ArtifactTreeAllocatedReadFailure::Allocation {
                requested: length,
                cause,
            })?;
        require_exact_buffer_length(length, &bytes)?;
        file.seek(SeekFrom::Start(offset)).map_err(|error| {
            ArtifactTreeAllocatedReadFailure::Media(ArtifactTreeFailure::io(
                ArtifactTreeFailureKind::Damaged,
                &error,
            ))
        })?;
        read_exact_owned(self, &mut file, &mut bytes)?;
        Ok(bytes)
    }
}

fn require_exact_buffer_length<E>(
    requested: usize,
    bytes: &[u8],
) -> Result<(), ArtifactTreeAllocatedReadFailure<E>> {
    if bytes.len() != requested {
        return Err(ArtifactTreeAllocatedReadFailure::BufferLengthMismatch {
            requested,
            observed: bytes.len(),
        });
    }
    Ok(())
}

fn read_exact_owned<E>(
    media: &ArtifactTreeMedia<'_>,
    file: &mut cap_std::fs::File,
    bytes: &mut [u8],
) -> Result<(), ArtifactTreeAllocatedReadFailure<E>> {
    let requested = bytes.len() as u64;
    let read = begin(media.owner, MediaOperationRole::PositionedRead, requested);
    if let Some(error) = read.fail_before_error() {
        read.denied();
        return Err(ArtifactTreeAllocatedReadFailure::Media(
            ArtifactTreeFailure::io(ArtifactTreeFailureKind::DeniedBeforeEffect, &error),
        ));
    }
    if read.transfer_limit(requested) != requested {
        read.denied();
        return Err(ArtifactTreeAllocatedReadFailure::Media(
            ArtifactTreeFailure::structural(ArtifactTreeFailureKind::AccessLimitExceeded),
        ));
    }
    match file.read_exact(bytes) {
        Ok(()) => {
            read.completed(requested);
            Ok(())
        }
        Err(error) => {
            read.denied();
            Err(ArtifactTreeAllocatedReadFailure::Media(
                ArtifactTreeFailure::io(ArtifactTreeFailureKind::Damaged, &error),
            ))
        }
    }
}
