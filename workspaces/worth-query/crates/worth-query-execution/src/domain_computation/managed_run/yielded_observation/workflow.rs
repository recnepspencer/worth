use std::sync::Arc;

use super::WorthQueryYieldedCheckpointInspection;
use {
    crate::domain_computation::vocabulary::managed_run::counters::WorthQueryManagedRunCounters,
    crate::domain_computation::vocabulary::managed_run::evidence::WorthQueryManagedProviderWorkEvidence,
    crate::domain_computation::WorthQueryWorkflowArtifactRegistryEvidence,
    crate::domain_computation::WorthQueryYieldTransitionCounters,
};

#[derive(Clone, Debug, PartialEq)]
pub struct WorthQueryYieldedWorkflowRunInspection {
    logical_run_identity: Arc<str>,
    yielded_attempt_identity: Arc<str>,
    operation_binding_identity: Arc<str>,
    installed_operation_identity: Arc<str>,
    semantic_basis_identity: Arc<str>,
    installation_generation: worth_query_installation::facade::WorthQueryInstallationGeneration,
    provider_session_identity: Arc<str>,
    checkpoint: WorthQueryYieldedCheckpointInspection,
    provider_work: WorthQueryManagedProviderWorkEvidence,
    run_counters: WorthQueryManagedRunCounters,
    yield_counters: WorthQueryYieldTransitionCounters,
    retained_capacity_reservation_count: usize,
    artifact_evidence: WorthQueryWorkflowArtifactRegistryEvidence,
}

impl WorthQueryYieldedWorkflowRunInspection {
    pub(in crate::domain_computation::managed_run) fn capture(
        affinity: &super::super::workflow::WorthQueryWorkflowRunAffinity,
        execution: &super::super::retained_graph_execution::WorthQueryRetainedManagedGraphExecution,
        run_counters: &WorthQueryManagedRunCounters,
        yield_counters: WorthQueryYieldTransitionCounters,
        artifact_evidence: WorthQueryWorkflowArtifactRegistryEvidence,
    ) -> Self {
        Self {
            logical_run_identity: Arc::from(affinity.logical_identity()),
            yielded_attempt_identity: Arc::from(affinity.attempt_identity()),
            operation_binding_identity: Arc::from(affinity.binding_identity_projection()),
            installed_operation_identity: Arc::from(affinity.operation_identity_projection()),
            semantic_basis_identity: Arc::from(affinity.basis_identity_projection()),
            installation_generation: affinity.installation_generation_projection(),
            provider_session_identity: Arc::from(affinity.evidence().provider_session_identity()),
            checkpoint: WorthQueryYieldedCheckpointInspection::capture(
                execution.checkpoint_evidence(),
            ),
            provider_work: affinity.provider_work_snapshot(),
            run_counters: run_counters.clone(),
            yield_counters,
            retained_capacity_reservation_count: affinity.retained_capacity_reservation_count(),
            artifact_evidence,
        }
    }

    pub fn logical_run_identity(&self) -> &str {
        &self.logical_run_identity
    }

    pub fn yielded_attempt_identity(&self) -> &str {
        &self.yielded_attempt_identity
    }

    pub fn operation_binding_identity(&self) -> &str {
        &self.operation_binding_identity
    }

    pub fn installed_operation_identity(&self) -> &str {
        &self.installed_operation_identity
    }

    pub fn semantic_basis_identity(&self) -> &str {
        &self.semantic_basis_identity
    }

    pub const fn installation_generation(
        &self,
    ) -> worth_query_installation::facade::WorthQueryInstallationGeneration {
        self.installation_generation
    }

    pub fn provider_session_identity(&self) -> &str {
        &self.provider_session_identity
    }

    pub const fn checkpoint(&self) -> &WorthQueryYieldedCheckpointInspection {
        &self.checkpoint
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

    pub const fn retained_capacity_reservation_count(&self) -> usize {
        self.retained_capacity_reservation_count
    }

    pub const fn artifact_evidence(&self) -> WorthQueryWorkflowArtifactRegistryEvidence {
        self.artifact_evidence
    }
}
