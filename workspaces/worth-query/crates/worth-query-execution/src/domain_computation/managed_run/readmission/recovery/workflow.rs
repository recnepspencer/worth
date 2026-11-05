use std::sync::Arc;

use super::workflow_progression::WorthQueryWorkflowReadmissionProgressionPermit;
use super::workflow_progression::{
    WorthQueryWorkflowBridgeRecoveryAssociation, WorthQueryWorkflowRestoredAssociation,
};
use super::workflow_recovery_cleanup::WorthQueryWorkflowReadmissionCleanupRequired;
use crate::domain_computation::artifact_owner::WorthQueryArtifactProductionGenerationAbortFailure;
use crate::domain_computation::managed_run::provider_restore::{
    WorthQueryManagedGraphRestorePending, WorthQueryManagedGraphRestoreRecoveryRequired,
};
use crate::domain_computation::managed_run::readmission::evidence::{
    WorthQueryReadmissionEvidence, WorthQueryReadmissionProgress,
};
use crate::domain_computation::managed_run::WorthQueryYieldedWorkflowRun;
use crate::domain_computation::vocabulary::provider_session::provider_execution_release::WorthQueryProviderExecutionReleaseEvidence;
use crate::domain_computation::{
    WorthQueryProviderCheckpointEvidence, WorthQueryProviderCheckpointReleaseEvidence,
};

mod workflow_reassembled;

pub(super) use workflow_reassembled::{owner_retry_required, owner_retry_yielded};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryWorkflowReadmissionRecoveryKind {
    BridgeCleanupFailed,
    ProviderRestorePanicked,
    ProviderRestoreRejectedAfterExecutionAdmission,
    RestoredExecutionReleaseRecoveryRequired,
    CheckpointReleasePanicked,
    ArtifactGenerationRollbackFailed,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryWorkflowReadmissionRecoveryPosture {
    YieldReassemblyPending,
    TerminalCleanupRequired,
    ArtifactGenerationCleanupRequired,
}

#[must_use = "workflow readmission recovery must be explicitly resolved"]
pub enum WorthQueryWorkflowReadmissionRecoveryRequired {
    YieldReassembly(WorthQueryWorkflowReadmissionYieldReassemblyRecovery),
    TerminalCleanup(WorthQueryWorkflowReadmissionTerminalRecovery),
}

#[must_use = "yield reassembly recovery must retry Bridge cleanup or become terminal cleanup"]
pub struct WorthQueryWorkflowReadmissionYieldReassemblyRecovery {
    kind: WorthQueryWorkflowReadmissionRecoveryKind,
    detail: Arc<str>,
    progress: WorthQueryReadmissionProgress,
    recovery: WorthQueryWorkflowBridgeCleanupRecoveryState,
}

#[must_use = "provider or artifact uncertainty can only enter terminal cleanup"]
pub struct WorthQueryWorkflowReadmissionTerminalRecovery {
    kind: WorthQueryWorkflowReadmissionRecoveryKind,
    detail: Arc<str>,
    progress: WorthQueryReadmissionProgress,
    resource: WorthQueryWorkflowReadmissionRecoveryResource,
}

#[must_use = "yield reassembly retains yielded or exact Bridge cleanup recovery authority"]
pub enum WorthQueryWorkflowReadmissionYieldReassemblyOutcome {
    Yielded(WorthQueryWorkflowReadmissionYieldReassembled),
    RecoveryRequired(WorthQueryWorkflowReadmissionYieldReassemblyRecovery),
}

pub struct WorthQueryWorkflowReadmissionYieldReassembled {
    yielded: WorthQueryYieldedWorkflowRun,
    evidence: WorthQueryReadmissionEvidence,
}

enum WorthQueryWorkflowReadmissionRecoveryResource {
    Provider(WorthQueryWorkflowProviderRecoveryState),
    ProviderGeneration(WorthQueryWorkflowProviderGenerationRecoveryState),
    ProviderPending(WorthQueryWorkflowProviderPendingRecoveryState),
}

pub(in crate::domain_computation::managed_run) struct WorthQueryWorkflowReadmissionRecoveryPermit {
    _owner: (),
}

impl WorthQueryWorkflowReadmissionRecoveryPermit {
    fn mint() -> Self {
        Self { _owner: () }
    }
}

struct WorthQueryWorkflowBridgeCleanupRecoveryState {
    association: WorthQueryWorkflowBridgeRecoveryAssociation,
}

struct WorthQueryWorkflowProviderRecoveryState {
    association: WorthQueryWorkflowRestoredAssociation,
    provider: WorthQueryManagedGraphRestoreRecoveryRequired,
}

struct WorthQueryWorkflowProviderGenerationRecoveryState {
    association: WorthQueryWorkflowRestoredAssociation,
    provider: WorthQueryManagedGraphRestoreRecoveryRequired,
    generation_rollback: WorthQueryArtifactProductionGenerationAbortFailure,
}

struct WorthQueryWorkflowProviderPendingRecoveryState {
    association: WorthQueryWorkflowRestoredAssociation,
    provider: WorthQueryManagedGraphRestorePending,
    generation_rollback: WorthQueryArtifactProductionGenerationAbortFailure,
}

impl WorthQueryWorkflowReadmissionRecoveryRequired {
    pub(super) fn bridge_cleanup(
        detail: impl Into<Arc<str>>,
        progress: WorthQueryReadmissionProgress,
        association: WorthQueryWorkflowBridgeRecoveryAssociation,
        _owner: &WorthQueryWorkflowReadmissionProgressionPermit,
    ) -> Self {
        Self::YieldReassembly(WorthQueryWorkflowReadmissionYieldReassemblyRecovery {
            kind: WorthQueryWorkflowReadmissionRecoveryKind::BridgeCleanupFailed,
            detail: detail.into(),
            progress,
            recovery: WorthQueryWorkflowBridgeCleanupRecoveryState { association },
        })
    }

    pub(super) fn provider(
        kind: WorthQueryWorkflowReadmissionRecoveryKind,
        detail: impl Into<Arc<str>>,
        progress: WorthQueryReadmissionProgress,
        association: WorthQueryWorkflowRestoredAssociation,
        provider: WorthQueryManagedGraphRestoreRecoveryRequired,
        _owner: &WorthQueryWorkflowReadmissionProgressionPermit,
    ) -> Self {
        Self::TerminalCleanup(WorthQueryWorkflowReadmissionTerminalRecovery {
            kind,
            detail: detail.into(),
            progress,
            resource: WorthQueryWorkflowReadmissionRecoveryResource::Provider(
                WorthQueryWorkflowProviderRecoveryState {
                    association,
                    provider,
                },
            ),
        })
    }

    pub(super) fn provider_generation(
        detail: impl Into<Arc<str>>,
        progress: WorthQueryReadmissionProgress,
        association: WorthQueryWorkflowRestoredAssociation,
        provider: WorthQueryManagedGraphRestoreRecoveryRequired,
        generation_rollback: WorthQueryArtifactProductionGenerationAbortFailure,
        _owner: &WorthQueryWorkflowReadmissionProgressionPermit,
    ) -> Self {
        Self::TerminalCleanup(WorthQueryWorkflowReadmissionTerminalRecovery {
            kind: WorthQueryWorkflowReadmissionRecoveryKind::ArtifactGenerationRollbackFailed,
            detail: detail.into(),
            progress,
            resource: WorthQueryWorkflowReadmissionRecoveryResource::ProviderGeneration(
                WorthQueryWorkflowProviderGenerationRecoveryState {
                    association,
                    provider,
                    generation_rollback,
                },
            ),
        })
    }

    pub(super) fn provider_pending(
        detail: impl Into<Arc<str>>,
        progress: WorthQueryReadmissionProgress,
        association: WorthQueryWorkflowRestoredAssociation,
        provider: WorthQueryManagedGraphRestorePending,
        generation_rollback: WorthQueryArtifactProductionGenerationAbortFailure,
        _owner: &WorthQueryWorkflowReadmissionProgressionPermit,
    ) -> Self {
        Self::TerminalCleanup(WorthQueryWorkflowReadmissionTerminalRecovery {
            kind: WorthQueryWorkflowReadmissionRecoveryKind::ArtifactGenerationRollbackFailed,
            detail: detail.into(),
            progress,
            resource: WorthQueryWorkflowReadmissionRecoveryResource::ProviderPending(
                WorthQueryWorkflowProviderPendingRecoveryState {
                    association,
                    provider,
                    generation_rollback,
                },
            ),
        })
    }

    pub const fn kind(&self) -> WorthQueryWorkflowReadmissionRecoveryKind {
        match self {
            Self::YieldReassembly(recovery) => recovery.kind,
            Self::TerminalCleanup(recovery) => recovery.kind,
        }
    }

    pub fn detail(&self) -> &str {
        match self {
            Self::YieldReassembly(recovery) => &recovery.detail,
            Self::TerminalCleanup(recovery) => &recovery.detail,
        }
    }

    pub const fn readmission_evidence(&self) -> WorthQueryReadmissionEvidence {
        match self {
            Self::YieldReassembly(recovery) => recovery.progress.evidence(),
            Self::TerminalCleanup(recovery) => recovery.progress.evidence(),
        }
    }

    pub fn posture(&self) -> WorthQueryWorkflowReadmissionRecoveryPosture {
        match self {
            Self::YieldReassembly(_) => {
                WorthQueryWorkflowReadmissionRecoveryPosture::YieldReassemblyPending
            }
            Self::TerminalCleanup(recovery) => recovery.posture(),
        }
    }

    pub fn checkpoint(&self) -> &WorthQueryProviderCheckpointEvidence {
        match self {
            Self::YieldReassembly(recovery) => recovery.recovery.association.checkpoint_evidence(),
            Self::TerminalCleanup(recovery) => recovery.checkpoint(),
        }
    }

    pub fn checkpoint_release(&self) -> Option<&WorthQueryProviderCheckpointReleaseEvidence> {
        match self {
            Self::YieldReassembly(_) => None,
            Self::TerminalCleanup(recovery) => recovery.checkpoint_release(),
        }
    }

    pub fn restored_execution_release_evidence(
        &self,
    ) -> Option<&WorthQueryProviderExecutionReleaseEvidence> {
        match self {
            Self::YieldReassembly(_) => None,
            Self::TerminalCleanup(recovery) => recovery.restored_execution_release_evidence(),
        }
    }
}

impl WorthQueryWorkflowReadmissionYieldReassemblyRecovery {
    pub const fn kind(&self) -> WorthQueryWorkflowReadmissionRecoveryKind {
        self.kind
    }

    pub fn detail(&self) -> &str {
        &self.detail
    }

    pub const fn readmission_evidence(&self) -> WorthQueryReadmissionEvidence {
        self.progress.evidence()
    }

    pub fn retry_to_yielded(self) -> WorthQueryWorkflowReadmissionYieldReassemblyOutcome {
        let owner = WorthQueryWorkflowReadmissionRecoveryPermit::mint();
        let WorthQueryWorkflowBridgeCleanupRecoveryState { association } = self.recovery;
        association
            .owner_retry_cleanup(&owner)
            .owner_resolve_retry(self.progress, &owner)
    }

    pub fn into_cleanup(self) -> WorthQueryWorkflowReadmissionCleanupRequired {
        let owner = WorthQueryWorkflowReadmissionRecoveryPermit::mint();
        let WorthQueryWorkflowBridgeCleanupRecoveryState { association } = self.recovery;
        WorthQueryWorkflowReadmissionCleanupRequired::bridge_recovery(
            association,
            self.progress,
            &owner,
        )
    }
}

impl WorthQueryWorkflowReadmissionTerminalRecovery {
    pub const fn kind(&self) -> WorthQueryWorkflowReadmissionRecoveryKind {
        self.kind
    }

    pub fn detail(&self) -> &str {
        &self.detail
    }

    pub const fn readmission_evidence(&self) -> WorthQueryReadmissionEvidence {
        self.progress.evidence()
    }

    pub fn posture(&self) -> WorthQueryWorkflowReadmissionRecoveryPosture {
        match &self.resource {
            WorthQueryWorkflowReadmissionRecoveryResource::Provider(recovery) => {
                let _ = recovery;
                WorthQueryWorkflowReadmissionRecoveryPosture::TerminalCleanupRequired
            }
            WorthQueryWorkflowReadmissionRecoveryResource::ProviderGeneration(recovery) => {
                let _ = recovery;
                WorthQueryWorkflowReadmissionRecoveryPosture::ArtifactGenerationCleanupRequired
            }
            WorthQueryWorkflowReadmissionRecoveryResource::ProviderPending(recovery) => {
                let _ = recovery;
                WorthQueryWorkflowReadmissionRecoveryPosture::ArtifactGenerationCleanupRequired
            }
        }
    }

    pub fn checkpoint(&self) -> &WorthQueryProviderCheckpointEvidence {
        match &self.resource {
            WorthQueryWorkflowReadmissionRecoveryResource::Provider(recovery) => {
                recovery.provider.checkpoint_evidence()
            }
            WorthQueryWorkflowReadmissionRecoveryResource::ProviderGeneration(recovery) => {
                recovery.provider.checkpoint_evidence()
            }
            WorthQueryWorkflowReadmissionRecoveryResource::ProviderPending(recovery) => {
                recovery.provider.checkpoint_evidence()
            }
        }
    }

    pub fn checkpoint_release(&self) -> Option<&WorthQueryProviderCheckpointReleaseEvidence> {
        match &self.resource {
            WorthQueryWorkflowReadmissionRecoveryResource::Provider(recovery) => {
                recovery.provider.checkpoint_release()
            }
            WorthQueryWorkflowReadmissionRecoveryResource::ProviderGeneration(recovery) => {
                recovery.provider.checkpoint_release()
            }
            WorthQueryWorkflowReadmissionRecoveryResource::ProviderPending(_) => None,
        }
    }

    pub fn restored_execution_release_evidence(
        &self,
    ) -> Option<&WorthQueryProviderExecutionReleaseEvidence> {
        match &self.resource {
            WorthQueryWorkflowReadmissionRecoveryResource::Provider(recovery) => {
                recovery.provider.restored_execution_release_evidence()
            }
            WorthQueryWorkflowReadmissionRecoveryResource::ProviderGeneration(recovery) => {
                recovery.provider.restored_execution_release_evidence()
            }
            WorthQueryWorkflowReadmissionRecoveryResource::ProviderPending(_) => None,
        }
    }

    pub fn into_cleanup(self) -> WorthQueryWorkflowReadmissionCleanupRequired {
        let owner = WorthQueryWorkflowReadmissionRecoveryPermit::mint();
        match self.resource {
            WorthQueryWorkflowReadmissionRecoveryResource::Provider(recovery) => {
                WorthQueryWorkflowReadmissionCleanupRequired::provider(
                    recovery.association,
                    recovery.provider.into_cleanup(),
                    None,
                    self.progress,
                    &owner,
                )
            }
            WorthQueryWorkflowReadmissionRecoveryResource::ProviderGeneration(recovery) => {
                WorthQueryWorkflowReadmissionCleanupRequired::provider(
                    recovery.association,
                    recovery.provider.into_cleanup(),
                    Some(recovery.generation_rollback),
                    self.progress,
                    &owner,
                )
            }
            WorthQueryWorkflowReadmissionRecoveryResource::ProviderPending(recovery) => {
                WorthQueryWorkflowReadmissionCleanupRequired::provider(
                    recovery.association,
                    recovery.provider.into_cleanup(),
                    Some(recovery.generation_rollback),
                    self.progress,
                    &owner,
                )
            }
        }
    }
}
