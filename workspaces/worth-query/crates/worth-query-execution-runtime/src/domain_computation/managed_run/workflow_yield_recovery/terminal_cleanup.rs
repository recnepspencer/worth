use std::sync::Arc;

use super::super::WorthQueryManagedRelationalObservationReleaseReceipt;
use worth_query_admission::facade::resource_admission::WorthQueryExecutionCapacityReservationScope;
use worth_runtime_bridge::facade::{
    BridgeExecutionBasisFinalizationReceipt, BridgeExecutionBasisSignalTerminal,
};

use super::WorthQueryWorkflowYieldRecoveryRequired;
use crate::domain_computation::artifact_owner::WorthQueryWorkflowArtifactRegistryEvidence;
use crate::domain_computation::managed_run::{
    workflow::WorthQueryWorkflowAffinityCleanupReceipt, WorthQueryManagedProviderWorkEvidence,
    WorthQueryManagedRunCleanupDisposition, WorthQueryManagedRunCounters,
    WorthQueryYieldCleanupCheckpointInspection, WorthQueryYieldRecoveryResourceEvidence,
    WorthQueryYieldTransitionCounters,
};

#[must_use = "workflow terminalized-yield cleanup outcomes must be resolved"]
pub enum WorthQueryWorkflowYieldRecoveryReleaseOutcome {
    Complete(WorthQueryWorkflowYieldRecoveryRelease),
    Pending(WorthQueryWorkflowYieldRecoveryReleasePending),
    RecoveryRequired(WorthQueryWorkflowYieldRecoveryRelease),
}

#[must_use = "pending workflow terminalized-yield cleanup retains exact recovery authority"]
pub struct WorthQueryWorkflowYieldRecoveryReleasePending {
    recovery: WorthQueryWorkflowYieldRecoveryRequired,
}

impl WorthQueryWorkflowYieldRecoveryReleasePending {
    pub(super) fn new(recovery: WorthQueryWorkflowYieldRecoveryRequired) -> Self {
        Self { recovery }
    }

    #[must_use = "retry returns the same workflow terminalized-yield cleanup authority"]
    pub fn retry(
        self,
    ) -> Result<
        WorthQueryWorkflowYieldRecoveryReleaseOutcome,
        WorthQueryWorkflowYieldRecoveryRequired,
    > {
        self.recovery.release_terminalized()
    }
}

pub(super) struct WorthQueryCompletedWorkflowYieldRecoveryCleanup {
    pub(super) affinity: WorthQueryWorkflowAffinityCleanupReceipt,
    pub(super) bridge: BridgeExecutionBasisFinalizationReceipt,
    pub(super) relational: WorthQueryManagedRelationalObservationReleaseReceipt,
    pub(super) artifact_evidence: WorthQueryWorkflowArtifactRegistryEvidence,
    pub(super) run_counters: WorthQueryManagedRunCounters,
    pub(super) yield_counters: WorthQueryYieldTransitionCounters,
    pub(super) recovery_evidence: WorthQueryYieldRecoveryResourceEvidence,
}

#[derive(Clone, Debug, PartialEq)]
pub struct WorthQueryWorkflowYieldRecoveryRelease {
    inspection: WorthQueryWorkflowYieldRecoveryCleanupInspection,
}

#[derive(Clone, Debug, PartialEq)]
pub struct WorthQueryWorkflowYieldRecoveryCleanupInspection {
    logical_run_identity: Arc<str>,
    attempt_identity: Arc<str>,
    disposition: WorthQueryManagedRunCleanupDisposition,
    checkpoint: Option<WorthQueryYieldCleanupCheckpointInspection>,
    bridge_reservation_released: bool,
    bridge_signal_terminal: BridgeExecutionBasisSignalTerminal,
    bridge_signal_transition_performed: bool,
    relational_basis_released: bool,
    provider_session_identity: Arc<str>,
    resource_plan_identity: Arc<str>,
    capacity_scope: WorthQueryExecutionCapacityReservationScope,
    released_reservation_count: usize,
    artifact_evidence: WorthQueryWorkflowArtifactRegistryEvidence,
    run_counters: WorthQueryManagedRunCounters,
    provider_work: WorthQueryManagedProviderWorkEvidence,
    yield_counters: WorthQueryYieldTransitionCounters,
}

impl WorthQueryWorkflowYieldRecoveryRelease {
    pub(super) fn from_completed(
        completed: WorthQueryCompletedWorkflowYieldRecoveryCleanup,
        recovery_required: bool,
    ) -> Self {
        let attempt = completed.affinity.attempt();
        let capacity = attempt.capacity();
        let checkpoint = completed
            .recovery_evidence
            .checkpoint_release()
            .or_else(|| {
                completed
                    .recovery_evidence
                    .provider_checkpoint_failure()
                    .and_then(|failure| failure.checkpoint_release())
            })
            .map(WorthQueryYieldCleanupCheckpointInspection::capture);
        let inspection = WorthQueryWorkflowYieldRecoveryCleanupInspection {
            logical_run_identity: Arc::from(completed.affinity.logical_run_identity()),
            attempt_identity: Arc::from(completed.affinity.yielded_attempt_identity()),
            disposition: if recovery_required {
                WorthQueryManagedRunCleanupDisposition::RecoveryRequired
            } else {
                WorthQueryManagedRunCleanupDisposition::CleanupComplete
            },
            checkpoint,
            bridge_reservation_released: completed.bridge.reservation_released(),
            bridge_signal_terminal: completed.bridge.signal_terminal(),
            bridge_signal_transition_performed: completed.bridge.signal_transition_performed(),
            relational_basis_released: completed.relational.released(),
            provider_session_identity: Arc::from(attempt.provider_session_identity()),
            resource_plan_identity: Arc::from(capacity.resource_plan_identity()),
            capacity_scope: capacity.scope(),
            released_reservation_count: capacity.released_reservation_count(),
            artifact_evidence: completed.artifact_evidence,
            run_counters: completed.run_counters,
            provider_work: completed.affinity.provider_work().clone(),
            yield_counters: completed.yield_counters,
        };
        Self { inspection }
    }

    pub const fn inspection(&self) -> &WorthQueryWorkflowYieldRecoveryCleanupInspection {
        &self.inspection
    }
}

impl WorthQueryWorkflowYieldRecoveryCleanupInspection {
    pub fn logical_run_identity(&self) -> &str {
        &self.logical_run_identity
    }
    pub fn yielded_attempt_identity(&self) -> &str {
        &self.attempt_identity
    }
    pub const fn disposition(&self) -> WorthQueryManagedRunCleanupDisposition {
        self.disposition
    }
    pub const fn checkpoint(&self) -> Option<&WorthQueryYieldCleanupCheckpointInspection> {
        self.checkpoint.as_ref()
    }
    pub fn provider_session_identity(&self) -> &str {
        &self.provider_session_identity
    }
    pub fn resource_plan_identity(&self) -> &str {
        &self.resource_plan_identity
    }
    pub const fn capacity_scope(&self) -> WorthQueryExecutionCapacityReservationScope {
        self.capacity_scope
    }
    pub const fn released_reservation_count(&self) -> usize {
        self.released_reservation_count
    }
    pub const fn bridge_signal_terminal(&self) -> BridgeExecutionBasisSignalTerminal {
        self.bridge_signal_terminal
    }
    pub const fn bridge_signal_transition_performed(&self) -> bool {
        self.bridge_signal_transition_performed
    }
    pub const fn artifact_evidence(&self) -> WorthQueryWorkflowArtifactRegistryEvidence {
        self.artifact_evidence
    }
    pub const fn resources_released(&self) -> bool {
        self.bridge_reservation_released
            && self.relational_basis_released
            && self.released_reservation_count != 0
            && self.artifact_evidence.retained_artifact_count() == 0
            && self.artifact_evidence.provider_release_pending_count() == 0
    }
    pub const fn provider_work(&self) -> &WorthQueryManagedProviderWorkEvidence {
        &self.provider_work
    }
    pub const fn run_counters(&self) -> &WorthQueryManagedRunCounters {
        &self.run_counters
    }
    pub const fn yield_counters(&self) -> WorthQueryYieldTransitionCounters {
        self.yield_counters
    }
}
