use worth_store_physical_format::{RecordArtifactFile, RecordFrameCoordinate};

use crate::filesystem_media::{
    ArtifactTreeFailure, ArtifactTreePublicationEffectOutcome, CompletedArtifactAppend,
    CompletedArtifactNewWrite, CompletedArtifactRangeRead, CompletedArtifactTreePublicationEffect,
    IndeterminateArtifactAppend, IndeterminateArtifactNewWrite,
    IndeterminateArtifactTreePublicationEffect,
};
use crate::{
    BackendQueueExecutionAdaptation, BackendQueueExecutionCompletion,
    BackendQueueExecutionPlanBinding,
};

use super::AdmittedRecoveryFilesystemMedia;

mod exact_prefix;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecoveryStagingWriteDisposition {
    Created,
    AlreadyMaterialized,
    CompletedFromExactPrefix,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompletedRecoveryStagingWrite {
    artifact: RecordArtifactFile,
    coordinate: RecordFrameCoordinate,
    payload_digest: [u8; 32],
    disposition: RecoveryStagingWriteDisposition,
    created: Option<CompletedArtifactNewWrite>,
    verified: Option<CompletedArtifactRangeRead>,
    prefix_verified: Option<CompletedArtifactRangeRead>,
    appended: Option<CompletedArtifactAppend>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IndeterminateRecoveryStagingWrite {
    artifact: RecordArtifactFile,
    payload_digest: [u8; 32],
    physical: RecoveryStagingIndeterminatePhysical,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RecoveryStagingIndeterminatePhysical {
    NewArtifact(IndeterminateArtifactNewWrite),
    Append {
        prefix_verified: Option<CompletedArtifactRangeRead>,
        append: IndeterminateArtifactAppend,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompletedScheduledRecoveryStagingWrite {
    physical: CompletedRecoveryStagingWrite,
    queue: BackendQueueExecutionCompletion,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeniedScheduledRecoveryStagingWrite {
    failure: ArtifactTreeFailure,
    queue: Option<BackendQueueExecutionCompletion>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IndeterminateScheduledRecoveryStagingWrite {
    physical: IndeterminateRecoveryStagingWrite,
    queue: BackendQueueExecutionCompletion,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RecoveryStagingWriteOutcome {
    Completed(Box<CompletedScheduledRecoveryStagingWrite>),
    DeniedBeforeEffect(Box<DeniedScheduledRecoveryStagingWrite>),
    Indeterminate(Box<IndeterminateScheduledRecoveryStagingWrite>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompletedScheduledRecoveryStagingSynchronization {
    physical: CompletedArtifactTreePublicationEffect,
    queue: BackendQueueExecutionCompletion,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IndeterminateScheduledRecoveryStagingSynchronization {
    physical: IndeterminateArtifactTreePublicationEffect,
    queue: BackendQueueExecutionCompletion,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RecoveryStagingSynchronizationOutcome {
    Completed(Box<CompletedScheduledRecoveryStagingSynchronization>),
    DeniedBeforeEffect(Box<DeniedScheduledRecoveryStagingWrite>),
    Indeterminate(Box<IndeterminateScheduledRecoveryStagingSynchronization>),
}

impl AdmittedRecoveryFilesystemMedia {
    /// Executes one scheduler-bound C4 ensure-exact operation. The queue ticket
    /// is issued before any media observation, so the convergence read cannot
    /// become an unscheduled recovery side lane.
    pub fn stage_recovery_artifact_scheduled(
        &self,
        artifact: RecordArtifactFile,
        bytes: &[u8],
        binding: BackendQueueExecutionPlanBinding,
    ) -> RecoveryStagingWriteOutcome {
        let Some(coordinate) = u32::try_from(bytes.len())
            .ok()
            .and_then(|length| RecordFrameCoordinate::new(artifact, 0, length))
        else {
            return denied_without_queue();
        };
        let physical = match super::discovery::record_artifact(artifact) {
            Ok(physical) => physical,
            Err(_) => return denied_without_queue(),
        };
        let ticket = match crate::BackendQueueExecutionAuthority::store_owned().issue_ticket(
            binding,
            &self.parts.execution_capability,
            BackendQueueExecutionAdaptation::None,
        ) {
            Ok(ticket) => ticket,
            Err(_) => return denied_without_queue(),
        };
        let media = self.parts.artifact_tree();
        let completed = match media.file_exists(&physical) {
            Ok(true) => {
                exact_prefix::complete_existing(&media, artifact, physical, coordinate, bytes)
            }
            Ok(false) => exact_prefix::create(&media, artifact, physical, coordinate, bytes),
            Err(failure) => Err(RecoveryStagingPhysicalFailure::Denied(failure)),
        };
        let queue = ticket.begin_completion().observe_queue_depth(1).complete();
        match completed {
            Ok(physical) => RecoveryStagingWriteOutcome::Completed(Box::new(
                CompletedScheduledRecoveryStagingWrite { physical, queue },
            )),
            Err(RecoveryStagingPhysicalFailure::Denied(failure)) => {
                RecoveryStagingWriteOutcome::DeniedBeforeEffect(Box::new(
                    DeniedScheduledRecoveryStagingWrite {
                        failure,
                        queue: Some(queue),
                    },
                ))
            }
            Err(RecoveryStagingPhysicalFailure::Indeterminate(physical)) => {
                RecoveryStagingWriteOutcome::Indeterminate(Box::new(
                    IndeterminateScheduledRecoveryStagingWrite { physical, queue },
                ))
            }
        }
    }

    pub fn synchronize_recovery_artifact_scheduled(
        &self,
        artifact: RecordArtifactFile,
        binding: BackendQueueExecutionPlanBinding,
    ) -> RecoveryStagingSynchronizationOutcome {
        let physical = match super::discovery::record_artifact(artifact) {
            Ok(physical) => physical,
            Err(_) => {
                return RecoveryStagingSynchronizationOutcome::DeniedBeforeEffect(Box::new(
                    DeniedScheduledRecoveryStagingWrite {
                        failure: structural_denial(),
                        queue: None,
                    },
                ))
            }
        };
        let ticket = match crate::BackendQueueExecutionAuthority::store_owned().issue_ticket(
            binding,
            &self.parts.execution_capability,
            BackendQueueExecutionAdaptation::None,
        ) {
            Ok(ticket) => ticket,
            Err(_) => {
                return RecoveryStagingSynchronizationOutcome::DeniedBeforeEffect(Box::new(
                    DeniedScheduledRecoveryStagingWrite {
                        failure: structural_denial(),
                        queue: None,
                    },
                ))
            }
        };
        let physical = self
            .parts
            .artifact_tree()
            .synchronize_file_effect(&physical);
        let queue = ticket.begin_completion().observe_queue_depth(1).complete();
        match physical {
            ArtifactTreePublicationEffectOutcome::Completed(physical) => {
                RecoveryStagingSynchronizationOutcome::Completed(Box::new(
                    CompletedScheduledRecoveryStagingSynchronization { physical, queue },
                ))
            }
            ArtifactTreePublicationEffectOutcome::DeniedBeforeEffect(failure) => {
                RecoveryStagingSynchronizationOutcome::DeniedBeforeEffect(Box::new(
                    DeniedScheduledRecoveryStagingWrite {
                        failure,
                        queue: Some(queue),
                    },
                ))
            }
            ArtifactTreePublicationEffectOutcome::Indeterminate(physical) => {
                RecoveryStagingSynchronizationOutcome::Indeterminate(Box::new(
                    IndeterminateScheduledRecoveryStagingSynchronization { physical, queue },
                ))
            }
        }
    }
}

enum RecoveryStagingPhysicalFailure {
    Denied(ArtifactTreeFailure),
    Indeterminate(IndeterminateRecoveryStagingWrite),
}

fn denied_without_queue() -> RecoveryStagingWriteOutcome {
    RecoveryStagingWriteOutcome::DeniedBeforeEffect(Box::new(DeniedScheduledRecoveryStagingWrite {
        failure: structural_denial(),
        queue: None,
    }))
}

fn structural_denial() -> ArtifactTreeFailure {
    ArtifactTreeFailure::recovery_denial()
}

impl CompletedScheduledRecoveryStagingWrite {
    pub const fn physical(&self) -> &CompletedRecoveryStagingWrite {
        &self.physical
    }
    pub const fn queue(&self) -> BackendQueueExecutionCompletion {
        self.queue
    }
}

impl DeniedScheduledRecoveryStagingWrite {
    pub const fn failure(&self) -> ArtifactTreeFailure {
        self.failure
    }
    pub const fn queue(&self) -> Option<BackendQueueExecutionCompletion> {
        self.queue
    }
}

impl IndeterminateScheduledRecoveryStagingWrite {
    pub const fn physical(&self) -> &IndeterminateRecoveryStagingWrite {
        &self.physical
    }
    pub const fn queue(&self) -> BackendQueueExecutionCompletion {
        self.queue
    }
}

impl CompletedScheduledRecoveryStagingSynchronization {
    pub const fn physical(&self) -> &CompletedArtifactTreePublicationEffect {
        &self.physical
    }
    pub const fn queue(&self) -> BackendQueueExecutionCompletion {
        self.queue
    }
}

impl IndeterminateScheduledRecoveryStagingSynchronization {
    pub const fn physical(&self) -> &IndeterminateArtifactTreePublicationEffect {
        &self.physical
    }
    pub const fn queue(&self) -> BackendQueueExecutionCompletion {
        self.queue
    }
}

impl CompletedRecoveryStagingWrite {
    pub const fn artifact(&self) -> RecordArtifactFile {
        self.artifact
    }
    pub const fn coordinate(&self) -> RecordFrameCoordinate {
        self.coordinate
    }
    pub const fn byte_count(&self) -> u64 {
        self.coordinate.length() as u64
    }
    pub const fn payload_digest(&self) -> [u8; 32] {
        self.payload_digest
    }
    pub const fn disposition(&self) -> RecoveryStagingWriteDisposition {
        self.disposition
    }
    pub const fn created(&self) -> Option<&CompletedArtifactNewWrite> {
        self.created.as_ref()
    }
    pub const fn verified(&self) -> Option<CompletedArtifactRangeRead> {
        self.verified
    }
    pub const fn prefix_verified(&self) -> Option<CompletedArtifactRangeRead> {
        self.prefix_verified
    }
    pub const fn appended(&self) -> Option<&CompletedArtifactAppend> {
        self.appended.as_ref()
    }
}

impl IndeterminateRecoveryStagingWrite {
    pub const fn artifact(&self) -> RecordArtifactFile {
        self.artifact
    }
    pub const fn payload_digest(&self) -> [u8; 32] {
        self.payload_digest
    }
    pub const fn evidence(&self) -> &RecoveryStagingIndeterminatePhysical {
        &self.physical
    }
    pub fn into_physical(self) -> RecoveryStagingIndeterminatePhysical {
        self.physical
    }
}
