//! Pure managed-run, artifact, and provider-release outcome vocabularies shared
//! across durable obligations, managed-run sessions, and primary-graph owners.

use worth_runtime_bridge::facade::{
    BridgeExecutionBasisFinalizationFailureKind, BridgeExecutionBasisSignalTerminal,
    BridgeManagedQueueFailureKind,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryManagedRunTerminalKind {
    Completed,
    Cancelled,
    TimedOut,
    Exhausted,
    Degraded,
    Failed,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryManagedRunCleanupDisposition {
    CleanupComplete,
    CleanupPending,
    RecoveryRequired,
}

/// The native managed-run owner that refused cleanup; the typed failure retains its exact terminal custody.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryManagedRunCleanupFailureKind {
    ProviderMemoryRetained,
    QueueRelease(BridgeManagedQueueFailureKind),
    BridgeFinalization(BridgeExecutionBasisFinalizationFailureKind),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryArtifactDenialKind {
    ArtifactContractNotInstalled,
    ProducerRoleNotInstalled,
    ConsumerRoleNotInstalled,
    ProviderFamilyMismatch,
    EmptySemanticProjection,
    InvalidProductionEvidence,
    ForeignRuntime,
    StaleInstallationGeneration,
    OperationMismatch,
    RunMismatch,
    StageMismatch,
    BasisMismatch,
    ArtifactContractMismatch,
    PayloadOwnerMismatch,
    StageExecutionMismatch,
    ActiveWorkflowRun,
    ProductionClosed,
    MovementForbidden,
    BorrowForbidden,
    LeaseForbidden,
    StaleLifecycleGeneration,
    ActiveBorrow,
    ActiveLease,
    AlreadyDisposed,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryProviderCheckpointReleaseDisposition {
    Released,
    Panicked,
}

impl WorthQueryProviderCheckpointReleaseDisposition {
    pub const fn recovery_required(self) -> bool {
        matches!(self, Self::Panicked)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryProviderCheckpointRetentionFailureKind {
    RetainedByteProbePanicked,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryProviderExecutionDisposalDisposition {
    Completed,
    Rejected,
    Panicked,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryProviderExecutionDestructorDisposition {
    Completed,
    Panicked,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryProviderCheckpointSuspensionFailureKind {
    ProviderRejected,
    ProviderPanicked,
    ProviderExecutionReleaseRecoveryRequired,
    CheckpointMemoryMismatch,
    CheckpointRetention(WorthQueryProviderCheckpointRetentionFailureKind),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryYieldRecoveryKind {
    ArtifactProductionFreeze(WorthQueryArtifactDenialKind),
    BridgeTerminalization(BridgeExecutionBasisFinalizationFailureKind),
    SignalAttemptAlreadyTerminal(BridgeExecutionBasisSignalTerminal),
    ProviderCheckpointSuspension(WorthQueryProviderCheckpointSuspensionFailureKind),
    RetainedBytesExceeded,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryDirectReadmissionRecoveryKind {
    BridgeCleanupFailed,
    ProviderRestorePanicked,
    ProviderRestoreRejectedAfterExecutionAdmission,
    RestoredExecutionReleaseRecoveryRequired,
    CheckpointReleasePanicked,
}

/// Native cleanup refusal is distinct from an inconsistent binding or completion witness.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryMutationCleanupFailureKind {
    Native(WorthQueryManagedRunCleanupFailureKind),
    AffinityInconsistent,
    CompletionInconsistent,
}
