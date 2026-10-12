use crate::intent_admission::{
    WorthQueryAuthoritativeIntentExecutionBinding, WorthQueryAuthoritativeIntentExecutionHandoff,
};
use crate::runtime::{
    WorthQueryIntentConsumerInspection, WorthQueryIntentDeclaration, WorthQueryIntentReceipt,
    WorthQueryRuntime, WorthQueryRuntimeError,
};

use super::{
    non_admitted_runtime_violation, WorthQueryIntentAdmissionDecision,
    WorthQueryIntentAdmissionEligibility, WorthQueryIntentDecisionTraceEnvelope,
    WorthQueryRawIntentAdmissionRequest, WorthQueryRuntimeIntentAdmissionReviewData,
};

pub struct WorthQueryRuntimeIntentAuthoring<'a> {
    runtime: &'a mut WorthQueryRuntime,
    declaration: WorthQueryIntentDeclaration,
}

impl<'a> WorthQueryRuntimeIntentAuthoring<'a> {
    pub(crate) fn new(
        runtime: &'a mut WorthQueryRuntime,
        declaration: WorthQueryIntentDeclaration,
    ) -> Self {
        Self {
            runtime,
            declaration,
        }
    }

    pub fn review(
        self,
    ) -> Result<WorthQueryRuntimeIntentAdmissionReview<'a>, WorthQueryRuntimeError> {
        let review = self
            .runtime
            .review_authoritative_runtime_intent(self.declaration)?;
        Ok(WorthQueryRuntimeIntentAdmissionReview {
            runtime: self.runtime,
            review,
        })
    }

    pub fn admit(self) -> Result<WorthQueryAdmittedRuntimeIntent<'a>, WorthQueryRuntimeError> {
        self.review()?.admit()
    }

    pub fn execute(
        self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<WorthQueryIntentReceipt, WorthQueryRuntimeError> {
        self.admit()?.execute(execution)
    }
}

pub struct WorthQueryRuntimeIntentAdmissionReview<'a> {
    runtime: &'a mut WorthQueryRuntime,
    review: WorthQueryRuntimeIntentAdmissionReviewData,
}

impl<'a> WorthQueryRuntimeIntentAdmissionReview<'a> {
    pub fn request(&self) -> &WorthQueryRawIntentAdmissionRequest {
        self.review.request()
    }

    pub fn eligibility(&self) -> &WorthQueryIntentAdmissionEligibility {
        self.review.eligibility()
    }

    pub fn decision(&self) -> &WorthQueryIntentAdmissionDecision {
        self.review.decision()
    }

    pub fn admitted_plan(&self) -> Option<&super::WorthQueryAdmittedIntentPlan> {
        self.review.admitted_plan()
    }

    pub fn admitted_handoff(&self) -> Option<WorthQueryAuthoritativeIntentExecutionHandoff> {
        self.runtime
            .resolve_reviewed_admitted_authoritative_intent_handoff(self.review.clone())
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

    pub fn admit(self) -> Result<WorthQueryAdmittedRuntimeIntent<'a>, WorthQueryRuntimeError> {
        let handoff = self
            .runtime
            .resolve_reviewed_admitted_authoritative_intent_handoff(self.review.clone())
            .map_err(|_| {
                let violation = non_admitted_runtime_violation(&self.review);
                self.runtime.intent_violation_error(
                    self.review
                        .request()
                        .runtime_declaration()
                        .expect("authoritative runtime review must preserve declaration"),
                    violation,
                    None,
                    self.review.decision_trace_envelope().cloned(),
                    None,
                )
            })?;
        let execution_binding = self
            .runtime
            .prepare_authoritative_intent_execution_binding(handoff.clone());
        Ok(WorthQueryAdmittedRuntimeIntent {
            runtime: self.runtime,
            review: self.review,
            handoff,
            execution_binding,
        })
    }

    pub fn execute(
        self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<WorthQueryIntentReceipt, WorthQueryRuntimeError> {
        self.admit()?.execute(execution)
    }
}

pub struct WorthQueryAdmittedRuntimeIntent<'a> {
    runtime: &'a mut WorthQueryRuntime,
    review: WorthQueryRuntimeIntentAdmissionReviewData,
    handoff: WorthQueryAuthoritativeIntentExecutionHandoff,
    execution_binding: WorthQueryAuthoritativeIntentExecutionBinding,
}

impl<'a> WorthQueryAdmittedRuntimeIntent<'a> {
    pub fn request(&self) -> &WorthQueryRawIntentAdmissionRequest {
        self.review.request()
    }

    pub fn eligibility(&self) -> &WorthQueryIntentAdmissionEligibility {
        self.review.eligibility()
    }

    pub fn decision(&self) -> &WorthQueryIntentAdmissionDecision {
        self.review.decision()
    }

    pub fn handoff(&self) -> &WorthQueryAuthoritativeIntentExecutionHandoff {
        &self.handoff
    }

    pub fn execution_binding(&self) -> &WorthQueryAuthoritativeIntentExecutionBinding {
        &self.execution_binding
    }

    pub fn execute(
        self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<WorthQueryIntentReceipt, WorthQueryRuntimeError> {
        self.runtime
            .execute_authoritative_intent_execution_binding(execution, self.execution_binding)
    }
}
