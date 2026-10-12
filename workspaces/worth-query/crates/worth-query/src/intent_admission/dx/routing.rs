use crate::intent_admission::dx::WorthQueryRuntimeIntentAdmissionReviewData;
use crate::intent_admission::{
    WorthQueryExistingTruthProbeExecutionBinding, WorthQueryExistingTruthProbeExecutionHandoff,
    WorthQueryIntentAdmissionDecision, WorthQueryIntentAdmissionEligibility,
    WorthQueryIntentDecisionTraceEnvelope,
};
use crate::runtime::{
    WorthQueryExistingTruthProbeRequest, WorthQueryExistingTruthProbeResult,
    WorthQueryIntentConsumerInspection, WorthQueryRuntime, WorthQueryRuntimeError,
};

pub struct WorthQueryRuntimeExistingTruthProbeIntentAuthoring<'a> {
    runtime: &'a WorthQueryRuntime,
    request: WorthQueryExistingTruthProbeRequest,
}

impl<'a> WorthQueryRuntimeExistingTruthProbeIntentAuthoring<'a> {
    pub(crate) fn new(
        runtime: &'a WorthQueryRuntime,
        request: WorthQueryExistingTruthProbeRequest,
    ) -> Self {
        Self { runtime, request }
    }

    pub fn review(
        self,
    ) -> Result<WorthQueryRuntimeExistingTruthProbeIntentAdmissionReview<'a>, WorthQueryRuntimeError>
    {
        let review = self
            .runtime
            .review_existing_truth_probe_routing(self.request)?;
        Ok(WorthQueryRuntimeExistingTruthProbeIntentAdmissionReview {
            runtime: self.runtime,
            review,
        })
    }

    pub fn admit(
        self,
    ) -> Result<WorthQueryAdmittedRuntimeExistingTruthProbeIntent<'a>, WorthQueryRuntimeError> {
        self.review()?.admit()
    }

    pub fn execute(
        self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<WorthQueryExistingTruthProbeResult, WorthQueryRuntimeError> {
        self.admit()?.execute(execution)
    }
}

pub struct WorthQueryRuntimeExistingTruthProbeIntentAdmissionReview<'a> {
    runtime: &'a WorthQueryRuntime,
    review: WorthQueryRuntimeIntentAdmissionReviewData,
}

impl<'a> WorthQueryRuntimeExistingTruthProbeIntentAdmissionReview<'a> {
    pub fn request(&self) -> &crate::intent_admission::WorthQueryRawIntentAdmissionRequest {
        self.review.request()
    }

    pub fn eligibility(&self) -> &WorthQueryIntentAdmissionEligibility {
        self.review.eligibility()
    }

    pub fn decision(&self) -> &WorthQueryIntentAdmissionDecision {
        self.review.decision()
    }

    pub fn admitted_handoff(&self) -> Option<WorthQueryExistingTruthProbeExecutionHandoff> {
        self.runtime
            .resolve_reviewed_admitted_existing_truth_probe_handoff(self.review.clone())
            .ok()
    }

    pub fn decision_trace_envelope(&self) -> Option<&WorthQueryIntentDecisionTraceEnvelope> {
        self.review.decision_trace_envelope()
    }

    pub fn consumer_inspection(&self) -> WorthQueryIntentConsumerInspection<'_> {
        WorthQueryIntentConsumerInspection::from_review(
            self.review.request().intent_name(),
            self.review.decision(),
            self.review.request().family(),
            self.review.request().entrypoint(),
            self.review.decision_trace_envelope(),
        )
    }

    pub fn admit(
        self,
    ) -> Result<WorthQueryAdmittedRuntimeExistingTruthProbeIntent<'a>, WorthQueryRuntimeError> {
        let handoff = self
            .runtime
            .resolve_reviewed_admitted_existing_truth_probe_handoff(self.review.clone())
            .map_err(|_| {
                self.runtime
                    .existing_truth_probe_non_admitted_error(&self.review)
            })?;
        let execution_binding = self
            .runtime
            .prepare_existing_truth_probe_execution_binding(handoff.clone());
        Ok(WorthQueryAdmittedRuntimeExistingTruthProbeIntent {
            runtime: self.runtime,
            review: self.review,
            handoff,
            execution_binding,
        })
    }

    pub fn execute(
        self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<WorthQueryExistingTruthProbeResult, WorthQueryRuntimeError> {
        self.admit()?.execute(execution)
    }
}

pub struct WorthQueryAdmittedRuntimeExistingTruthProbeIntent<'a> {
    runtime: &'a WorthQueryRuntime,
    review: WorthQueryRuntimeIntentAdmissionReviewData,
    handoff: WorthQueryExistingTruthProbeExecutionHandoff,
    execution_binding: WorthQueryExistingTruthProbeExecutionBinding,
}

impl<'a> WorthQueryAdmittedRuntimeExistingTruthProbeIntent<'a> {
    pub fn request(&self) -> &crate::intent_admission::WorthQueryRawIntentAdmissionRequest {
        self.review.request()
    }

    pub fn eligibility(&self) -> &WorthQueryIntentAdmissionEligibility {
        self.review.eligibility()
    }

    pub fn decision(&self) -> &WorthQueryIntentAdmissionDecision {
        self.review.decision()
    }

    pub fn handoff(&self) -> &WorthQueryExistingTruthProbeExecutionHandoff {
        &self.handoff
    }

    pub fn execution_binding(&self) -> &WorthQueryExistingTruthProbeExecutionBinding {
        &self.execution_binding
    }

    pub fn execute(
        self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<WorthQueryExistingTruthProbeResult, WorthQueryRuntimeError> {
        self.runtime
            .execute_existing_truth_probe_execution_binding(execution, self.execution_binding)
    }
}
