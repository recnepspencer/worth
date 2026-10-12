use crate::intent_admission::{
    WorthQueryAuthoritativeMutationExecutionBinding,
    WorthQueryAuthoritativeMutationExecutionHandoff,
};
use crate::runtime::{
    WorthQueryIntentConsumerInspection, WorthQueryRuntime, WorthQueryRuntimeError,
    WorthQueryWriteCommand, WorthQueryWriteReceipt,
};

use super::{
    WorthQueryIntentAdmissionDecision, WorthQueryIntentAdmissionEligibility,
    WorthQueryIntentDecisionTraceEnvelope, WorthQueryRawIntentAdmissionRequest,
    WorthQueryRuntimeIntentAdmissionReviewData,
};

pub struct WorthQueryRuntimeWriteIntentAuthoring<'a> {
    runtime: &'a mut WorthQueryRuntime,
    command: WorthQueryWriteCommand,
}

impl<'a> WorthQueryRuntimeWriteIntentAuthoring<'a> {
    pub(crate) fn new(runtime: &'a mut WorthQueryRuntime, command: WorthQueryWriteCommand) -> Self {
        Self { runtime, command }
    }

    pub fn review(
        self,
        request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<WorthQueryRuntimeWriteIntentAdmissionReview<'a>, WorthQueryRuntimeError> {
        let review = self
            .runtime
            .review_authoritative_runtime_write(request_execution, self.command)?;
        Ok(WorthQueryRuntimeWriteIntentAdmissionReview {
            runtime: self.runtime,
            review,
        })
    }

    pub fn admit(
        self,
        request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<WorthQueryAdmittedRuntimeWriteIntent<'a>, WorthQueryRuntimeError> {
        self.review(request_execution)?.admit()
    }

    pub fn execute(
        self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<WorthQueryWriteReceipt, WorthQueryRuntimeError> {
        self.admit(execution)?.execute(execution)
    }
}

pub struct WorthQueryRuntimeWriteIntentAdmissionReview<'a> {
    runtime: &'a mut WorthQueryRuntime,
    review: WorthQueryRuntimeIntentAdmissionReviewData,
}

impl<'a> WorthQueryRuntimeWriteIntentAdmissionReview<'a> {
    pub fn request(&self) -> &WorthQueryRawIntentAdmissionRequest {
        self.review.request()
    }

    pub fn eligibility(&self) -> &WorthQueryIntentAdmissionEligibility {
        self.review.eligibility()
    }

    pub fn decision(&self) -> &WorthQueryIntentAdmissionDecision {
        self.review.decision()
    }

    pub fn admitted_handoff(&self) -> Option<WorthQueryAuthoritativeMutationExecutionHandoff> {
        self.runtime
            .resolve_reviewed_admitted_authoritative_write_handoff(self.review.clone())
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

    pub fn admit(self) -> Result<WorthQueryAdmittedRuntimeWriteIntent<'a>, WorthQueryRuntimeError> {
        let handoff = self
            .runtime
            .resolve_reviewed_admitted_authoritative_write_handoff(self.review.clone())?;
        let execution_binding = self
            .runtime
            .prepare_authoritative_mutation_execution_binding(handoff.clone());
        Ok(WorthQueryAdmittedRuntimeWriteIntent {
            runtime: self.runtime,
            review: self.review,
            handoff,
            execution_binding,
        })
    }

    pub fn execute(
        self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<WorthQueryWriteReceipt, WorthQueryRuntimeError> {
        self.admit()?.execute(execution)
    }
}

pub struct WorthQueryAdmittedRuntimeWriteIntent<'a> {
    runtime: &'a mut WorthQueryRuntime,
    review: WorthQueryRuntimeIntentAdmissionReviewData,
    handoff: WorthQueryAuthoritativeMutationExecutionHandoff,
    execution_binding: WorthQueryAuthoritativeMutationExecutionBinding,
}

impl<'a> WorthQueryAdmittedRuntimeWriteIntent<'a> {
    pub fn request(&self) -> &WorthQueryRawIntentAdmissionRequest {
        self.review.request()
    }

    pub fn eligibility(&self) -> &WorthQueryIntentAdmissionEligibility {
        self.review.eligibility()
    }

    pub fn decision(&self) -> &WorthQueryIntentAdmissionDecision {
        self.review.decision()
    }

    pub fn handoff(&self) -> &WorthQueryAuthoritativeMutationExecutionHandoff {
        &self.handoff
    }

    pub fn execution_binding(&self) -> &WorthQueryAuthoritativeMutationExecutionBinding {
        &self.execution_binding
    }

    pub fn execute(
        self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<WorthQueryWriteReceipt, WorthQueryRuntimeError> {
        self.runtime
            .execute_authoritative_mutation_execution_binding(execution, self.execution_binding)
    }
}
