use std::sync::Arc;

use super::WorthQueryManagedRelationalObservation;
use worth_runtime_bridge::facade::BridgeBoundExecutionBasis;

use super::{
    WorthQueryManagedGraphCallRequest, WorthQueryManagedRunCounters, WorthQueryManagedRunDenial,
    WorthQueryManagedRunDenialKind, WorthQueryManagedRunTerminalKind,
    WorthQueryManagedSafePointFailure, WorthQueryManagedSafePointObservation,
    WorthQueryManagedWorkflowArtifactAuthority,
};
use crate::domain_computation::artifact_owner::{
    WorthQueryArtifactOccurrenceLedger, WorthQueryWorkflowArtifactAuthority,
    WorthQueryWorkflowArtifactRegistryEvidence,
};
use crate::domain_computation::{
    WorthQueryArtifactDenial, WorthQueryArtifactDenialKind,
    WorthQueryExecutionBoundOperationAuthority, WorthQueryExecutionResourceAttemptEvidence,
    WorthQueryGraphProviderCallRequest, WorthQueryWorkflowExecutionResourceAttempt,
};

mod provider_plan_admission;
mod run_affinity;
mod running_observations;
mod running_operations;
mod terminal;
#[path = "workflow_yield_freeze.rs"]
pub(in crate::domain_computation::managed_run) mod yield_freeze;

pub(in crate::domain_computation) use provider_plan_admission::WorthQueryWorkflowProviderPlanPermit;
pub(in crate::domain_computation) use run_affinity::WorthQueryWorkflowRunTransitionPermit;
pub(super) use run_affinity::{
    WorthQueryWorkflowAffinityCleanupReceipt, WorthQueryWorkflowRunAffinity,
    WorthQueryWorkflowRunProviderRestoreOutcome, WorthQueryWorkflowRunReadmissionPending,
    WorthQueryWorkflowRunRestoredPending, WorthQueryWorkflowYieldReleasePending,
};
pub use terminal::{
    WorthQueryWorkflowRunCleanupFailure, WorthQueryWorkflowRunCleanupInspection,
    WorthQueryWorkflowRunCleanupOutcome, WorthQueryWorkflowRunCleanupPending,
    WorthQueryWorkflowRunCleanupReceipt, WorthQueryWorkflowRunTerminal,
};

pub struct WorthQueryAdmittedWorkflowRun {
    affinity: WorthQueryWorkflowRunAffinity,
    bridge_basis: BridgeBoundExecutionBasis,
    relational_basis: WorthQueryManagedRelationalObservation,
    counters: WorthQueryManagedRunCounters,
}

impl WorthQueryAdmittedWorkflowRun {
    pub(in crate::domain_computation) fn new(
        _operation: &WorthQueryExecutionBoundOperationAuthority,
        resource_attempt: WorthQueryWorkflowExecutionResourceAttempt,
        bridge_basis: BridgeBoundExecutionBasis,
        relational_basis: WorthQueryManagedRelationalObservation,
        counters: WorthQueryManagedRunCounters,
    ) -> Self {
        Self {
            affinity: WorthQueryWorkflowRunAffinity::initial(resource_attempt),
            bridge_basis,
            relational_basis,
            counters,
        }
    }

    pub fn identity(&self) -> &str {
        self.affinity.attempt_identity()
    }

    pub fn logical_run_identity(&self) -> &str {
        self.affinity.logical_identity()
    }

    pub fn counters(&self) -> &WorthQueryManagedRunCounters {
        &self.counters
    }

    pub(crate) fn belongs_to_operation(
        &self,
        operation: &WorthQueryExecutionBoundOperationAuthority,
    ) -> bool {
        self.affinity.belongs_to_operation(operation)
    }

    pub fn start(
        self,
    ) -> Result<WorthQueryRunningWorkflowRun, WorthQueryWorkflowRunStartRejection> {
        let artifacts = match self.affinity.bind_managed_workflow_artifacts() {
            Ok(artifacts) => artifacts,
            Err(denial) => {
                return Err(WorthQueryWorkflowRunStartRejection {
                    denial,
                    admitted: self,
                    artifacts: None,
                });
            }
        };
        self.finish_start(artifacts)
    }

    #[doc(hidden)]
    pub fn start_with_artifacts(
        self,
        artifacts: WorthQueryWorkflowArtifactAuthority,
    ) -> Result<WorthQueryRunningWorkflowRun, WorthQueryWorkflowRunStartRejection> {
        if !artifacts.belongs_to_binding(self.affinity.binding_authority()) {
            return Err(WorthQueryWorkflowRunStartRejection {
                denial: WorthQueryArtifactDenial::new(
                    WorthQueryArtifactDenialKind::RunMismatch,
                    None,
                    "managed workflow artifacts belong to a different operation binding",
                ),
                admitted: self,
                artifacts: Some(artifacts),
            });
        }
        self.finish_start(artifacts)
    }

    fn finish_start(
        self,
        artifacts: WorthQueryWorkflowArtifactAuthority,
    ) -> Result<WorthQueryRunningWorkflowRun, WorthQueryWorkflowRunStartRejection> {
        let provider_artifact_occurrences = Arc::new(WorthQueryArtifactOccurrenceLedger::default());
        Ok(WorthQueryRunningWorkflowRun {
            affinity: self.affinity,
            bridge_basis: self.bridge_basis,
            relational_basis: self.relational_basis,
            counters: self.counters,
            artifacts,
            provider_artifact_occurrences,
        })
    }

    pub fn release(
        self,
    ) -> crate::domain_computation::WorthQueryWorkflowExecutionAttemptReleaseReceipt {
        self.affinity.release_initial()
    }
}

pub struct WorthQueryWorkflowRunStartRejection {
    denial: WorthQueryArtifactDenial,
    admitted: WorthQueryAdmittedWorkflowRun,
    artifacts: Option<WorthQueryWorkflowArtifactAuthority>,
}

impl WorthQueryWorkflowRunStartRejection {
    pub fn denial(&self) -> &WorthQueryArtifactDenial {
        &self.denial
    }

    pub fn into_admitted(self) -> WorthQueryAdmittedWorkflowRun {
        if let Some(artifacts) = self.artifacts {
            artifacts.registry().close_cancelled();
        }
        self.admitted
    }

    pub fn release(
        self,
    ) -> crate::domain_computation::WorthQueryWorkflowExecutionAttemptReleaseReceipt {
        if let Some(artifacts) = self.artifacts {
            artifacts.registry().close_cancelled();
        }
        self.admitted.release()
    }
}

impl std::fmt::Debug for WorthQueryWorkflowRunStartRejection {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("WorthQueryWorkflowRunStartRejection")
            .field("denial", &self.denial)
            .field("run_identity", &self.admitted.identity())
            .finish()
    }
}

pub struct WorthQueryRunningWorkflowRun {
    affinity: WorthQueryWorkflowRunAffinity,
    bridge_basis: BridgeBoundExecutionBasis,
    relational_basis: WorthQueryManagedRelationalObservation,
    counters: WorthQueryManagedRunCounters,
    artifacts: WorthQueryWorkflowArtifactAuthority,
    provider_artifact_occurrences: Arc<WorthQueryArtifactOccurrenceLedger>,
}

impl WorthQueryRunningWorkflowRun {
    pub fn abandon(self) -> WorthQueryWorkflowRunTerminal {
        self.terminal(WorthQueryManagedRunTerminalKind::Failed)
    }

    pub(in crate::domain_computation::managed_run) fn owner_restore_readmission(
        affinity: WorthQueryWorkflowRunAffinity,
        bridge_basis: BridgeBoundExecutionBasis,
        relational_basis: WorthQueryManagedRelationalObservation,
        counters: WorthQueryManagedRunCounters,
        artifacts: WorthQueryWorkflowArtifactAuthority,
        provider_artifact_occurrences: Arc<WorthQueryArtifactOccurrenceLedger>,
        _owner: &WorthQueryWorkflowRunTransitionPermit,
    ) -> Self {
        Self {
            affinity,
            bridge_basis,
            relational_basis,
            counters,
            artifacts,
            provider_artifact_occurrences,
        }
    }

    pub fn completed(
        self,
    ) -> Result<WorthQueryWorkflowRunTerminal, WorthQueryWorkflowRunCompletionRejection> {
        if self.affinity.provider_work_has_uncertainty() {
            return Err(WorthQueryWorkflowRunCompletionRejection {
                denial: WorthQueryManagedRunDenial::new(
                    WorthQueryManagedRunDenialKind::UnverifiedProviderWork,
                    "workflow provider work must be receipt-bound before completion",
                    self.counters.clone(),
                ),
                running: self,
            });
        }
        Ok(self.terminal(WorthQueryManagedRunTerminalKind::Completed))
    }

    pub(super) fn terminal(
        self,
        kind: WorthQueryManagedRunTerminalKind,
    ) -> WorthQueryWorkflowRunTerminal {
        WorthQueryWorkflowRunTerminal::from_running(self, kind)
    }

    pub(crate) fn terminate_for_convergence(
        self,
        kind: WorthQueryManagedRunTerminalKind,
    ) -> WorthQueryWorkflowRunTerminal {
        self.terminal(kind)
    }
}

pub struct WorthQueryWorkflowRunCompletionRejection {
    denial: WorthQueryManagedRunDenial,
    running: WorthQueryRunningWorkflowRun,
}

impl WorthQueryWorkflowRunCompletionRejection {
    pub fn denial(&self) -> &WorthQueryManagedRunDenial {
        &self.denial
    }

    pub fn into_running(self) -> WorthQueryRunningWorkflowRun {
        self.running
    }
}

impl std::fmt::Debug for WorthQueryWorkflowRunCompletionRejection {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("WorthQueryWorkflowRunCompletionRejection")
            .field("denial", &self.denial)
            .field("run_identity", &self.running.identity())
            .finish()
    }
}
