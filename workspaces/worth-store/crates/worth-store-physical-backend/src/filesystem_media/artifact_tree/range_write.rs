use std::io::{Seek, SeekFrom};

use worth_store_physical_format::RecordFrameCoordinate;

use super::{
    ArtifactRangeWriteDurability, ArtifactRangeWriteDurabilityRequirement,
    ArtifactRangeWriteOutcome, ArtifactTreeFailure, ArtifactTreeFailureKind, ArtifactTreeFile,
    ArtifactTreeMedia, CompletedArtifactRangeWrite, CompletedScheduledArtifactRangeWrite,
    IndeterminateArtifactRangeWrite, ScheduledArtifactRangeWriteOutcome,
};
use crate::{
    BackendQueueExecutionAdaptation, BackendQueueExecutionPlanBinding, BackendQueueSpeculativeScope,
};

mod arena;
mod candidate_prefix;

#[derive(Clone, Copy)]
enum ArtifactRangeWritePosture {
    ExistingRange,
    AppendAtEof,
    ArenaRange,
    CandidatePrefix,
}

#[derive(Clone, Copy)]
struct ArtifactRangeWriteRequest<'a> {
    artifact: &'a ArtifactTreeFile,
    coordinate: RecordFrameCoordinate,
    bytes: &'a [u8],
    durability: ArtifactRangeWriteDurabilityRequirement,
    posture: ArtifactRangeWritePosture,
}

struct ScheduledArtifactRangeWriteContext {
    binding: BackendQueueExecutionPlanBinding,
    adaptation: BackendQueueExecutionAdaptation,
    writeback_scope: Option<BackendQueueSpeculativeScope>,
}

impl ArtifactTreeMedia<'_> {
    #[allow(clippy::too_many_arguments)]
    pub fn write_scheduled_exact_at(
        &self,
        artifact: &ArtifactTreeFile,
        coordinate: RecordFrameCoordinate,
        bytes: &[u8],
        binding: BackendQueueExecutionPlanBinding,
        adaptation: BackendQueueExecutionAdaptation,
        scope: BackendQueueSpeculativeScope,
        durability: ArtifactRangeWriteDurabilityRequirement,
    ) -> ScheduledArtifactRangeWriteOutcome {
        self.write_scheduled(
            ArtifactRangeWriteRequest::existing(artifact, coordinate, bytes, durability),
            ScheduledArtifactRangeWriteContext {
                binding,
                adaptation,
                writeback_scope: Some(scope),
            },
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn append_scheduled_writeback_at_eof(
        &self,
        artifact: &ArtifactTreeFile,
        coordinate: RecordFrameCoordinate,
        bytes: &[u8],
        binding: BackendQueueExecutionPlanBinding,
        adaptation: BackendQueueExecutionAdaptation,
        scope: BackendQueueSpeculativeScope,
        durability: ArtifactRangeWriteDurabilityRequirement,
    ) -> ScheduledArtifactRangeWriteOutcome {
        self.write_scheduled(
            ArtifactRangeWriteRequest::append(artifact, coordinate, bytes, durability),
            ScheduledArtifactRangeWriteContext {
                binding,
                adaptation,
                writeback_scope: Some(scope),
            },
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn write_scheduled_foreground_exact_at(
        &self,
        artifact: &ArtifactTreeFile,
        coordinate: RecordFrameCoordinate,
        bytes: &[u8],
        binding: BackendQueueExecutionPlanBinding,
        adaptation: BackendQueueExecutionAdaptation,
        durability: ArtifactRangeWriteDurabilityRequirement,
    ) -> ScheduledArtifactRangeWriteOutcome {
        self.write_scheduled(
            ArtifactRangeWriteRequest::existing(artifact, coordinate, bytes, durability),
            ScheduledArtifactRangeWriteContext {
                binding,
                adaptation,
                writeback_scope: None,
            },
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn append_scheduled_foreground_exact_at(
        &self,
        artifact: &ArtifactTreeFile,
        coordinate: RecordFrameCoordinate,
        bytes: &[u8],
        binding: BackendQueueExecutionPlanBinding,
        adaptation: BackendQueueExecutionAdaptation,
        durability: ArtifactRangeWriteDurabilityRequirement,
    ) -> ScheduledArtifactRangeWriteOutcome {
        self.write_scheduled(
            ArtifactRangeWriteRequest::append(artifact, coordinate, bytes, durability),
            ScheduledArtifactRangeWriteContext {
                binding,
                adaptation,
                writeback_scope: None,
            },
        )
    }

    fn write_scheduled(
        &self,
        request: ArtifactRangeWriteRequest<'_>,
        context: ScheduledArtifactRangeWriteContext,
    ) -> ScheduledArtifactRangeWriteOutcome {
        let ticket = match crate::BackendQueueExecutionAuthority::store_owned().issue_ticket(
            context.binding,
            self.execution_capability,
            context.adaptation,
        ) {
            Ok(ticket) => ticket,
            Err(_) => {
                return ScheduledArtifactRangeWriteOutcome::DeniedBeforeEffect(
                    ArtifactTreeFailure::structural(ArtifactTreeFailureKind::DeniedBeforeEffect),
                );
            }
        };
        match self.write_exact(request) {
            ArtifactRangeWriteOutcome::Completed(physical) => {
                let queue = ticket.begin_completion().observe_queue_depth(1);
                let queue = match context.writeback_scope {
                    Some(scope) => queue.observe_write_back(1, scope),
                    None => queue,
                }
                .complete();
                ScheduledArtifactRangeWriteOutcome::Completed(Box::new(
                    CompletedScheduledArtifactRangeWrite { physical, queue },
                ))
            }
            ArtifactRangeWriteOutcome::DeniedBeforeEffect(failure) => {
                ScheduledArtifactRangeWriteOutcome::DeniedBeforeEffect(failure)
            }
            ArtifactRangeWriteOutcome::Indeterminate(failure) => {
                ScheduledArtifactRangeWriteOutcome::Indeterminate(failure)
            }
        }
    }

    pub fn write_exact_at(
        &self,
        artifact: &ArtifactTreeFile,
        coordinate: RecordFrameCoordinate,
        bytes: &[u8],
    ) -> ArtifactRangeWriteOutcome {
        self.write_exact(ArtifactRangeWriteRequest::existing(
            artifact,
            coordinate,
            bytes,
            ArtifactRangeWriteDurabilityRequirement::BufferedWrite,
        ))
    }

    pub fn append_exact_at_eof(
        &self,
        artifact: &ArtifactTreeFile,
        coordinate: RecordFrameCoordinate,
        bytes: &[u8],
    ) -> ArtifactRangeWriteOutcome {
        self.write_exact(ArtifactRangeWriteRequest::append(
            artifact,
            coordinate,
            bytes,
            ArtifactRangeWriteDurabilityRequirement::BufferedWrite,
        ))
    }

    fn write_exact(&self, request: ArtifactRangeWriteRequest<'_>) -> ArtifactRangeWriteOutcome {
        let end = match request.validate() {
            Ok(end) => end,
            Err(failure) => return ArtifactRangeWriteOutcome::DeniedBeforeEffect(failure),
        };
        let _coordination = match self
            .owner
            .begin_artifact_mutation(vec![request.artifact.coordination_key()])
        {
            Ok(coordination) => coordination,
            Err(_) => {
                return ArtifactRangeWriteOutcome::DeniedBeforeEffect(
                    ArtifactTreeFailure::structural(ArtifactTreeFailureKind::DeniedBeforeEffect),
                );
            }
        };
        let mut file = match self.prepare_write_target(request, end) {
            Ok(file) => file,
            Err(failure) => return ArtifactRangeWriteOutcome::DeniedBeforeEffect(failure),
        };
        let sequence_file = match file.try_clone().map(cap_std::fs::File::into_std) {
            Ok(file) => file,
            Err(error) => {
                return ArtifactRangeWriteOutcome::DeniedBeforeEffect(ArtifactTreeFailure::io(
                    ArtifactTreeFailureKind::DeniedBeforeEffect,
                    &error,
                ));
            }
        };
        let sequence = match self.owner.mutation_sequence_for(&sequence_file) {
            Ok(sequence) => sequence,
            Err(error) => {
                return ArtifactRangeWriteOutcome::DeniedBeforeEffect(ArtifactTreeFailure::io(
                    ArtifactTreeFailureKind::DeniedBeforeEffect,
                    &error,
                ));
            }
        };
        let _sequence = sequence.lock();
        let effect = super::exact_write_effect::execute(self.owner, &mut file, request.bytes);
        self.settle_exact_write(request, &file, effect)
    }

    fn prepare_write_target(
        &self,
        request: ArtifactRangeWriteRequest<'_>,
        end: u64,
    ) -> Result<cap_std::fs::File, ArtifactTreeFailure> {
        let directory = self.open_directory(&request.artifact.directory)?;
        let mut file = self.open_mutable_file(&directory, &request.artifact.file_name)?;
        let length = super::super::artifact_tree_effects::artifact_file_length(self.owner, &file)?;
        if !request.accepts_file_length(length, end) {
            return Err(ArtifactTreeFailure::structural(
                ArtifactTreeFailureKind::AccessLimitExceeded,
            ));
        }
        if matches!(request.posture, ArtifactRangeWritePosture::CandidatePrefix) {
            candidate_prefix::verify(self.owner, &mut file, request.bytes, length)?;
        }
        file.seek(SeekFrom::Start(request.coordinate.offset()))
            .map_err(|error| {
                ArtifactTreeFailure::io(ArtifactTreeFailureKind::DeniedBeforeEffect, &error)
            })?;
        Ok(file)
    }

    fn settle_exact_write(
        &self,
        request: ArtifactRangeWriteRequest<'_>,
        file: &cap_std::fs::File,
        effect: super::exact_write_effect::ExactWriteEffect,
    ) -> ArtifactRangeWriteOutcome {
        match effect {
            super::exact_write_effect::ExactWriteEffect::DeniedBeforeEffect(failure) => {
                ArtifactRangeWriteOutcome::DeniedBeforeEffect(failure)
            }
            super::exact_write_effect::ExactWriteEffect::Indeterminate {
                failure,
                completed_bytes,
                operation,
            } => ArtifactRangeWriteOutcome::Indeterminate(IndeterminateArtifactRangeWrite::new(
                failure,
                self.owner.identity(),
                self.store,
                request.coordinate,
                request.bytes,
                completed_bytes,
                operation,
            )),
            super::exact_write_effect::ExactWriteEffect::Completed(operation) => {
                self.complete_exact_write(request, file, operation)
            }
        }
    }

    fn complete_exact_write(
        &self,
        request: ArtifactRangeWriteRequest<'_>,
        file: &cap_std::fs::File,
        operation: crate::filesystem_media::MediaOperationIdentity,
    ) -> ArtifactRangeWriteOutcome {
        let durability = match request.durability {
            ArtifactRangeWriteDurabilityRequirement::BufferedWrite => {
                ArtifactRangeWriteDurability::BufferedWriteCompleted
            }
            ArtifactRangeWriteDurabilityRequirement::FileDataSynchronization => {
                if let Err(failure) =
                    super::super::artifact_tree_effects::synchronize_file_for_operation(
                        self.owner, file, operation,
                    )
                {
                    return ArtifactRangeWriteOutcome::Indeterminate(
                        IndeterminateArtifactRangeWrite::new(
                            failure,
                            self.owner.identity(),
                            self.store,
                            request.coordinate,
                            request.bytes,
                            request.bytes.len() as u64,
                            operation,
                        ),
                    );
                }
                ArtifactRangeWriteDurability::FileDataSynchronized
            }
        };
        let mut receipt = CompletedArtifactRangeWrite::buffered(
            self.owner.identity(),
            self.store,
            request.coordinate,
            request.bytes,
            operation,
        );
        receipt.set_durability(durability);
        ArtifactRangeWriteOutcome::Completed(receipt)
    }
}

impl<'a> ArtifactRangeWriteRequest<'a> {
    const fn existing(
        artifact: &'a ArtifactTreeFile,
        coordinate: RecordFrameCoordinate,
        bytes: &'a [u8],
        durability: ArtifactRangeWriteDurabilityRequirement,
    ) -> Self {
        Self {
            artifact,
            coordinate,
            bytes,
            durability,
            posture: ArtifactRangeWritePosture::ExistingRange,
        }
    }

    const fn append(
        artifact: &'a ArtifactTreeFile,
        coordinate: RecordFrameCoordinate,
        bytes: &'a [u8],
        durability: ArtifactRangeWriteDurabilityRequirement,
    ) -> Self {
        Self {
            artifact,
            coordinate,
            bytes,
            durability,
            posture: ArtifactRangeWritePosture::AppendAtEof,
        }
    }

    fn validate(self) -> Result<u64, ArtifactTreeFailure> {
        if self.bytes.len() != self.coordinate.length() as usize
            || self.artifact.file_name != self.coordinate.artifact().file_name()
        {
            return Err(ArtifactTreeFailure::structural(
                ArtifactTreeFailureKind::AccessLimitExceeded,
            ));
        }
        self.coordinate
            .offset()
            .checked_add(self.bytes.len() as u64)
            .ok_or_else(|| {
                ArtifactTreeFailure::structural(ArtifactTreeFailureKind::AccessLimitExceeded)
            })
    }

    const fn accepts_file_length(self, length: u64, end: u64) -> bool {
        match self.posture {
            ArtifactRangeWritePosture::ExistingRange => end <= length,
            ArtifactRangeWritePosture::AppendAtEof => self.coordinate.offset() == length,
            ArtifactRangeWritePosture::ArenaRange => true,
            ArtifactRangeWritePosture::CandidatePrefix => {
                self.coordinate.offset() == 0 && length <= end
            }
        }
    }
}
