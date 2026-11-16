use std::sync::Arc;

use crate::domain_computation::managed_run::readmission::direct_lane::preflight::preparation::{
    WorthQueryDirectBridgeReadmissionAttempt, WorthQueryDirectYieldedState,
};

use worth_runtime_bridge::facade::BridgeExecutionBasisReadmissionCleanupOutcome;
use worth_runtime_bridge::facade::{
    BridgeExecutionBasisReadmissionPending, BridgeExecutionBasisReadmissionRecoveryRequired,
};

mod cleanup;

pub use cleanup::{
    WorthQueryDirectReadmissionCleanupInspection, WorthQueryDirectReadmissionCleanupOutcome,
    WorthQueryDirectReadmissionCleanupPending, WorthQueryDirectReadmissionCleanupPendingInspection,
    WorthQueryDirectReadmissionCleanupReceipt, WorthQueryDirectReadmissionCleanupRequired,
};

use super::super::WorthQueryDirectYieldedParts;
use crate::domain_computation::managed_run::provider_restore::WorthQueryManagedGraphRestoreRecoveryRequired;
use crate::domain_computation::managed_run::readmission::direct_lane::preflight::preparation::WorthQueryDirectRetainedState;
use crate::domain_computation::managed_run::readmission::evidence::{
    WorthQueryReadmissionEvidence, WorthQueryReadmissionProgress,
};
use crate::domain_computation::managed_run::retained_graph_execution::WorthQueryRetainedManagedGraphExecution;
use crate::domain_computation::managed_run::run_affinity::{
    WorthQueryDirectRunAffinity, WorthQueryDirectRunReadmissionPending,
};
use crate::domain_computation::managed_run::WorthQueryYieldedDirectRun;
use crate::domain_computation::provider_session::graph_provider::bounded_step::WorthQueryProviderExecutionReleaseEvidence;
use crate::domain_computation::{
    WorthQueryProviderCheckpointEvidence, WorthQueryProviderCheckpointReleaseEvidence,
};

pub(super) struct WorthQueryDirectBridgeCleanupRecoveryState {
    state: WorthQueryDirectRetainedState,
    affinity: WorthQueryDirectRunAffinity,
    execution: WorthQueryRetainedManagedGraphExecution,
    bridge: BridgeExecutionBasisReadmissionRecoveryRequired,
}

struct WorthQueryDirectYieldedReassembly {
    state: WorthQueryDirectRetainedState,
    affinity: WorthQueryDirectRunAffinity,
    execution: WorthQueryRetainedManagedGraphExecution,
}

pub(super) struct WorthQueryDirectProviderRecoveryState {
    state: WorthQueryDirectRetainedState,
    resource: WorthQueryDirectRunReadmissionPending,
    bridge: BridgeExecutionBasisReadmissionPending,
    provider: WorthQueryManagedGraphRestoreRecoveryRequired,
}

impl WorthQueryDirectBridgeCleanupRecoveryState {
    pub(super) fn from_bridge_attempt(
        attempt: WorthQueryDirectBridgeReadmissionAttempt,
        bridge: BridgeExecutionBasisReadmissionRecoveryRequired,
    ) -> Self {
        Self {
            state: attempt.state,
            affinity: attempt.resource.abort(
                crate::domain_computation::managed_run::WorthQueryDirectReadmissionTransitionPermit::mint(),
            ),
            execution: attempt.execution,
            bridge,
        }
    }

    pub(super) fn from_rollback(
        rollback: super::WorthQueryDirectRollbackAfterBridgeAbort,
        bridge: BridgeExecutionBasisReadmissionRecoveryRequired,
    ) -> Self {
        Self {
            state: rollback.state,
            affinity: rollback.resource.abort(
                crate::domain_computation::managed_run::WorthQueryDirectReadmissionTransitionPermit::mint(),
            ),
            execution: rollback.execution,
            bridge,
        }
    }
}

impl WorthQueryDirectProviderRecoveryState {
    pub(super) fn from_seed(
        seed: super::WorthQueryDirectProviderRecoverySeed,
        provider: WorthQueryManagedGraphRestoreRecoveryRequired,
    ) -> Self {
        Self {
            state: seed.state,
            resource: seed.resource,
            bridge: seed.bridge,
            provider,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryDirectReadmissionRecoveryKind {
    BridgeCleanupFailed,
    ProviderRestorePanicked,
    ProviderRestoreRejectedAfterExecutionAdmission,
    RestoredExecutionReleaseRecoveryRequired,
    CheckpointReleasePanicked,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryDirectReadmissionRecoveryPosture {
    YieldReassemblyPending,
    TerminalCleanupRequired,
}

#[must_use = "direct readmission recovery must be explicitly resolved"]
pub enum WorthQueryDirectReadmissionRecoveryRequired {
    YieldReassembly(WorthQueryDirectReadmissionYieldReassemblyRecovery),
    TerminalCleanup(WorthQueryDirectReadmissionTerminalRecovery),
}

#[must_use = "yield reassembly recovery must retry Bridge cleanup or become terminal cleanup"]
pub struct WorthQueryDirectReadmissionYieldReassemblyRecovery {
    kind: WorthQueryDirectReadmissionRecoveryKind,
    detail: Arc<str>,
    progress: WorthQueryReadmissionProgress,
    recovery: WorthQueryDirectBridgeCleanupRecoveryState,
}

#[must_use = "provider recovery uncertainty can only enter terminal cleanup"]
pub struct WorthQueryDirectReadmissionTerminalRecovery {
    kind: WorthQueryDirectReadmissionRecoveryKind,
    detail: Arc<str>,
    progress: WorthQueryReadmissionProgress,
    recovery: WorthQueryDirectProviderRecoveryState,
}

#[must_use = "yield reassembly retains yielded or exact Bridge cleanup recovery authority"]
pub enum WorthQueryDirectReadmissionYieldReassemblyOutcome {
    Yielded(WorthQueryDirectReadmissionYieldReassembled),
    RecoveryRequired(WorthQueryDirectReadmissionYieldReassemblyRecovery),
}

pub struct WorthQueryDirectReadmissionYieldReassembled {
    yielded: WorthQueryYieldedDirectRun,
    evidence: WorthQueryReadmissionEvidence,
}

impl WorthQueryDirectReadmissionRecoveryRequired {
    pub(super) fn bridge_cleanup(
        detail: impl Into<Arc<str>>,
        progress: WorthQueryReadmissionProgress,
        recovery: WorthQueryDirectBridgeCleanupRecoveryState,
    ) -> Self {
        if let Some(custody) = recovery.affinity.custody() {
            custody.record_recovery(crate::domain_computation::primary_graph::query_obligations::QueryManagedRunRecovery::Readmission(WorthQueryDirectReadmissionRecoveryKind::BridgeCleanupFailed));
        }
        Self::YieldReassembly(WorthQueryDirectReadmissionYieldReassemblyRecovery {
            kind: WorthQueryDirectReadmissionRecoveryKind::BridgeCleanupFailed,
            detail: detail.into(),
            progress,
            recovery,
        })
    }

    pub(super) fn provider(
        kind: WorthQueryDirectReadmissionRecoveryKind,
        detail: impl Into<Arc<str>>,
        progress: WorthQueryReadmissionProgress,
        recovery: WorthQueryDirectProviderRecoveryState,
    ) -> Self {
        if let Some(custody) = recovery.resource.custody() {
            custody.record_recovery(crate::domain_computation::primary_graph::query_obligations::QueryManagedRunRecovery::Readmission(kind));
        }
        Self::TerminalCleanup(WorthQueryDirectReadmissionTerminalRecovery {
            kind,
            detail: detail.into(),
            progress,
            recovery,
        })
    }

    pub const fn kind(&self) -> WorthQueryDirectReadmissionRecoveryKind {
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

    pub const fn posture(&self) -> WorthQueryDirectReadmissionRecoveryPosture {
        match self {
            Self::YieldReassembly(_) => {
                WorthQueryDirectReadmissionRecoveryPosture::YieldReassemblyPending
            }
            Self::TerminalCleanup(_) => {
                WorthQueryDirectReadmissionRecoveryPosture::TerminalCleanupRequired
            }
        }
    }

    pub fn checkpoint(&self) -> &WorthQueryProviderCheckpointEvidence {
        match self {
            Self::YieldReassembly(recovery) => recovery.recovery.execution.checkpoint_evidence(),
            Self::TerminalCleanup(recovery) => recovery.recovery.provider.checkpoint_evidence(),
        }
    }

    pub fn checkpoint_release(&self) -> Option<&WorthQueryProviderCheckpointReleaseEvidence> {
        match self {
            Self::YieldReassembly(_) => None,
            Self::TerminalCleanup(recovery) => recovery.recovery.provider.checkpoint_release(),
        }
    }

    pub fn restored_execution_release_evidence(
        &self,
    ) -> Option<&WorthQueryProviderExecutionReleaseEvidence> {
        match self {
            Self::YieldReassembly(_) => None,
            Self::TerminalCleanup(recovery) => recovery
                .recovery
                .provider
                .restored_execution_release_evidence(),
        }
    }
}

impl WorthQueryDirectReadmissionYieldReassemblyRecovery {
    pub const fn kind(&self) -> WorthQueryDirectReadmissionRecoveryKind {
        self.kind
    }

    pub fn detail(&self) -> &str {
        &self.detail
    }

    pub const fn readmission_evidence(&self) -> WorthQueryReadmissionEvidence {
        self.progress.evidence()
    }

    pub fn retry_to_yielded(self) -> WorthQueryDirectReadmissionYieldReassemblyOutcome {
        let WorthQueryDirectBridgeCleanupRecoveryState {
            state,
            affinity,
            bridge,
            execution,
        } = self.recovery;
        retry_direct_bridge_cleanup(
            WorthQueryDirectYieldedReassembly {
                state,
                affinity,
                execution,
            },
            bridge.retry_cleanup(),
            self.progress,
        )
    }

    pub fn into_cleanup(self) -> WorthQueryDirectReadmissionCleanupRequired {
        WorthQueryDirectReadmissionCleanupRequired::bridge_recovery(self.recovery, self.progress)
    }
}

impl WorthQueryDirectReadmissionTerminalRecovery {
    pub const fn kind(&self) -> WorthQueryDirectReadmissionRecoveryKind {
        self.kind
    }

    pub fn detail(&self) -> &str {
        &self.detail
    }

    pub const fn readmission_evidence(&self) -> WorthQueryReadmissionEvidence {
        self.progress.evidence()
    }

    pub fn checkpoint(&self) -> &WorthQueryProviderCheckpointEvidence {
        self.recovery.provider.checkpoint_evidence()
    }

    pub fn checkpoint_release(&self) -> Option<&WorthQueryProviderCheckpointReleaseEvidence> {
        self.recovery.provider.checkpoint_release()
    }

    pub fn restored_execution_release_evidence(
        &self,
    ) -> Option<&WorthQueryProviderExecutionReleaseEvidence> {
        self.recovery.provider.restored_execution_release_evidence()
    }

    pub fn into_cleanup(self) -> WorthQueryDirectReadmissionCleanupRequired {
        WorthQueryDirectReadmissionCleanupRequired::provider(
            self.recovery.state,
            self.recovery.resource.abort(
                crate::domain_computation::managed_run::WorthQueryDirectReadmissionTransitionPermit::mint(),
            ),
            self.recovery.bridge,
            self.recovery.provider.into_cleanup(),
            self.progress,
        )
    }
}

impl WorthQueryDirectReadmissionYieldReassembled {
    pub const fn readmission_evidence(&self) -> WorthQueryReadmissionEvidence {
        self.evidence
    }

    pub fn into_yielded(self) -> WorthQueryYieldedDirectRun {
        self.yielded
    }
}

fn retry_direct_bridge_cleanup(
    pending: WorthQueryDirectYieldedReassembly,
    bridge: BridgeExecutionBasisReadmissionCleanupOutcome,
    mut progress: WorthQueryReadmissionProgress,
) -> WorthQueryDirectReadmissionYieldReassemblyOutcome {
    let WorthQueryDirectYieldedReassembly {
        state,
        affinity,
        execution,
    } = pending;
    match bridge {
        BridgeExecutionBasisReadmissionCleanupOutcome::Complete(returned) => {
            let (bridge, bridge_counters) = returned.into_parts();
            progress.observe_bridge(bridge_counters);
            WorthQueryDirectReadmissionYieldReassemblyOutcome::Yielded(
                WorthQueryDirectReadmissionYieldReassembled {
                    yielded: WorthQueryDirectYieldedParts {
                        state: WorthQueryDirectYieldedState {
                            affinity,
                            retained: state,
                        },
                        bridge,
                        execution,
                    }
                    .into_yielded(),
                    evidence: progress.evidence(),
                },
            )
        }
        BridgeExecutionBasisReadmissionCleanupOutcome::RecoveryRequired(bridge) => {
            let detail = bridge.detail().to_owned();
            progress.observe_bridge(bridge.counters());
            WorthQueryDirectReadmissionYieldReassemblyOutcome::RecoveryRequired(
                WorthQueryDirectReadmissionYieldReassemblyRecovery {
                    kind: WorthQueryDirectReadmissionRecoveryKind::BridgeCleanupFailed,
                    detail: detail.into(),
                    progress,
                    recovery: WorthQueryDirectBridgeCleanupRecoveryState {
                        state,
                        affinity,
                        execution,
                        bridge,
                    },
                },
            )
        }
    }
}
