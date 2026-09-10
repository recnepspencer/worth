use crate::domain_computation::artifact_owner::WorthQueryArtifactOccurrenceSnapshot;
use crate::domain_computation::provider_session::graph_provider::bounded_step::WorthQueryGraphProviderMemorySnapshot;
use crate::domain_computation::{
    WorthQueryGraphProviderStepReport, WorthQueryProviderExecutionReleaseEvidence,
};

use super::evidence::WorthQueryManagedProviderWorkEvidenceParts;
use super::retention::WorthQueryManagedProviderRetentionLedger;
use super::WorthQueryManagedProviderCleanupAuthority;
use super::{
    WorthQueryManagedProviderExecutionReleaseSummary, WorthQueryManagedProviderWorkEvidence,
};

pub(crate) struct WorthQueryManagedProviderWorkLedger {
    provider_session_identity:
        crate::domain_computation::WorthQueryExecutionProviderSessionIdentity,
    issued_call_count: usize,
    abandoned_call_count: usize,
    interrupted_call_count: usize,
    admitted_receipt_count: usize,
    completed_work_units: u64,
    attempted_effect_count: u64,
    applied_effect_count: u64,
    produced_artifact_count: usize,
    retained_artifact_count: usize,
    disposed_artifact_count: usize,
    peak_scratch_bytes: usize,
    retention: WorthQueryManagedProviderRetentionLedger,
    checkpoint_available: bool,
    checkpoint_available_observation_count: usize,
    provider_step_attempt_count: usize,
    safe_point_request_lookup_count: usize,
    pressure_classification_count: usize,
    output_capacity_classification_count: usize,
    queue_request_lookup_count: usize,
    queue_state_mutation_count: usize,
    last_safe_point: Option<super::super::WorthQueryManagedSafePointObservation>,
    last_step_failure:
        Option<crate::domain_computation::WorthQueryGraphProviderStepFailureEvidence>,
    provider_execution_release: WorthQueryManagedProviderExecutionReleaseSummary,
    cleanup: WorthQueryManagedProviderCleanupAuthority,
}

impl WorthQueryManagedProviderWorkLedger {
    pub(crate) fn new(
        provider_session_identity: crate::domain_computation::WorthQueryExecutionProviderSessionIdentity,
    ) -> Self {
        Self {
            provider_session_identity,
            issued_call_count: 0,
            abandoned_call_count: 0,
            interrupted_call_count: 0,
            admitted_receipt_count: 0,
            completed_work_units: 0,
            attempted_effect_count: 0,
            applied_effect_count: 0,
            produced_artifact_count: 0,
            retained_artifact_count: 0,
            disposed_artifact_count: 0,
            peak_scratch_bytes: 0,
            retention: WorthQueryManagedProviderRetentionLedger::default(),
            checkpoint_available: false,
            checkpoint_available_observation_count: 0,
            provider_step_attempt_count: 0,
            safe_point_request_lookup_count: 0,
            pressure_classification_count: 0,
            output_capacity_classification_count: 0,
            queue_request_lookup_count: 0,
            queue_state_mutation_count: 0,
            last_safe_point: None,
            last_step_failure: None,
            provider_execution_release: WorthQueryManagedProviderExecutionReleaseSummary::default(),
            cleanup: WorthQueryManagedProviderCleanupAuthority::default(),
        }
    }

    pub(crate) fn begin_step_call(&mut self) {
        self.issued_call_count = self.issued_call_count.saturating_add(1);
    }

    pub(crate) fn admit_step(&mut self, report: &WorthQueryGraphProviderStepReport) {
        self.completed_work_units = self
            .completed_work_units
            .saturating_add(report.completed_work_units());
        self.attempted_effect_count = self
            .attempted_effect_count
            .saturating_add(report.attempted_effect_count());
        self.applied_effect_count = self
            .applied_effect_count
            .saturating_add(report.applied_effect_count());
        self.peak_scratch_bytes = self
            .peak_scratch_bytes
            .max(usize::try_from(report.peak_scratch_bytes()).unwrap_or(usize::MAX));
        let artifacts = report.artifact_evidence();
        self.retention.admit_step(
            report.retained_evidence(),
            self.cleanup.provider_retained_bytes(),
        );
        self.produced_artifact_count = self
            .produced_artifact_count
            .saturating_add(artifacts.produced_artifact_count());
        self.retained_artifact_count = artifacts.retained_artifact_count();
        self.disposed_artifact_count = self
            .disposed_artifact_count
            .saturating_add(artifacts.disposed_artifact_count());
        self.checkpoint_available = report.checkpoint_available();
        if report.checkpoint_available() {
            self.checkpoint_available_observation_count = self
                .checkpoint_available_observation_count
                .saturating_add(1);
        }
        self.last_step_failure = report.failure().cloned();
    }

    pub(crate) fn complete_step_call(&mut self) {
        self.admitted_receipt_count = self.admitted_receipt_count.saturating_add(1);
    }

    pub(crate) fn record_provider_step_attempt(&mut self) {
        self.provider_step_attempt_count = self.provider_step_attempt_count.saturating_add(1);
    }

    pub(crate) fn record_safe_point(
        &mut self,
        observation: &super::super::WorthQueryManagedSafePointObservation,
    ) {
        let counters = observation.counters();
        self.safe_point_request_lookup_count = self
            .safe_point_request_lookup_count
            .saturating_add(counters.exact_signal_request_lookup_count());
        self.pressure_classification_count = self
            .pressure_classification_count
            .saturating_add(counters.pressure_classification_count());
        self.last_safe_point = Some(observation.clone());
    }

    pub(crate) fn record_provider_step_admission(
        &mut self,
        counters: super::super::provider_step_admission::WorthQueryProviderStepAdmissionCounters,
    ) {
        self.output_capacity_classification_count = self
            .output_capacity_classification_count
            .saturating_add(counters.output_capacity_classification_count());
    }

    pub(crate) fn record_queue_mutation(
        &mut self,
        counters: worth_runtime_bridge::facade::BridgeManagedQueueMutationCounters,
    ) {
        self.queue_request_lookup_count = self
            .queue_request_lookup_count
            .saturating_add(counters.exact_signal_request_lookup_count());
        self.queue_state_mutation_count = self
            .queue_state_mutation_count
            .saturating_add(counters.queue_state_mutation_count());
    }

    pub(crate) fn release_or_retain_queue_occupancy(
        &mut self,
        basis: &mut worth_runtime_bridge::facade::BridgeBoundExecutionBasis,
        occupancy: worth_runtime_bridge::facade::BridgeManagedQueueOccupancy,
    ) -> bool {
        match basis.release_managed_queue_occupancy(occupancy) {
            Ok(mutation) => {
                self.record_queue_mutation(mutation.counters());
                true
            }
            Err(failure) => {
                self.cleanup
                    .retain_queue_occupancy(failure.into_occupancy());
                false
            }
        }
    }

    pub(crate) fn record_provider_execution_release(
        &mut self,
        evidence: &WorthQueryProviderExecutionReleaseEvidence,
    ) {
        self.provider_execution_release.record(evidence);
    }

    pub(crate) fn observe_active_provider_memory(
        &mut self,
        memory: WorthQueryGraphProviderMemorySnapshot,
    ) {
        self.retention
            .observe_active_provider(memory, self.cleanup.provider_retained_bytes());
    }

    pub(crate) fn retain_provider_memory(
        &mut self,
        memory: crate::domain_computation::provider_session::graph_provider::bounded_step::WorthQueryGraphProviderMemoryArena,
    ) {
        let retained_bytes = self.cleanup.retain_provider_memory(memory);
        self.retention
            .reconcile_provider_liabilities(retained_bytes);
    }

    pub(crate) fn release_projection_bytes(&mut self, retained_bytes: usize) -> bool {
        self.retention.release_projection(retained_bytes)
    }

    pub(crate) fn transfer_projection_to_output(
        &mut self,
        projection_bytes: usize,
        additional_bytes: usize,
    ) -> bool {
        self.retention
            .transfer_projection_to_output(projection_bytes, additional_bytes)
    }

    pub(crate) fn release_output_bytes(&mut self, retained_bytes: usize) -> bool {
        self.retention.release_output(retained_bytes)
    }

    pub(crate) fn transfer_output_to_receipt(
        &mut self,
        envelope_bytes: usize,
        total_output_bytes: usize,
    ) -> bool {
        self.retention.retain_output_envelope(envelope_bytes);
        self.retention.release_output(total_output_bytes)
    }

    pub(crate) fn settle_artifacts(&mut self, snapshot: WorthQueryArtifactOccurrenceSnapshot) {
        self.produced_artifact_count = snapshot.produced_artifact_count();
        self.retained_artifact_count = snapshot.retained_artifact_count();
        self.disposed_artifact_count = snapshot.disposed_artifact_count();
        self.retention.settle_artifacts(snapshot.retained_bytes());
    }

    pub(crate) fn interrupt_step_call(&mut self) {
        self.interrupted_call_count = self.interrupted_call_count.saturating_add(1);
    }

    pub(crate) fn abandon(&mut self) {
        self.abandoned_call_count = self.abandoned_call_count.saturating_add(1);
    }

    pub(crate) fn has_uncertainty(&self) -> bool {
        self.abandoned_call_count != 0
    }

    pub(crate) fn rebind_direct_provider_session(
        mut self,
        binding: crate::domain_computation::provider_session::WorthQueryDirectProviderWorkRebinding,
    ) -> Result<Self, Self> {
        if !binding.admits(&self.provider_session_identity) {
            return Err(self);
        }
        self.provider_session_identity = binding.into_fresh();
        Ok(self)
    }

    pub(crate) fn rebind_workflow_provider_session(
        mut self,
        binding: crate::domain_computation::provider_session::WorthQueryWorkflowProviderWorkRebinding,
    ) -> Result<Self, Self> {
        if !binding.admits(&self.provider_session_identity) {
            return Err(self);
        }
        self.provider_session_identity = binding.into_fresh();
        Ok(self)
    }

    pub(crate) fn snapshot(&self) -> WorthQueryManagedProviderWorkEvidence {
        WorthQueryManagedProviderWorkEvidence::from_parts(
            WorthQueryManagedProviderWorkEvidenceParts {
                provider_session_identity: self.provider_session_identity.description(),
                issued_call_count: self.issued_call_count,
                abandoned_call_count: self.abandoned_call_count,
                interrupted_call_count: self.interrupted_call_count,
                admitted_receipt_count: self.admitted_receipt_count,
                completed_work_units: self.completed_work_units,
                attempted_effect_count: self.attempted_effect_count,
                applied_effect_count: self.applied_effect_count,
                produced_artifact_count: self.produced_artifact_count,
                retained_artifact_count: self.retained_artifact_count,
                disposed_artifact_count: self.disposed_artifact_count,
                peak_scratch_bytes: self.peak_scratch_bytes,
                provider_retained_bytes: self.retention.provider_bytes(),
                output_retained_bytes: self.retention.output_bytes(),
                retained_bytes: self.retention.current_bytes(),
                peak_retained_bytes: self.retention.peak_bytes(),
                checkpoint_available: self.checkpoint_available,
                checkpoint_available_observation_count: self.checkpoint_available_observation_count,
                provider_step_attempt_count: self.provider_step_attempt_count,
                safe_point_request_lookup_count: self.safe_point_request_lookup_count,
                pressure_classification_count: self.pressure_classification_count,
                output_capacity_classification_count: self.output_capacity_classification_count,
                queue_request_lookup_count: self.queue_request_lookup_count,
                queue_state_mutation_count: self.queue_state_mutation_count,
                last_safe_point: self.last_safe_point.clone(),
                last_step_failure: self.last_step_failure.clone(),
                provider_execution_release: self.provider_execution_release.clone(),
            },
        )
    }

    pub(crate) fn into_evidence(self) -> WorthQueryManagedProviderWorkEvidence {
        self.snapshot()
    }

    pub(crate) fn into_terminal_parts(
        self,
    ) -> (
        WorthQueryManagedProviderWorkEvidence,
        WorthQueryManagedProviderCleanupAuthority,
    ) {
        let evidence = self.snapshot();
        (evidence, self.cleanup)
    }
}
