use std::sync::Arc;

use crate::domain_computation::managed_run::WorthQueryManagedRelationalObservationReleaseReceipt;
use worth_query_admission::facade::resource_admission::WorthQueryExecutionCapacityReservationScope;
use worth_runtime_bridge::facade::{
    BridgeExecutionBasisFinalizationReceipt, BridgeExecutionBasisSignalTerminal,
};

use super::{
    WorthQueryWorkflowReadmissionCleanupReceipt, WorthQueryWorkflowReadmissionPartialCleanupReceipt,
};
use crate::domain_computation::artifact_owner::{
    WorthQueryArtifactDenialKind, WorthQueryArtifactProductionGenerationAbortFailure,
    WorthQueryWorkflowArtifactRegistryEvidence,
};
use crate::domain_computation::managed_run::readmission::{
    WorthQueryReadmissionCleanupCheckpointInspection,
    WorthQueryReadmissionRestoredExecutionCleanupInspection,
};
use crate::domain_computation::managed_run::workflow::WorthQueryWorkflowAffinityCleanupReceipt;
use crate::domain_computation::managed_run::{
    WorthQueryManagedProviderWorkEvidence, WorthQueryManagedRunCleanupDisposition,
    WorthQueryManagedRunCounters, WorthQueryYieldTransitionCounters,
};
use crate::domain_computation::provider_session::graph_provider::bounded_step::WorthQueryProviderExecutionReleaseEvidence;
use crate::domain_computation::{
    WorthQueryProviderCheckpointReleaseEvidence, WorthQueryReadmissionEvidence,
};

pub(super) struct WorthQueryCompletedWorkflowReadmissionCleanup {
    pub(super) affinity: WorthQueryWorkflowAffinityCleanupReceipt,
    pub(super) disposition: WorthQueryManagedRunCleanupDisposition,
    pub(super) checkpoint_release: WorthQueryProviderCheckpointReleaseEvidence,
    pub(super) restored_execution_release: Option<WorthQueryProviderExecutionReleaseEvidence>,
    pub(super) bridge: BridgeExecutionBasisFinalizationReceipt,
    pub(super) relational: WorthQueryManagedRelationalObservationReleaseReceipt,
    pub(super) artifact_evidence: WorthQueryWorkflowArtifactRegistryEvidence,
    pub(super) generation_rollback: Option<WorthQueryArtifactGenerationRollbackEvidence>,
    pub(super) run_counters: WorthQueryManagedRunCounters,
    pub(super) yield_counters: WorthQueryYieldTransitionCounters,
    pub(super) readmission_evidence: WorthQueryReadmissionEvidence,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryArtifactGenerationRollbackEvidence {
    prior_generation: u64,
    pending_generation: u64,
    denial_kind: WorthQueryArtifactDenialKind,
    detail: &'static str,
}

#[derive(Clone, Debug, PartialEq)]
pub struct WorthQueryWorkflowReadmissionCleanupInspection {
    logical_run_identity: Arc<str>,
    yielded_attempt_identity: Arc<str>,
    disposition: WorthQueryManagedRunCleanupDisposition,
    checkpoint: WorthQueryReadmissionCleanupCheckpointInspection,
    restored_execution: Option<WorthQueryReadmissionRestoredExecutionCleanupInspection>,
    bridge_reservation_released: bool,
    bridge_signal_terminal: BridgeExecutionBasisSignalTerminal,
    bridge_signal_transition_performed: bool,
    relational_basis_released: bool,
    provider_session_identity: Arc<str>,
    resource_plan_identity: Arc<str>,
    capacity_scope: WorthQueryExecutionCapacityReservationScope,
    released_reservation_count: usize,
    artifact_evidence: WorthQueryWorkflowArtifactRegistryEvidence,
    generation_rollback: Option<WorthQueryArtifactGenerationRollbackEvidence>,
    run_counters: WorthQueryManagedRunCounters,
    provider_work: WorthQueryManagedProviderWorkEvidence,
    yield_counters: WorthQueryYieldTransitionCounters,
    readmission_evidence: WorthQueryReadmissionEvidence,
}

#[derive(Clone, Debug, PartialEq)]
pub struct WorthQueryWorkflowReadmissionCleanupPendingInspection {
    logical_run_identity: Arc<str>,
    yielded_attempt_identity: Arc<str>,
    checkpoint: WorthQueryReadmissionCleanupCheckpointInspection,
    restored_execution: Option<WorthQueryReadmissionRestoredExecutionCleanupInspection>,
    provider_session_identity: Arc<str>,
    resource_plan_identity: Arc<str>,
    capacity_scope: WorthQueryExecutionCapacityReservationScope,
    released_reservation_count: usize,
    artifact_evidence: WorthQueryWorkflowArtifactRegistryEvidence,
    generation_rollback: Option<WorthQueryArtifactGenerationRollbackEvidence>,
    artifact_cleanup_pending: bool,
    bridge_cleanup_pending: bool,
    run_counters: WorthQueryManagedRunCounters,
    provider_work: WorthQueryManagedProviderWorkEvidence,
    yield_counters: WorthQueryYieldTransitionCounters,
    readmission_evidence: WorthQueryReadmissionEvidence,
}

impl WorthQueryWorkflowReadmissionCleanupReceipt {
    pub(super) fn from_completed(completed: WorthQueryCompletedWorkflowReadmissionCleanup) -> Self {
        let attempt = completed.affinity.attempt();
        let capacity = attempt.capacity();
        let inspection = WorthQueryWorkflowReadmissionCleanupInspection {
            logical_run_identity: Arc::from(completed.affinity.logical_run_identity()),
            yielded_attempt_identity: Arc::from(completed.affinity.yielded_attempt_identity()),
            disposition: completed.disposition,
            checkpoint: WorthQueryReadmissionCleanupCheckpointInspection::capture(
                &completed.checkpoint_release,
            ),
            restored_execution: completed
                .restored_execution_release
                .as_ref()
                .map(WorthQueryReadmissionRestoredExecutionCleanupInspection::capture),
            bridge_reservation_released: completed.bridge.reservation_released(),
            bridge_signal_terminal: completed.bridge.signal_terminal(),
            bridge_signal_transition_performed: completed.bridge.signal_transition_performed(),
            relational_basis_released: completed.relational.released(),
            provider_session_identity: Arc::from(attempt.provider_session_identity()),
            resource_plan_identity: Arc::from(capacity.resource_plan_identity()),
            capacity_scope: capacity.scope(),
            released_reservation_count: capacity.released_reservation_count(),
            artifact_evidence: completed.artifact_evidence,
            generation_rollback: completed.generation_rollback,
            run_counters: completed.run_counters,
            provider_work: completed.affinity.provider_work().clone(),
            yield_counters: completed.yield_counters,
            readmission_evidence: completed.readmission_evidence,
        };
        Self { inspection }
    }

    pub const fn inspection(&self) -> &WorthQueryWorkflowReadmissionCleanupInspection {
        &self.inspection
    }
}

impl WorthQueryWorkflowReadmissionCleanupPendingInspection {
    pub(super) fn capture(
        partial: &WorthQueryWorkflowReadmissionPartialCleanupReceipt,
        artifact_cleanup_pending: bool,
        bridge_cleanup_pending: bool,
    ) -> Self {
        let attempt = partial.affinity.attempt();
        let capacity = attempt.capacity();
        Self {
            logical_run_identity: Arc::from(partial.affinity.logical_run_identity()),
            yielded_attempt_identity: Arc::from(partial.affinity.yielded_attempt_identity()),
            checkpoint: WorthQueryReadmissionCleanupCheckpointInspection::capture(
                &partial.checkpoint_release,
            ),
            restored_execution: partial
                .restored_execution_release
                .as_ref()
                .map(WorthQueryReadmissionRestoredExecutionCleanupInspection::capture),
            provider_session_identity: Arc::from(attempt.provider_session_identity()),
            resource_plan_identity: Arc::from(capacity.resource_plan_identity()),
            capacity_scope: capacity.scope(),
            released_reservation_count: capacity.released_reservation_count(),
            artifact_evidence: partial.artifact_evidence,
            generation_rollback: partial.generation_rollback.clone(),
            artifact_cleanup_pending,
            bridge_cleanup_pending,
            run_counters: partial.run_counters.clone(),
            provider_work: partial.affinity.provider_work().clone(),
            yield_counters: partial.yield_counters,
            readmission_evidence: partial.readmission_progress.evidence(),
        }
    }

    pub fn logical_run_identity(&self) -> &str {
        &self.logical_run_identity
    }
    pub fn yielded_attempt_identity(&self) -> &str {
        &self.yielded_attempt_identity
    }
    pub const fn disposition(&self) -> WorthQueryManagedRunCleanupDisposition {
        WorthQueryManagedRunCleanupDisposition::CleanupPending
    }
    pub const fn checkpoint(&self) -> &WorthQueryReadmissionCleanupCheckpointInspection {
        &self.checkpoint
    }
    pub const fn restored_execution(
        &self,
    ) -> Option<&WorthQueryReadmissionRestoredExecutionCleanupInspection> {
        self.restored_execution.as_ref()
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
    pub const fn artifact_evidence(&self) -> WorthQueryWorkflowArtifactRegistryEvidence {
        self.artifact_evidence
    }
    pub const fn generation_rollback(
        &self,
    ) -> Option<&WorthQueryArtifactGenerationRollbackEvidence> {
        self.generation_rollback.as_ref()
    }
    pub const fn artifact_cleanup_pending(&self) -> bool {
        self.artifact_cleanup_pending
    }
    pub const fn bridge_cleanup_pending(&self) -> bool {
        self.bridge_cleanup_pending
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
    pub const fn readmission_evidence(&self) -> WorthQueryReadmissionEvidence {
        self.readmission_evidence
    }
}

impl WorthQueryWorkflowReadmissionCleanupInspection {
    pub fn logical_run_identity(&self) -> &str {
        &self.logical_run_identity
    }
    pub fn yielded_attempt_identity(&self) -> &str {
        &self.yielded_attempt_identity
    }
    pub const fn disposition(&self) -> WorthQueryManagedRunCleanupDisposition {
        self.disposition
    }
    pub const fn checkpoint(&self) -> &WorthQueryReadmissionCleanupCheckpointInspection {
        &self.checkpoint
    }
    pub const fn restored_execution(
        &self,
    ) -> Option<&WorthQueryReadmissionRestoredExecutionCleanupInspection> {
        self.restored_execution.as_ref()
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
    pub const fn generation_rollback(
        &self,
    ) -> Option<&WorthQueryArtifactGenerationRollbackEvidence> {
        self.generation_rollback.as_ref()
    }
    pub fn resources_released(&self) -> bool {
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
    pub const fn readmission_evidence(&self) -> WorthQueryReadmissionEvidence {
        self.readmission_evidence
    }
}

impl WorthQueryArtifactGenerationRollbackEvidence {
    pub(super) fn capture(failure: WorthQueryArtifactProductionGenerationAbortFailure) -> Self {
        Self {
            prior_generation: failure.prior_generation().ordinal(),
            pending_generation: failure.pending_generation().ordinal(),
            denial_kind: failure.denial().kind(),
            detail: failure.detail(),
        }
    }

    pub const fn prior_generation(&self) -> u64 {
        self.prior_generation
    }
    pub const fn pending_generation(&self) -> u64 {
        self.pending_generation
    }
    pub const fn denial_kind(&self) -> WorthQueryArtifactDenialKind {
        self.denial_kind
    }
    pub const fn detail(&self) -> &'static str {
        self.detail
    }
}
