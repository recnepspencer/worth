use worth_store_physical_backend::{
    ArtifactTreeFailure, BackendQueueExecutionCompletion, MediaOperationIdentity,
};
use worth_store_wal::WalSegmentArtifactIdentity;

use super::{
    RecoveryCleanupArtifactRevalidationDenial, RecoveryCleanupArtifactRevalidationProgress,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompletedRecoveryCleanupPhysicalRemoval {
    artifact: WalSegmentArtifactIdentity,
    admission: [u8; 32],
    operation: MediaOperationIdentity,
    queue: BackendQueueExecutionCompletion,
    revalidation: RecoveryCleanupArtifactRevalidationProgress,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeniedRecoveryCleanupPhysicalRemoval {
    artifact: WalSegmentArtifactIdentity,
    admission: [u8; 32],
    cause: RecoveryCleanupRemovalDenialCause,
    queue: Option<BackendQueueExecutionCompletion>,
    revalidation: RecoveryCleanupArtifactRevalidationProgress,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IndeterminateRecoveryCleanupPhysicalRemoval {
    artifact: WalSegmentArtifactIdentity,
    admission: [u8; 32],
    operation: MediaOperationIdentity,
    failure: ArtifactTreeFailure,
    queue: BackendQueueExecutionCompletion,
    revalidation: RecoveryCleanupArtifactRevalidationProgress,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecoveryCleanupRemovalDenialCause {
    Admission,
    Preparation(ArtifactTreeFailure),
    Revalidation(RecoveryCleanupArtifactRevalidationDenial),
    Removal(ArtifactTreeFailure),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RecoveryCleanupRemovalOutcome {
    Completed(Box<CompletedRecoveryCleanupPhysicalRemoval>),
    DeniedBeforeEffect(Box<DeniedRecoveryCleanupPhysicalRemoval>),
    Indeterminate(Box<IndeterminateRecoveryCleanupPhysicalRemoval>),
}

impl CompletedRecoveryCleanupPhysicalRemoval {
    pub(in crate::physical_runtime) const fn new(
        artifact: WalSegmentArtifactIdentity,
        admission: [u8; 32],
        operation: MediaOperationIdentity,
        queue: BackendQueueExecutionCompletion,
        revalidation: RecoveryCleanupArtifactRevalidationProgress,
    ) -> Self {
        Self {
            artifact,
            admission,
            operation,
            queue,
            revalidation,
        }
    }

    pub const fn artifact(&self) -> WalSegmentArtifactIdentity {
        self.artifact
    }
    pub const fn admission(&self) -> [u8; 32] {
        self.admission
    }
    pub const fn operation(&self) -> MediaOperationIdentity {
        self.operation
    }
    pub const fn queue(&self) -> BackendQueueExecutionCompletion {
        self.queue
    }
    pub const fn revalidation(&self) -> RecoveryCleanupArtifactRevalidationProgress {
        self.revalidation
    }
}

impl DeniedRecoveryCleanupPhysicalRemoval {
    pub(in crate::physical_runtime) const fn new(
        artifact: WalSegmentArtifactIdentity,
        admission: [u8; 32],
        cause: RecoveryCleanupRemovalDenialCause,
        queue: Option<BackendQueueExecutionCompletion>,
        revalidation: RecoveryCleanupArtifactRevalidationProgress,
    ) -> Self {
        Self {
            artifact,
            admission,
            cause,
            queue,
            revalidation,
        }
    }

    pub const fn artifact(&self) -> WalSegmentArtifactIdentity {
        self.artifact
    }
    pub const fn admission(&self) -> [u8; 32] {
        self.admission
    }
    pub const fn failure(&self) -> ArtifactTreeFailure {
        self.cause.failure()
    }
    pub const fn cause(&self) -> RecoveryCleanupRemovalDenialCause {
        self.cause
    }
    pub const fn queue(&self) -> Option<BackendQueueExecutionCompletion> {
        self.queue
    }
    pub const fn revalidation(&self) -> RecoveryCleanupArtifactRevalidationProgress {
        self.revalidation
    }
}

impl IndeterminateRecoveryCleanupPhysicalRemoval {
    pub(in crate::physical_runtime) const fn new(
        artifact: WalSegmentArtifactIdentity,
        admission: [u8; 32],
        operation: MediaOperationIdentity,
        failure: ArtifactTreeFailure,
        queue: BackendQueueExecutionCompletion,
        revalidation: RecoveryCleanupArtifactRevalidationProgress,
    ) -> Self {
        Self {
            artifact,
            admission,
            operation,
            failure,
            queue,
            revalidation,
        }
    }

    pub const fn artifact(&self) -> WalSegmentArtifactIdentity {
        self.artifact
    }
    pub const fn admission(&self) -> [u8; 32] {
        self.admission
    }
    pub const fn operation(&self) -> MediaOperationIdentity {
        self.operation
    }
    pub const fn failure(&self) -> ArtifactTreeFailure {
        self.failure
    }
    pub const fn queue(&self) -> BackendQueueExecutionCompletion {
        self.queue
    }
    pub const fn revalidation(&self) -> RecoveryCleanupArtifactRevalidationProgress {
        self.revalidation
    }
}

impl RecoveryCleanupRemovalDenialCause {
    pub const fn failure(self) -> ArtifactTreeFailure {
        match self {
            Self::Preparation(failure) | Self::Removal(failure) => failure,
            Self::Admission | Self::Revalidation(_) => ArtifactTreeFailure::recovery_denial(),
        }
    }
}

impl CompletedRecoveryCleanupPhysicalRemoval {
    pub(in crate::physical_runtime) fn from_backend(
        artifact: WalSegmentArtifactIdentity,
        completed: worth_store_physical_backend::BackendCompletedRecoveryCleanupRemoval,
    ) -> Self {
        Self::new(
            artifact,
            completed.admission(),
            completed.operation(),
            completed.queue(),
            completed.revalidation().into(),
        )
    }
}

impl DeniedRecoveryCleanupPhysicalRemoval {
    pub(in crate::physical_runtime) fn from_backend(
        artifact: WalSegmentArtifactIdentity,
        denied: worth_store_physical_backend::BackendDeniedRecoveryCleanupRemoval,
    ) -> Self {
        Self::new(
            artifact,
            denied.admission(),
            denied.cause().into(),
            denied.queue(),
            denied.revalidation().into(),
        )
    }
}

impl IndeterminateRecoveryCleanupPhysicalRemoval {
    pub(in crate::physical_runtime) fn from_backend(
        artifact: WalSegmentArtifactIdentity,
        indeterminate: worth_store_physical_backend::BackendIndeterminateRecoveryCleanupRemoval,
    ) -> Self {
        Self::new(
            artifact,
            indeterminate.admission(),
            indeterminate.operation(),
            indeterminate.failure(),
            indeterminate.queue(),
            indeterminate.revalidation().into(),
        )
    }
}

impl From<worth_store_physical_backend::BackendRecoveryCleanupRemovalDenialCause>
    for RecoveryCleanupRemovalDenialCause
{
    fn from(cause: worth_store_physical_backend::BackendRecoveryCleanupRemovalDenialCause) -> Self {
        match cause {
            worth_store_physical_backend::BackendRecoveryCleanupRemovalDenialCause::Admission => {
                Self::Admission
            }
            worth_store_physical_backend::BackendRecoveryCleanupRemovalDenialCause::Revalidation(
                denial,
            ) => Self::Revalidation(denial.into()),
            worth_store_physical_backend::BackendRecoveryCleanupRemovalDenialCause::Removal(
                failure,
            ) => Self::Removal(failure),
        }
    }
}
