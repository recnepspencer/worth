use super::*;

impl WorthQueryWorkflowBridgeRecoveryAssociation {
    pub fn checkpoint_evidence(
        &self,
    ) -> &crate::domain_computation::WorthQueryProviderCheckpointEvidence {
        self.execution.checkpoint_evidence()
    }

    pub fn owner_retry_cleanup(
        self,
        _owner: &WorthQueryWorkflowReadmissionRecoveryPermit,
    ) -> WorthQueryWorkflowBridgeRecoveryTransition {
        match self.bridge.retry_cleanup() {
            worth_runtime_bridge::facade::BridgeExecutionBasisReadmissionCleanupOutcome::Complete(
                returned,
            ) => {
                let (bridge, counters) = returned.into_parts();
                WorthQueryWorkflowBridgeRecoveryOutcome::Yielded {
                    yielded: self
                        .state
                        .restore_yielded(self.affinity, bridge, self.execution),
                    counters,
                }
            }
            worth_runtime_bridge::facade::BridgeExecutionBasisReadmissionCleanupOutcome::RecoveryRequired(
                bridge,
            ) => WorthQueryWorkflowBridgeRecoveryOutcome::RecoveryRequired {
                detail: Arc::from(bridge.detail()),
                counters: bridge.counters(),
                association: Self {
                    state: self.state,
                    affinity: self.affinity,
                    bridge,
                    execution: self.execution,
                },
            },
        }
        .into()
    }
}

pub struct WorthQueryWorkflowBridgeRecoveryTransition
{
    outcome: WorthQueryWorkflowBridgeRecoveryOutcome,
}

enum WorthQueryWorkflowBridgeRecoveryOutcome {
    Yielded {
        yielded: WorthQueryYieldedWorkflowRun,
        counters: worth_runtime_bridge::facade::BridgeExecutionBasisReadmissionCounters,
    },
    RecoveryRequired {
        association: WorthQueryWorkflowBridgeRecoveryAssociation,
        detail: Arc<str>,
        counters: worth_runtime_bridge::facade::BridgeExecutionBasisReadmissionCounters,
    },
}

impl From<WorthQueryWorkflowBridgeRecoveryOutcome> for WorthQueryWorkflowBridgeRecoveryTransition {
    fn from(outcome: WorthQueryWorkflowBridgeRecoveryOutcome) -> Self {
        Self { outcome }
    }
}

impl WorthQueryWorkflowBridgeRecoveryTransition {
    pub fn owner_resolve_retry(
        self,
        mut progress: crate::domain_computation::managed_run::readmission::evidence::WorthQueryReadmissionProgress,
        _owner: &WorthQueryWorkflowReadmissionRecoveryPermit,
    ) -> crate::domain_computation::managed_run::readmission::workflow_recovery::WorthQueryWorkflowReadmissionYieldReassemblyOutcome{
        match self.outcome {
            WorthQueryWorkflowBridgeRecoveryOutcome::Yielded { yielded, counters } => {
                progress.observe_bridge(counters);
                crate::domain_computation::managed_run::readmission::workflow_recovery::owner_retry_yielded(yielded, progress)
            }
            WorthQueryWorkflowBridgeRecoveryOutcome::RecoveryRequired {
                association,
                detail,
                counters,
            } => {
                progress.observe_bridge(counters);
                crate::domain_computation::managed_run::readmission::workflow_recovery::owner_retry_required(
                    association,
                    detail,
                    progress,
                )
            }
        }
    }
}
