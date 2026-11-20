use std::sync::Arc;

use super::WorthQueryManagedRelationalObservation;
use worth_runtime_bridge::facade::BridgeYieldedExecutionBasis;

mod terminal_cleanup;

pub use terminal_cleanup::{
    WorthQueryWorkflowYieldRecoveryCleanupInspection, WorthQueryWorkflowYieldRecoveryRelease,
    WorthQueryWorkflowYieldRecoveryReleaseOutcome, WorthQueryWorkflowYieldRecoveryReleasePending,
};

use self::terminal_cleanup::WorthQueryCompletedWorkflowYieldRecoveryCleanup;
use super::{
    WorthQueryManagedRunCounters, WorthQueryPausedWorkflowGraphExecution,
    WorthQueryWorkflowYieldOutcome, WorthQueryYieldRecoveryKind,
    WorthQueryYieldRecoveryResourceEvidence, WorthQueryYieldTransitionCounters,
};
use crate::domain_computation::artifact_owner::{
    WorthQueryFrozenWorkflowArtifactAuthority, WorthQueryWorkflowArtifactRegistryEvidence,
};

pub(super) enum WorthQueryWorkflowYieldRecoveryState {
    Running(WorthQueryPausedWorkflowGraphExecution),
    Terminalized(WorthQueryTerminalizedWorkflowYieldRecovery),
}

pub(super) struct WorthQueryTerminalizedWorkflowYieldRecovery {
    pub(super) affinity: super::workflow::WorthQueryWorkflowYieldReleasePending,
    pub(super) relational_basis: WorthQueryManagedRelationalObservation,
    pub(super) artifacts: WorthQueryFrozenWorkflowArtifactAuthority,
    pub(super) bridge: BridgeYieldedExecutionBasis,
    pub(super) run_counters: WorthQueryManagedRunCounters,
}

#[must_use = "workflow yield recovery retains paused or terminal-cleanup authority"]
pub struct WorthQueryWorkflowYieldRecoveryRequired {
    kind: WorthQueryYieldRecoveryKind,
    detail: Arc<str>,
    counters: WorthQueryYieldTransitionCounters,
    resource_evidence: WorthQueryYieldRecoveryResourceEvidence,
    state: WorthQueryWorkflowYieldRecoveryState,
}

impl WorthQueryWorkflowYieldRecoveryRequired {
    pub const fn kind(&self) -> WorthQueryYieldRecoveryKind {
        self.kind
    }

    pub fn detail(&self) -> &str {
        &self.detail
    }

    pub const fn counters(&self) -> WorthQueryYieldTransitionCounters {
        self.counters
    }

    pub fn resource_evidence(&self) -> &WorthQueryYieldRecoveryResourceEvidence {
        &self.resource_evidence
    }

    pub fn run_counters(&self) -> &WorthQueryManagedRunCounters {
        match &self.state {
            WorthQueryWorkflowYieldRecoveryState::Running(paused) => {
                paused.active.running.counters()
            }
            WorthQueryWorkflowYieldRecoveryState::Terminalized(state) => &state.run_counters,
        }
    }

    pub fn running_attempt_recoverable(&self) -> bool {
        matches!(self.state, WorthQueryWorkflowYieldRecoveryState::Running(_))
    }

    pub fn artifact_evidence(&self) -> Option<WorthQueryWorkflowArtifactRegistryEvidence> {
        match &self.state {
            WorthQueryWorkflowYieldRecoveryState::Running(_) => None,
            WorthQueryWorkflowYieldRecoveryState::Terminalized(state) => {
                Some(state.artifacts.registry().evidence())
            }
        }
    }

    #[must_use = "recovering a workflow yielded run returns the exact paused or recovery owner"]
    pub fn into_paused(self) -> Result<WorthQueryPausedWorkflowGraphExecution, Self> {
        match self.state {
            WorthQueryWorkflowYieldRecoveryState::Running(paused) => Ok(paused),
            state => Err(Self {
                kind: self.kind,
                detail: self.detail,
                counters: self.counters,
                resource_evidence: self.resource_evidence,
                state,
            }),
        }
    }

    #[must_use = "terminalized workflow yield cleanup returns an outcome or recovery owner"]
    pub fn release_terminalized(
        self,
    ) -> Result<WorthQueryWorkflowYieldRecoveryReleaseOutcome, Self> {
        let Self {
            kind,
            detail,
            counters,
            resource_evidence,
            state,
        } = self;
        let WorthQueryWorkflowYieldRecoveryState::Terminalized(state) = state else {
            return Err(Self {
                kind,
                detail,
                counters,
                resource_evidence,
                state,
            });
        };
        let registry = state.artifacts.registry();
        registry.close_cancelled();
        let artifact_evidence = registry.evidence();
        if artifact_evidence.retained_artifact_count() != 0
            || artifact_evidence.provider_release_pending_count() != 0
        {
            return Ok(WorthQueryWorkflowYieldRecoveryReleaseOutcome::Pending(
                WorthQueryWorkflowYieldRecoveryReleasePending::new(Self {
                    kind,
                    detail,
                    counters,
                    resource_evidence,
                    state: WorthQueryWorkflowYieldRecoveryState::Terminalized(state),
                }),
            ));
        }
        drop(state.artifacts);
        let recovery_required = artifact_evidence.provider_release_recovery_required_count() != 0;
        let release = WorthQueryWorkflowYieldRecoveryRelease::from_completed(
            WorthQueryCompletedWorkflowYieldRecoveryCleanup {
                affinity: state.affinity.release(),
                bridge: state.bridge.release(),
                relational: state.relational_basis.release(),
                artifact_evidence,
                run_counters: state.run_counters,
                yield_counters: counters,
                recovery_evidence: resource_evidence,
            },
            recovery_required,
        );
        Ok(if recovery_required {
            WorthQueryWorkflowYieldRecoveryReleaseOutcome::RecoveryRequired(release)
        } else {
            WorthQueryWorkflowYieldRecoveryReleaseOutcome::Complete(release)
        })
    }
}

pub(super) fn running_recovery(
    kind: WorthQueryYieldRecoveryKind,
    detail: Arc<str>,
    counters: WorthQueryYieldTransitionCounters,
    paused: WorthQueryPausedWorkflowGraphExecution,
) -> WorthQueryWorkflowYieldOutcome {
    WorthQueryWorkflowYieldOutcome::RecoveryRequired(WorthQueryWorkflowYieldRecoveryRequired {
        kind,
        detail,
        counters,
        resource_evidence: WorthQueryYieldRecoveryResourceEvidence::default(),
        state: WorthQueryWorkflowYieldRecoveryState::Running(paused),
    })
}

pub(super) fn terminalized_recovery(
    kind: WorthQueryYieldRecoveryKind,
    detail: Arc<str>,
    counters: WorthQueryYieldTransitionCounters,
    state: WorthQueryTerminalizedWorkflowYieldRecovery,
    resource_evidence: WorthQueryYieldRecoveryResourceEvidence,
) -> WorthQueryWorkflowYieldOutcome {
    WorthQueryWorkflowYieldOutcome::RecoveryRequired(WorthQueryWorkflowYieldRecoveryRequired {
        kind,
        detail,
        counters,
        resource_evidence,
        state: WorthQueryWorkflowYieldRecoveryState::Terminalized(state),
    })
}
