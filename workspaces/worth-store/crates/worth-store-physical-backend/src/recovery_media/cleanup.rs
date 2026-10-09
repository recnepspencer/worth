use worth_store_physical_format::store_namespace::StableStoreIdentity;

use super::AdmittedRecoveryFilesystemMedia;
use crate::filesystem_media::{
    ArtifactTreeFailure, ArtifactTreeFile, ArtifactTreePublicationEffectOutcome,
    CompletedArtifactTreePublicationEffect, IndeterminateArtifactTreePublicationEffect,
};
use crate::{BackendQueueExecutionAdaptation, BackendQueueExecutionCompletion};

mod revalidation;
pub use revalidation::{
    BackendRecoveryCleanupArtifactRevalidationDenial,
    BackendRecoveryCleanupArtifactRevalidationProgress,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackendCompletedRecoveryCleanupRemoval {
    artifact: ArtifactTreeFile,
    admission: [u8; 32],
    physical: CompletedArtifactTreePublicationEffect,
    queue: BackendQueueExecutionCompletion,
    revalidation: BackendRecoveryCleanupArtifactRevalidationProgress,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackendDeniedRecoveryCleanupRemoval {
    artifact: ArtifactTreeFile,
    admission: [u8; 32],
    cause: BackendRecoveryCleanupRemovalDenialCause,
    queue: Option<BackendQueueExecutionCompletion>,
    revalidation: BackendRecoveryCleanupArtifactRevalidationProgress,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackendIndeterminateRecoveryCleanupRemoval {
    artifact: ArtifactTreeFile,
    admission: [u8; 32],
    physical: IndeterminateArtifactTreePublicationEffect,
    queue: BackendQueueExecutionCompletion,
    revalidation: BackendRecoveryCleanupArtifactRevalidationProgress,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackendRecoveryCleanupRemovalDenialCause {
    Admission,
    Revalidation(BackendRecoveryCleanupArtifactRevalidationDenial),
    Removal(ArtifactTreeFailure),
}

/// Exact physical bytes expected at one recovery artifact coordinate.
///
/// This is descriptive input, not removal authority. The cleanup execution
/// capability separately binds the complete request to one admitted media
/// owner before C.4 will perform any reread or unlink.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackendRecoveryArtifactExpectation {
    artifact: ArtifactTreeFile,
    byte_count: u64,
    digest: [u8; 32],
}

/// Pure C.4 request for revalidating and durably removing one exact recovery
/// artifact. It is deliberately physical: the Store recovery owner validates
/// whether the selected checkpoint permits removing this artifact before the
/// request exists. The backend does not reimplement that admission decision.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackendRecoveryCleanupRemovalRequest {
    store: StableStoreIdentity,
    session: [u8; 16],
    cleanup_plan: [u8; 32],
    checkpoint: BackendRecoveryArtifactExpectation,
    artifact: BackendRecoveryArtifactExpectation,
    admission: [u8; 32],
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BackendRecoveryCleanupRemovalOutcome {
    Completed(Box<BackendCompletedRecoveryCleanupRemoval>),
    DeniedBeforeEffect(Box<BackendDeniedRecoveryCleanupRemoval>),
    Indeterminate(Box<BackendIndeterminateRecoveryCleanupRemoval>),
}

#[derive(Debug, Clone)]
struct BackendCleanupRemovalBasis {
    artifact: ArtifactTreeFile,
    admission: [u8; 32],
    revalidation: BackendRecoveryCleanupArtifactRevalidationProgress,
}

/// Executes C.4 reread and durable-removal mechanics for one exact physical
/// recovery artifact after the Store cleanup owner has consumed its private
/// command.
///
/// This is a mechanism boundary, not cleanup admission. It cannot mint Store
/// cleanup eligibility or performed evidence. The Store owner remains
/// responsible for fresh-reopen, checkpoint/WAL, last-copy, policy, plan, and
/// per-artifact consumption before calling this adapter.
#[doc(hidden)]
pub fn execute_recovery_cleanup_removal(
    media: &AdmittedRecoveryFilesystemMedia,
    request: BackendRecoveryCleanupRemovalRequest,
    binding: crate::BackendQueueExecutionPlanBinding,
) -> BackendRecoveryCleanupRemovalOutcome {
    if request.store != media.store_identity() {
        return denied_without_queue(
            request.artifact().clone(),
            request.admission,
            BackendRecoveryCleanupRemovalDenialCause::Admission,
        );
    }
    let ticket = match crate::BackendQueueExecutionAuthority::store_owned().issue_ticket(
        binding,
        &media.parts.execution_capability,
        BackendQueueExecutionAdaptation::None,
    ) {
        Ok(ticket) => ticket,
        Err(_) => {
            return denied_without_queue(
                request.artifact().clone(),
                request.admission,
                BackendRecoveryCleanupRemovalDenialCause::Admission,
            )
        }
    };
    let queue = ticket.begin_completion().observe_queue_depth(1).complete();
    let revalidated = match revalidation::verify(media, &request) {
        Ok(revalidated) => revalidated,
        Err(failure) => {
            return denied_with_queue(
                BackendCleanupRemovalBasis {
                    artifact: request.artifact().clone(),
                    admission: request.admission,
                    revalidation: failure.progress(),
                },
                BackendRecoveryCleanupRemovalDenialCause::Revalidation(failure.denial()),
                queue,
            )
        }
    };
    let physical = media
        .parts
        .artifact_tree()
        .remove_file_durably_observed(revalidated.artifact());
    lower_removal(
        BackendCleanupRemovalBasis {
            artifact: request.artifact().clone(),
            admission: request.admission,
            revalidation: revalidated.progress(),
        },
        physical,
        queue,
    )
}

fn lower_removal(
    basis: BackendCleanupRemovalBasis,
    physical: ArtifactTreePublicationEffectOutcome,
    queue: BackendQueueExecutionCompletion,
) -> BackendRecoveryCleanupRemovalOutcome {
    match physical {
        ArtifactTreePublicationEffectOutcome::Completed(physical) => {
            BackendRecoveryCleanupRemovalOutcome::Completed(Box::new(
                BackendCompletedRecoveryCleanupRemoval {
                    artifact: basis.artifact,
                    admission: basis.admission,
                    physical,
                    queue,
                    revalidation: basis.revalidation,
                },
            ))
        }
        ArtifactTreePublicationEffectOutcome::DeniedBeforeEffect(failure) => denied_with_queue(
            basis,
            BackendRecoveryCleanupRemovalDenialCause::Removal(failure),
            queue,
        ),
        ArtifactTreePublicationEffectOutcome::Indeterminate(physical) => {
            BackendRecoveryCleanupRemovalOutcome::Indeterminate(Box::new(
                BackendIndeterminateRecoveryCleanupRemoval {
                    artifact: basis.artifact,
                    admission: basis.admission,
                    physical,
                    queue,
                    revalidation: basis.revalidation,
                },
            ))
        }
    }
}

fn denied_with_queue(
    basis: BackendCleanupRemovalBasis,
    cause: BackendRecoveryCleanupRemovalDenialCause,
    queue: BackendQueueExecutionCompletion,
) -> BackendRecoveryCleanupRemovalOutcome {
    BackendRecoveryCleanupRemovalOutcome::DeniedBeforeEffect(Box::new(
        BackendDeniedRecoveryCleanupRemoval {
            artifact: basis.artifact,
            admission: basis.admission,
            cause,
            queue: Some(queue),
            revalidation: basis.revalidation,
        },
    ))
}

fn denied_without_queue(
    artifact: ArtifactTreeFile,
    admission: [u8; 32],
    cause: BackendRecoveryCleanupRemovalDenialCause,
) -> BackendRecoveryCleanupRemovalOutcome {
    BackendRecoveryCleanupRemovalOutcome::DeniedBeforeEffect(Box::new(
        BackendDeniedRecoveryCleanupRemoval {
            artifact,
            admission,
            cause,
            queue: None,
            revalidation: BackendRecoveryCleanupArtifactRevalidationProgress::default(),
        },
    ))
}

impl BackendCompletedRecoveryCleanupRemoval {
    pub const fn artifact(&self) -> &ArtifactTreeFile {
        &self.artifact
    }
    pub const fn admission(&self) -> [u8; 32] {
        self.admission
    }
    pub const fn physical(&self) -> &CompletedArtifactTreePublicationEffect {
        &self.physical
    }
    pub const fn operation(&self) -> crate::MediaOperationIdentity {
        self.physical.operation()
    }
    pub const fn queue(&self) -> BackendQueueExecutionCompletion {
        self.queue
    }
    pub const fn revalidation(&self) -> BackendRecoveryCleanupArtifactRevalidationProgress {
        self.revalidation
    }
}

impl BackendDeniedRecoveryCleanupRemoval {
    pub const fn artifact(&self) -> &ArtifactTreeFile {
        &self.artifact
    }
    pub const fn admission(&self) -> [u8; 32] {
        self.admission
    }
    pub const fn failure(&self) -> ArtifactTreeFailure {
        self.cause.failure()
    }
    pub const fn cause(&self) -> BackendRecoveryCleanupRemovalDenialCause {
        self.cause
    }
    pub const fn queue(&self) -> Option<BackendQueueExecutionCompletion> {
        self.queue
    }
    pub const fn revalidation(&self) -> BackendRecoveryCleanupArtifactRevalidationProgress {
        self.revalidation
    }
}

impl BackendIndeterminateRecoveryCleanupRemoval {
    pub const fn artifact(&self) -> &ArtifactTreeFile {
        &self.artifact
    }
    pub const fn admission(&self) -> [u8; 32] {
        self.admission
    }
    pub const fn physical(&self) -> &IndeterminateArtifactTreePublicationEffect {
        &self.physical
    }
    pub const fn operation(&self) -> crate::MediaOperationIdentity {
        self.physical.operation()
    }
    pub const fn failure(&self) -> ArtifactTreeFailure {
        self.physical.failure()
    }
    pub const fn queue(&self) -> BackendQueueExecutionCompletion {
        self.queue
    }
    pub const fn revalidation(&self) -> BackendRecoveryCleanupArtifactRevalidationProgress {
        self.revalidation
    }
}

impl BackendRecoveryCleanupRemovalDenialCause {
    pub const fn failure(self) -> ArtifactTreeFailure {
        match self {
            Self::Removal(failure) => failure,
            Self::Admission | Self::Revalidation(_) => ArtifactTreeFailure::recovery_denial(),
        }
    }
}

impl BackendRecoveryCleanupRemovalRequest {
    pub fn new(
        store: StableStoreIdentity,
        session: [u8; 16],
        cleanup_plan: [u8; 32],
        checkpoint: BackendRecoveryArtifactExpectation,
        artifact: BackendRecoveryArtifactExpectation,
        admission: [u8; 32],
    ) -> Option<Self> {
        (session != [0; 16] && cleanup_plan != [0; 32] && admission != [0; 32]).then_some(Self {
            store,
            session,
            cleanup_plan,
            checkpoint,
            artifact,
            admission,
        })
    }

    pub const fn store(&self) -> StableStoreIdentity {
        self.store
    }

    pub const fn session(&self) -> [u8; 16] {
        self.session
    }

    pub const fn cleanup_plan(&self) -> [u8; 32] {
        self.cleanup_plan
    }

    pub const fn checkpoint(&self) -> &ArtifactTreeFile {
        self.checkpoint.artifact()
    }

    pub const fn checkpoint_bytes(&self) -> u64 {
        self.checkpoint.byte_count()
    }

    pub const fn checkpoint_digest(&self) -> [u8; 32] {
        self.checkpoint.digest()
    }

    pub const fn artifact(&self) -> &ArtifactTreeFile {
        self.artifact.artifact()
    }

    pub const fn artifact_bytes(&self) -> u64 {
        self.artifact.byte_count()
    }

    pub const fn artifact_digest(&self) -> [u8; 32] {
        self.artifact.digest()
    }

    pub const fn admission(&self) -> [u8; 32] {
        self.admission
    }
}

impl BackendRecoveryArtifactExpectation {
    pub fn new(artifact: ArtifactTreeFile, byte_count: u64, digest: [u8; 32]) -> Option<Self> {
        (byte_count != 0).then_some(Self {
            artifact,
            byte_count,
            digest,
        })
    }

    /// Preserves the exact bounded checkpoint-residue observation, including
    /// an interrupted zero-byte candidate. This does not grant unlink authority.
    #[cfg(feature = "recovery-runtime-owner")]
    pub fn from_checkpoint_observation(
        observation: &super::ObservedRecoveryCheckpointArtifact,
    ) -> Self {
        Self {
            artifact: observation.artifact().clone(),
            byte_count: observation.byte_count(),
            digest: observation.digest(),
        }
    }

    pub const fn artifact(&self) -> &ArtifactTreeFile {
        &self.artifact
    }

    pub const fn byte_count(&self) -> u64 {
        self.byte_count
    }

    pub const fn digest(&self) -> [u8; 32] {
        self.digest
    }
}
