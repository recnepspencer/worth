use std::sync::Arc;

use super::WorthQueryManagedRelationalObservation;
use worth_runtime_bridge::facade::{BridgeBoundExecutionBasis, BridgeExecutionBasisSignalTerminal};

use super::super::managed_graph_execution::WorthQueryManagedGraphExecution;
use super::super::workflow_yield_eligibility::classify_workflow_retained_bytes_denial;
use super::super::workflow_yield_recovery::running_recovery;
use super::super::workflow_yield_transition::WorthQueryWorkflowYieldCheckpointPending;
use super::super::yield_eligibility::WorthQueryManagedYieldSafePoint;
use {crate::domain_computation::vocabulary::managed_run::counters::WorthQueryManagedRunCounters, super::super::WorthQueryPausedWorkflowGraphExecution, super::super::WorthQueryRunningWorkflowRun, super::super::WorthQueryWorkflowYieldDenied, super::super::WorthQueryWorkflowYieldOutcome, super::super::WorthQueryYieldRecoveryKind, super::super::WorthQueryYieldTransitionCounters};
use super::WorthQueryWorkflowRunAffinity;
use crate::domain_computation::artifact_owner::{
    WorthQueryArtifactOccurrenceLedger, WorthQueryWorkflowArtifactAuthority,
    WorthQueryWorkflowArtifactFreezePending,
};

pub(in crate::domain_computation::managed_run) fn freeze_and_finalize_workflow_yield(
    paused: WorthQueryPausedWorkflowGraphExecution,
    counters: WorthQueryYieldTransitionCounters,
) -> Result<WorthQueryWorkflowYieldCheckpointPending, WorthQueryWorkflowYieldOutcome> {
    WorthQueryWorkflowYieldBridgePending::from_paused(paused, counters)
        .prepare_artifact_freeze()?
        .finalize_bridge()
}

struct WorthQueryWorkflowYieldBridgePending {
    affinity: WorthQueryWorkflowRunAffinity,
    bridge_basis: BridgeBoundExecutionBasis,
    relational_basis: WorthQueryManagedRelationalObservation,
    run_counters: WorthQueryManagedRunCounters,
    artifacts: WorthQueryWorkflowArtifactAuthority,
    provider_artifact_occurrences: Arc<WorthQueryArtifactOccurrenceLedger>,
    execution: WorthQueryManagedGraphExecution,
    safe_point: WorthQueryManagedYieldSafePoint,
    yield_counters: WorthQueryYieldTransitionCounters,
}

impl WorthQueryWorkflowYieldBridgePending {
    fn from_paused(
        paused: WorthQueryPausedWorkflowGraphExecution,
        yield_counters: WorthQueryYieldTransitionCounters,
    ) -> Self {
        let WorthQueryPausedWorkflowGraphExecution { active, safe_point } = paused;
        let super::super::WorthQueryActiveWorkflowGraphExecution { running, execution } = active;
        let WorthQueryRunningWorkflowRun {
            affinity,
            bridge_basis,
            relational_basis,
            counters,
            artifacts,
            provider_artifact_occurrences,
        } = running;
        Self {
            affinity,
            bridge_basis,
            relational_basis,
            run_counters: counters,
            artifacts,
            provider_artifact_occurrences,
            execution,
            safe_point,
            yield_counters,
        }
    }

    fn prepare_artifact_freeze(
        mut self,
    ) -> Result<WorthQueryWorkflowYieldArtifactFreezePending, WorthQueryWorkflowYieldOutcome> {
        let artifacts = match WorthQueryWorkflowArtifactFreezePending::prepare(self.artifacts) {
            Ok(pending) => pending,
            Err((artifacts, denial)) => {
                self.artifacts = artifacts;
                return Err(running_recovery(
                    WorthQueryYieldRecoveryKind::ArtifactProductionFreeze(denial.kind()),
                    Arc::from(denial.detail()),
                    self.yield_counters,
                    self.into_paused(),
                ));
            }
        };
        self.yield_counters.observed_artifact_registry();
        let artifact_evidence = artifacts.evidence();
        self.yield_counters.validated_retained_resources();
        let ceiling = self
            .affinity
            .operation_resources()
            .envelope()
            .yield_contract()
            .expect("eligibility established the installed yield contract")
            .retained_bytes_ceiling();
        if let Some((kind, detail)) = classify_workflow_retained_bytes_denial(
            self.safe_point.retained().provider_bytes(),
            artifact_evidence.retained_bytes(),
            ceiling,
        ) {
            self.artifacts = artifacts.abort();
            let counters = self.yield_counters;
            return Err(WorthQueryWorkflowYieldOutcome::Denied(
                WorthQueryWorkflowYieldDenied {
                    kind,
                    detail: Arc::from(detail),
                    paused: self.into_paused(),
                    counters,
                },
            ));
        }
        Ok(WorthQueryWorkflowYieldArtifactFreezePending {
            affinity: self.affinity,
            bridge_basis: self.bridge_basis,
            relational_basis: self.relational_basis,
            run_counters: self.run_counters,
            artifacts,
            provider_artifact_occurrences: self.provider_artifact_occurrences,
            execution: self.execution,
            safe_point: self.safe_point,
            yield_counters: self.yield_counters,
        })
    }

    fn into_paused(self) -> WorthQueryPausedWorkflowGraphExecution {
        WorthQueryPausedWorkflowGraphExecution {
            active: super::super::WorthQueryActiveWorkflowGraphExecution {
                running: WorthQueryRunningWorkflowRun {
                    affinity: self.affinity,
                    bridge_basis: self.bridge_basis,
                    relational_basis: self.relational_basis,
                    counters: self.run_counters,
                    artifacts: self.artifacts,
                    provider_artifact_occurrences: self.provider_artifact_occurrences,
                },
                execution: self.execution,
            },
            safe_point: self.safe_point,
        }
    }
}

struct WorthQueryWorkflowYieldArtifactFreezePending {
    affinity: WorthQueryWorkflowRunAffinity,
    bridge_basis: BridgeBoundExecutionBasis,
    relational_basis: WorthQueryManagedRelationalObservation,
    run_counters: WorthQueryManagedRunCounters,
    artifacts: WorthQueryWorkflowArtifactFreezePending,
    provider_artifact_occurrences: Arc<WorthQueryArtifactOccurrenceLedger>,
    execution: WorthQueryManagedGraphExecution,
    safe_point: WorthQueryManagedYieldSafePoint,
    yield_counters: WorthQueryYieldTransitionCounters,
}

impl WorthQueryWorkflowYieldArtifactFreezePending {
    fn finalize_bridge(
        mut self,
    ) -> Result<WorthQueryWorkflowYieldCheckpointPending, WorthQueryWorkflowYieldOutcome> {
        self.yield_counters.attempted_bridge_finalization();
        let bridge = match self.bridge_basis.yield_execution_basis() {
            Ok(receipt) => receipt,
            Err(failure) => {
                let kind = failure.kind();
                let detail = Arc::from(failure.detail());
                let artifacts = self.artifacts.abort();
                let running = WorthQueryRunningWorkflowRun {
                    affinity: self.affinity,
                    bridge_basis: failure.into_basis(),
                    relational_basis: self.relational_basis,
                    counters: self.run_counters,
                    artifacts,
                    provider_artifact_occurrences: self.provider_artifact_occurrences,
                };
                return Err(running_recovery(
                    WorthQueryYieldRecoveryKind::BridgeTerminalization(kind),
                    detail,
                    self.yield_counters,
                    WorthQueryPausedWorkflowGraphExecution {
                        active: super::super::WorthQueryActiveWorkflowGraphExecution {
                            running,
                            execution: self.execution,
                        },
                        safe_point: self.safe_point,
                    },
                ));
            }
        };
        let artifacts = self.artifacts.commit();
        let mut pending = WorthQueryWorkflowYieldCheckpointPending {
            affinity: self.affinity,
            relational_basis: self.relational_basis,
            run_counters: self.run_counters,
            artifacts,
            provider_artifact_occurrences: self.provider_artifact_occurrences,
            execution: self.execution,
            bridge,
            yield_counters: self.yield_counters,
        };
        if !pending.bridge.receipt().signal_transition_performed()
            || pending.bridge.receipt().signal_terminal()
                != BridgeExecutionBasisSignalTerminal::Cancelled
        {
            pending.affinity.interrupt_provider_step_call();
            let terminal = pending.bridge.receipt().signal_terminal();
            return Err(pending.recovery(
                WorthQueryYieldRecoveryKind::SignalAttemptAlreadyTerminal(terminal),
                Arc::from(
                    "Signal attempt terminalized before Query could perform the yield transition",
                ),
            ));
        }
        Ok(pending)
    }
}
