use worth_store_physical_format::{RecordArtifactFile, RecordFrameCoordinate};

use crate::filesystem_media::{
    ArtifactRangeReadOutcome, ArtifactTreeFailure, CompletedArtifactRangeRead,
};
use crate::{
    BackendQueueExecutionAdaptation, BackendQueueExecutionCompletion,
    BackendQueueExecutionPlanBinding,
};

use super::AdmittedRecoveryFilesystemMedia;
mod range;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompletedScheduledRecoveryReopenRead {
    store: worth_store_physical_format::store_namespace::StableStoreIdentity,
    artifact: RecordArtifactFile,
    bytes: Box<[u8]>,
    physical: CompletedArtifactRangeRead,
    queue: BackendQueueExecutionCompletion,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeniedScheduledRecoveryReopenRead {
    artifact: RecordArtifactFile,
    failure: ArtifactTreeFailure,
    queue: Option<BackendQueueExecutionCompletion>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RecoveryReopenReadOutcome {
    Completed(CompletedScheduledRecoveryReopenRead),
    Denied(DeniedScheduledRecoveryReopenRead),
}

impl AdmittedRecoveryFilesystemMedia {
    pub fn read_recovery_artifact_scheduled(
        &self,
        artifact: RecordArtifactFile,
        maximum_bytes: u64,
        binding: BackendQueueExecutionPlanBinding,
    ) -> RecoveryReopenReadOutcome {
        let physical = match super::discovery::record_artifact(artifact) {
            Ok(physical) => physical,
            Err(_) => return denied_without_queue(artifact),
        };
        let ticket = match crate::BackendQueueExecutionAuthority::store_owned().issue_ticket(
            binding,
            &self.parts.execution_capability,
            BackendQueueExecutionAdaptation::None,
        ) {
            Ok(ticket) => ticket,
            Err(_) => return denied_without_queue(artifact),
        };
        let media = self.parts.artifact_tree();
        let length = match media.file_length(&physical) {
            Ok(length) if length != 0 && length <= maximum_bytes => length,
            Ok(_) | Err(_) => {
                let queue = ticket.begin_completion().observe_queue_depth(1).complete();
                return RecoveryReopenReadOutcome::Denied(DeniedScheduledRecoveryReopenRead {
                    artifact,
                    failure: structural_denial(),
                    queue: Some(queue),
                });
            }
        };
        let Some(coordinate) = u32::try_from(length)
            .ok()
            .and_then(|length| RecordFrameCoordinate::new(artifact, 0, length))
        else {
            let queue = ticket.begin_completion().observe_queue_depth(1).complete();
            return RecoveryReopenReadOutcome::Denied(DeniedScheduledRecoveryReopenRead {
                artifact,
                failure: structural_denial(),
                queue: Some(queue),
            });
        };
        let mut bytes = vec![0; length as usize];
        let result = media.read_exact_range(&physical, coordinate, &mut bytes);
        let queue = ticket.begin_completion().observe_queue_depth(1).complete();
        match result {
            ArtifactRangeReadOutcome::Completed(physical) => {
                RecoveryReopenReadOutcome::Completed(CompletedScheduledRecoveryReopenRead {
                    store: self.parts.store_identity,
                    artifact,
                    bytes: bytes.into_boxed_slice(),
                    physical,
                    queue,
                })
            }
            ArtifactRangeReadOutcome::DeniedBeforeEffect(failure) => {
                RecoveryReopenReadOutcome::Denied(DeniedScheduledRecoveryReopenRead {
                    artifact,
                    failure,
                    queue: Some(queue),
                })
            }
        }
    }
}

impl CompletedScheduledRecoveryReopenRead {
    /// Retained exact read bytes; other fields are inline values.
    pub fn owned_heap_bytes(&self) -> Option<u64> {
        u64::try_from(self.bytes.len()).ok()
    }

    /// Preserve backend read provenance for the recovery integrity ingress.
    pub fn observed(&self) -> super::ObservedRecoveryArtifact {
        super::ObservedRecoveryArtifact::new(
            self.store,
            super::RecoveryDiscoveryArtifact::Record(self.artifact),
            self.physical.coordinate().offset(),
            Some(self.bytes.to_vec()),
        )
    }
    pub const fn artifact(&self) -> RecordArtifactFile {
        self.artifact
    }
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }
    pub const fn physical(&self) -> CompletedArtifactRangeRead {
        self.physical
    }
    pub const fn queue(&self) -> BackendQueueExecutionCompletion {
        self.queue
    }
}

impl DeniedScheduledRecoveryReopenRead {
    pub const fn artifact(&self) -> RecordArtifactFile {
        self.artifact
    }
    pub const fn failure(&self) -> ArtifactTreeFailure {
        self.failure
    }
    pub const fn queue(&self) -> Option<BackendQueueExecutionCompletion> {
        self.queue
    }
}

fn denied_without_queue(artifact: RecordArtifactFile) -> RecoveryReopenReadOutcome {
    RecoveryReopenReadOutcome::Denied(DeniedScheduledRecoveryReopenRead {
        artifact,
        failure: structural_denial(),
        queue: None,
    })
}

fn structural_denial() -> ArtifactTreeFailure {
    ArtifactTreeFailure::recovery_denial()
}
