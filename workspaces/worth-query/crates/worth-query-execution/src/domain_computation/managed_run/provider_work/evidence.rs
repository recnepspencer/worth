use std::sync::Arc;

use super::WorthQueryManagedProviderExecutionReleaseSummary;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryManagedProviderSessionDisposition {
    Unused,
    ReceiptsAdmitted,
    Interrupted,
    Uncertain,
}

#[derive(Clone, Debug, PartialEq)]
pub struct WorthQueryManagedProviderWorkEvidence {
    provider_session_identity: Arc<str>,
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
    provider_retained_bytes: usize,
    output_retained_bytes: usize,
    retained_bytes: usize,
    peak_retained_bytes: usize,
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
}

pub(super) struct WorthQueryManagedProviderWorkEvidenceParts {
    pub(super) provider_session_identity: Arc<str>,
    pub(super) issued_call_count: usize,
    pub(super) abandoned_call_count: usize,
    pub(super) interrupted_call_count: usize,
    pub(super) admitted_receipt_count: usize,
    pub(super) completed_work_units: u64,
    pub(super) attempted_effect_count: u64,
    pub(super) applied_effect_count: u64,
    pub(super) produced_artifact_count: usize,
    pub(super) retained_artifact_count: usize,
    pub(super) disposed_artifact_count: usize,
    pub(super) peak_scratch_bytes: usize,
    pub(super) provider_retained_bytes: usize,
    pub(super) output_retained_bytes: usize,
    pub(super) retained_bytes: usize,
    pub(super) peak_retained_bytes: usize,
    pub(super) checkpoint_available: bool,
    pub(super) checkpoint_available_observation_count: usize,
    pub(super) provider_step_attempt_count: usize,
    pub(super) safe_point_request_lookup_count: usize,
    pub(super) pressure_classification_count: usize,
    pub(super) output_capacity_classification_count: usize,
    pub(super) queue_request_lookup_count: usize,
    pub(super) queue_state_mutation_count: usize,
    pub(super) last_safe_point: Option<super::super::WorthQueryManagedSafePointObservation>,
    pub(super) last_step_failure:
        Option<crate::domain_computation::WorthQueryGraphProviderStepFailureEvidence>,
    pub(super) provider_execution_release: WorthQueryManagedProviderExecutionReleaseSummary,
}

impl WorthQueryManagedProviderWorkEvidence {
    pub(super) fn from_parts(parts: WorthQueryManagedProviderWorkEvidenceParts) -> Self {
        Self {
            provider_session_identity: parts.provider_session_identity,
            issued_call_count: parts.issued_call_count,
            abandoned_call_count: parts.abandoned_call_count,
            interrupted_call_count: parts.interrupted_call_count,
            admitted_receipt_count: parts.admitted_receipt_count,
            completed_work_units: parts.completed_work_units,
            attempted_effect_count: parts.attempted_effect_count,
            applied_effect_count: parts.applied_effect_count,
            produced_artifact_count: parts.produced_artifact_count,
            retained_artifact_count: parts.retained_artifact_count,
            disposed_artifact_count: parts.disposed_artifact_count,
            peak_scratch_bytes: parts.peak_scratch_bytes,
            provider_retained_bytes: parts.provider_retained_bytes,
            output_retained_bytes: parts.output_retained_bytes,
            retained_bytes: parts.retained_bytes,
            peak_retained_bytes: parts.peak_retained_bytes,
            checkpoint_available: parts.checkpoint_available,
            checkpoint_available_observation_count: parts.checkpoint_available_observation_count,
            provider_step_attempt_count: parts.provider_step_attempt_count,
            safe_point_request_lookup_count: parts.safe_point_request_lookup_count,
            pressure_classification_count: parts.pressure_classification_count,
            output_capacity_classification_count: parts.output_capacity_classification_count,
            queue_request_lookup_count: parts.queue_request_lookup_count,
            queue_state_mutation_count: parts.queue_state_mutation_count,
            last_safe_point: parts.last_safe_point,
            last_step_failure: parts.last_step_failure,
            provider_execution_release: parts.provider_execution_release,
        }
    }

    pub fn provider_session_identity(&self) -> &str {
        &self.provider_session_identity
    }

    pub fn session_disposition(&self) -> WorthQueryManagedProviderSessionDisposition {
        if self.abandoned_call_count != 0 {
            WorthQueryManagedProviderSessionDisposition::Uncertain
        } else if self.interrupted_call_count != 0 {
            WorthQueryManagedProviderSessionDisposition::Interrupted
        } else if self.admitted_receipt_count == 0 {
            WorthQueryManagedProviderSessionDisposition::Unused
        } else {
            WorthQueryManagedProviderSessionDisposition::ReceiptsAdmitted
        }
    }

    pub fn issued_call_count(&self) -> usize {
        self.issued_call_count
    }

    pub fn admitted_receipt_count(&self) -> usize {
        self.admitted_receipt_count
    }

    pub fn abandoned_call_count(&self) -> usize {
        self.abandoned_call_count
    }

    pub fn interrupted_call_count(&self) -> usize {
        self.interrupted_call_count
    }

    pub fn completed_work_units(&self) -> u64 {
        self.completed_work_units
    }

    pub fn attempted_effect_count(&self) -> u64 {
        self.attempted_effect_count
    }

    pub fn applied_effect_count(&self) -> u64 {
        self.applied_effect_count
    }

    pub fn produced_artifact_count(&self) -> usize {
        self.produced_artifact_count
    }

    pub fn retained_artifact_count(&self) -> usize {
        self.retained_artifact_count
    }

    pub fn disposed_artifact_count(&self) -> usize {
        self.disposed_artifact_count
    }

    pub fn peak_scratch_bytes(&self) -> usize {
        self.peak_scratch_bytes
    }

    pub fn retained_bytes(&self) -> usize {
        self.retained_bytes
    }

    pub const fn provider_retained_bytes(&self) -> usize {
        self.provider_retained_bytes
    }

    pub const fn output_retained_bytes(&self) -> usize {
        self.output_retained_bytes
    }

    pub const fn peak_retained_bytes(&self) -> usize {
        self.peak_retained_bytes
    }

    pub const fn checkpoint_available(&self) -> bool {
        self.checkpoint_available
    }

    pub const fn checkpoint_available_observation_count(&self) -> usize {
        self.checkpoint_available_observation_count
    }

    pub const fn provider_step_attempt_count(&self) -> usize {
        self.provider_step_attempt_count
    }

    pub const fn safe_point_request_lookup_count(&self) -> usize {
        self.safe_point_request_lookup_count
    }

    pub const fn pressure_classification_count(&self) -> usize {
        self.pressure_classification_count
    }

    pub const fn output_capacity_classification_count(&self) -> usize {
        self.output_capacity_classification_count
    }

    pub const fn queue_request_lookup_count(&self) -> usize {
        self.queue_request_lookup_count
    }

    pub const fn queue_state_mutation_count(&self) -> usize {
        self.queue_state_mutation_count
    }

    pub const fn last_safe_point(
        &self,
    ) -> Option<&super::super::WorthQueryManagedSafePointObservation> {
        self.last_safe_point.as_ref()
    }

    pub const fn last_step_failure(
        &self,
    ) -> Option<&crate::domain_computation::WorthQueryGraphProviderStepFailureEvidence> {
        self.last_step_failure.as_ref()
    }

    pub const fn provider_execution_release(
        &self,
    ) -> &WorthQueryManagedProviderExecutionReleaseSummary {
        &self.provider_execution_release
    }

    pub(crate) fn has_uncertainty(&self) -> bool {
        self.abandoned_call_count != 0
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

    pub(crate) fn reconcile_provider_retained_bytes(&mut self, provider_retained_bytes: usize) {
        self.retained_bytes = self
            .retained_bytes
            .saturating_sub(self.provider_retained_bytes)
            .saturating_add(provider_retained_bytes);
        self.provider_retained_bytes = provider_retained_bytes;
    }

    pub(crate) fn requires_cleanup_recovery(&self) -> bool {
        self.has_uncertainty() || self.provider_execution_release.recovery_required()
    }
}
