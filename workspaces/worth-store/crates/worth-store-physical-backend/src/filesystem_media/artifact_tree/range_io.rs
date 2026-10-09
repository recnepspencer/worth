use std::io::{Read, Seek, SeekFrom, Write};

use cap_fs_ext::{FollowSymlinks, OpenOptionsFollowExt};
use cap_std::fs::OpenOptions;

use super::super::artifact_tree_effects::{
    artifact_file_length, begin, begin_identified, write_all_interposed,
};
use super::{
    ArtifactNewFileWriteOutcome, ArtifactTreeFailure, ArtifactTreeFailureKind, ArtifactTreeFile,
    ArtifactTreeMedia,
};
use crate::filesystem_media::{FilesystemMediaOwner, MediaOperationRole};

/// A newly-created artifact whose bytes may be supplied in bounded chunks.
pub struct ArtifactTreeNewFile<'media> {
    owner: &'media FilesystemMediaOwner,
    create_operation: crate::filesystem_media::MediaOperationIdentity,
    file: cap_std::fs::File,
    mutation_sequence: crate::filesystem_media::file_mutation_sequence::FileMutationSequence,
    _coordination:
        crate::filesystem_media::artifact_mutation_coordinator::CoordinatedArtifactMutation<'media>,
    completed_bytes: u64,
}

pub(super) enum ArtifactTreeCreateFileOutcome<'media> {
    Created(ArtifactTreeNewFile<'media>),
    DeniedBeforeEffect(ArtifactTreeFailure),
    Indeterminate {
        failure: ArtifactTreeFailure,
        operation: crate::filesystem_media::MediaOperationIdentity,
    },
}

impl ArtifactTreeNewFile<'_> {
    pub const fn create_operation(&self) -> crate::filesystem_media::MediaOperationIdentity {
        self.create_operation
    }

    pub fn write_chunk(&mut self, bytes: &[u8]) -> Result<(), ArtifactTreeFailure> {
        let _sequence = self.mutation_sequence.lock();
        write_all_interposed(self.owner, &mut self.file, bytes)?;
        self.completed_bytes = self
            .completed_bytes
            .checked_add(bytes.len() as u64)
            .ok_or_else(|| ArtifactTreeFailure::structural(ArtifactTreeFailureKind::Damaged))?;
        Ok(())
    }

    pub const fn completed_bytes(&self) -> u64 {
        self.completed_bytes
    }

    pub(super) fn write_exact_artifact_chunk(
        &mut self,
        offset: u64,
        bytes: &[u8],
    ) -> ArtifactNewFileWriteOutcome {
        if self.completed_bytes != 0 || bytes.is_empty() {
            return ArtifactNewFileWriteOutcome::DeniedBeforeEffect(
                ArtifactTreeFailure::structural(ArtifactTreeFailureKind::AccessLimitExceeded),
            );
        }
        let _sequence = self.mutation_sequence.lock();
        if let Err(error) = self.file.seek(SeekFrom::Start(offset)) {
            return ArtifactNewFileWriteOutcome::DeniedBeforeEffect(ArtifactTreeFailure::io(
                ArtifactTreeFailureKind::DeniedBeforeEffect,
                &error,
            ));
        }
        match super::exact_write_effect::execute(self.owner, &mut self.file, bytes) {
            super::exact_write_effect::ExactWriteEffect::DeniedBeforeEffect(failure) => {
                ArtifactNewFileWriteOutcome::DeniedBeforeEffect(failure)
            }
            super::exact_write_effect::ExactWriteEffect::Indeterminate {
                failure,
                completed_bytes,
                operation,
            } => {
                self.completed_bytes = completed_bytes;
                ArtifactNewFileWriteOutcome::Indeterminate {
                    failure,
                    completed_bytes,
                    operation,
                }
            }
            super::exact_write_effect::ExactWriteEffect::Completed(operation) => {
                self.completed_bytes = bytes.len() as u64;
                ArtifactNewFileWriteOutcome::Completed(operation)
            }
        }
    }

    fn write_obligation_record(&mut self, bytes: &[u8]) -> Result<(), ArtifactTreeFailure> {
        let _sequence = self.mutation_sequence.lock();
        let requested = bytes.len() as u64;
        let attempt = begin(self.owner, MediaOperationRole::Append, requested);
        if let Some(error) = attempt.fail_before_error() {
            attempt.denied();
            return Err(ArtifactTreeFailure::io(
                ArtifactTreeFailureKind::DeniedBeforeEffect,
                &error,
            ));
        }
        let limit = attempt.transfer_limit(requested) as usize;
        if let Err(error) = self.file.write_all(&bytes[..limit]) {
            attempt.indeterminate(0);
            return Err(ArtifactTreeFailure::io(
                ArtifactTreeFailureKind::IndeterminateEffect,
                &error,
            ));
        }
        if limit != bytes.len() || attempt.effect_observation_is_indeterminate() {
            attempt.indeterminate(limit as u64);
            return Err(ArtifactTreeFailure::structural(
                ArtifactTreeFailureKind::IndeterminateEffect,
            ));
        }
        self.completed_bytes = requested;
        attempt.completed(requested);
        super::super::artifact_tree_effects::synchronize_file(self.owner, &self.file)
    }
}

impl ArtifactTreeMedia<'_> {
    pub fn write_new_obligation_record(
        &self,
        artifact: &ArtifactTreeFile,
        bytes: &[u8],
    ) -> Result<(), ArtifactTreeFailure> {
        let mut file = self.create_new_file(artifact)?;
        file.write_obligation_record(bytes)
    }

    pub fn create_new_file(
        &self,
        artifact: &ArtifactTreeFile,
    ) -> Result<ArtifactTreeNewFile<'_>, ArtifactTreeFailure> {
        match self.create_new_file_observed(artifact) {
            ArtifactTreeCreateFileOutcome::Created(file) => Ok(file),
            ArtifactTreeCreateFileOutcome::DeniedBeforeEffect(failure)
            | ArtifactTreeCreateFileOutcome::Indeterminate { failure, .. } => Err(failure),
        }
    }

    pub(super) fn create_new_file_observed(
        &self,
        artifact: &ArtifactTreeFile,
    ) -> ArtifactTreeCreateFileOutcome<'_> {
        let coordination = self
            .owner
            .begin_artifact_namespace_mutation(vec![artifact.coordination_key()])
            .map_err(|_| {
                ArtifactTreeFailure::structural(ArtifactTreeFailureKind::DeniedBeforeEffect)
            });
        let coordination = match coordination {
            Ok(coordination) => coordination,
            Err(failure) => {
                return ArtifactTreeCreateFileOutcome::DeniedBeforeEffect(failure);
            }
        };
        let directory = match self.open_directory(&artifact.directory) {
            Ok(directory) => directory,
            Err(failure) => {
                return ArtifactTreeCreateFileOutcome::DeniedBeforeEffect(failure);
            }
        };
        let mut options = OpenOptions::new();
        options
            .read(true)
            .write(true)
            .create_new(true)
            .follow(FollowSymlinks::No);
        let Some((create_operation, create)) =
            begin_identified(self.owner, MediaOperationRole::CreateNew, 0)
        else {
            return ArtifactTreeCreateFileOutcome::DeniedBeforeEffect(
                ArtifactTreeFailure::structural(ArtifactTreeFailureKind::DeniedBeforeEffect),
            );
        };
        if let Some(error) = create.fail_before_error() {
            create.denied();
            return ArtifactTreeCreateFileOutcome::DeniedBeforeEffect(ArtifactTreeFailure::io(
                ArtifactTreeFailureKind::DeniedBeforeEffect,
                &error,
            ));
        }
        match directory.open_with(&artifact.file_name, &options) {
            Ok(file) => {
                let sequence_file = match file.try_clone().map(cap_std::fs::File::into_std) {
                    Ok(file) => file,
                    Err(_) => {
                        create.indeterminate(0);
                        return ArtifactTreeCreateFileOutcome::Indeterminate {
                            failure: ArtifactTreeFailure::structural(
                                ArtifactTreeFailureKind::IndeterminateEffect,
                            ),
                            operation: create_operation,
                        };
                    }
                };
                let mutation_sequence = match self.owner.mutation_sequence_for(&sequence_file) {
                    Ok(sequence) => sequence,
                    Err(_) => {
                        create.indeterminate(0);
                        return ArtifactTreeCreateFileOutcome::Indeterminate {
                            failure: ArtifactTreeFailure::structural(
                                ArtifactTreeFailureKind::IndeterminateEffect,
                            ),
                            operation: create_operation,
                        };
                    }
                };
                create.completed(0);
                ArtifactTreeCreateFileOutcome::Created(ArtifactTreeNewFile {
                    owner: self.owner,
                    create_operation,
                    file,
                    mutation_sequence,
                    _coordination: coordination.release_namespace(),
                    completed_bytes: 0,
                })
            }
            Err(error) => {
                create.denied();
                let kind = if error.kind() == std::io::ErrorKind::AlreadyExists {
                    ArtifactTreeFailureKind::AlreadyExists
                } else {
                    ArtifactTreeFailureKind::DeniedBeforeEffect
                };
                ArtifactTreeCreateFileOutcome::DeniedBeforeEffect(ArtifactTreeFailure::io(
                    kind, &error,
                ))
            }
        }
    }

    pub fn file_length(&self, artifact: &ArtifactTreeFile) -> Result<u64, ArtifactTreeFailure> {
        let directory = self.open_directory(&artifact.directory)?;
        let file = self.open_readable_file(&directory, &artifact.file_name)?;
        artifact_file_length(self.owner, &file)
    }

    pub fn read_exact_at(
        &self,
        artifact: &ArtifactTreeFile,
        offset: u64,
        target: &mut [u8],
    ) -> Result<(), ArtifactTreeFailure> {
        let directory = self.open_directory(&artifact.directory)?;
        let mut file = self.open_readable_file(&directory, &artifact.file_name)?;
        let end = offset.checked_add(target.len() as u64).ok_or_else(|| {
            ArtifactTreeFailure::structural(ArtifactTreeFailureKind::AccessLimitExceeded)
        })?;
        if end > artifact_file_length(self.owner, &file)? {
            return Err(ArtifactTreeFailure::structural(
                ArtifactTreeFailureKind::Damaged,
            ));
        }
        file.seek(SeekFrom::Start(offset))
            .map_err(|error| ArtifactTreeFailure::io(ArtifactTreeFailureKind::Damaged, &error))?;
        let read = begin(
            self.owner,
            MediaOperationRole::PositionedRead,
            target.len() as u64,
        );
        if let Some(error) = read.fail_before_error() {
            read.denied();
            return Err(ArtifactTreeFailure::io(
                ArtifactTreeFailureKind::DeniedBeforeEffect,
                &error,
            ));
        }
        let requested = target.len() as u64;
        if read.transfer_limit(requested) as usize != target.len() {
            read.denied();
            return Err(ArtifactTreeFailure::structural(
                ArtifactTreeFailureKind::AccessLimitExceeded,
            ));
        }
        match file.read_exact(target) {
            Ok(()) => {
                read.completed(requested);
                Ok(())
            }
            Err(error) => {
                read.denied();
                Err(ArtifactTreeFailure::io(
                    ArtifactTreeFailureKind::Damaged,
                    &error,
                ))
            }
        }
    }
}
