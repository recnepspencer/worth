use crate::intent_admission::dx::WorthQueryRuntimeIntentAdmissionReviewData;
use crate::intent_admission::{
    WorthQueryGenericInspectionIntentSeed, WorthQueryGenericInspectionIntentTarget,
    WorthQueryIntentAdmissionDecision, WorthQueryIntentAdmissionEligibility,
    WorthQueryIntentDecisionTraceEnvelope, WorthQueryUnifiedInspectionExecutionBinding,
    WorthQueryUnifiedInspectionExecutionHandoff,
};
use crate::runtime::{
    WorthQueryIntentConsumerInspection, WorthQueryRuntime, WorthQueryRuntimeError,
    WorthQueryUnifiedInspectionResult,
};

pub struct WorthQueryRuntimeInspectionIntentAuthoring<'a> {
    runtime: &'a WorthQueryRuntime,
    seed: WorthQueryGenericInspectionIntentSeed,
}

impl<'a> WorthQueryRuntimeInspectionIntentAuthoring<'a> {
    pub(crate) fn new(
        runtime: &'a WorthQueryRuntime,
        seed: WorthQueryGenericInspectionIntentSeed,
    ) -> Self {
        Self { runtime, seed }
    }

    pub fn review(
        self,
    ) -> Result<WorthQueryRuntimeInspectionIntentAdmissionReview<'a>, WorthQueryRuntimeError> {
        let review = self.runtime.review_unified_inspection(self.seed)?;
        Ok(WorthQueryRuntimeInspectionIntentAdmissionReview {
            runtime: self.runtime,
            review,
        })
    }

    pub fn admit(
        self,
    ) -> Result<WorthQueryAdmittedRuntimeInspectionIntent<'a>, WorthQueryRuntimeError> {
        self.review()?.admit()
    }

    pub fn execute(
        self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<WorthQueryUnifiedInspectionResult, WorthQueryRuntimeError> {
        self.admit()?.execute(execution)
    }
}

pub struct WorthQueryRuntimeInspectionIntentAdmissionReview<'a> {
    runtime: &'a WorthQueryRuntime,
    review: WorthQueryRuntimeIntentAdmissionReviewData,
}

impl<'a> WorthQueryRuntimeInspectionIntentAdmissionReview<'a> {
    pub fn request(&self) -> &crate::intent_admission::WorthQueryRawIntentAdmissionRequest {
        self.review.request()
    }

    pub fn eligibility(&self) -> &WorthQueryIntentAdmissionEligibility {
        self.review.eligibility()
    }

    pub fn decision(&self) -> &WorthQueryIntentAdmissionDecision {
        self.review.decision()
    }

    pub fn admitted_handoff(&self) -> Option<WorthQueryUnifiedInspectionExecutionHandoff> {
        self.runtime
            .resolve_reviewed_admitted_unified_inspection_handoff(self.review.clone())
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
    ) -> Result<WorthQueryAdmittedRuntimeInspectionIntent<'a>, WorthQueryRuntimeError> {
        let handoff = self
            .runtime
            .resolve_reviewed_admitted_unified_inspection_handoff(self.review.clone())
            .map_err(|_| {
                self.runtime
                    .unified_inspection_non_admitted_error(&self.review)
            })?;
        let execution_binding = self
            .runtime
            .prepare_unified_inspection_execution_binding(handoff.clone());
        Ok(WorthQueryAdmittedRuntimeInspectionIntent {
            runtime: self.runtime,
            review: self.review,
            handoff,
            execution_binding,
        })
    }

    pub fn execute(
        self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<WorthQueryUnifiedInspectionResult, WorthQueryRuntimeError> {
        self.admit()?.execute(execution)
    }
}

pub struct WorthQueryAdmittedRuntimeInspectionIntent<'a> {
    runtime: &'a WorthQueryRuntime,
    review: WorthQueryRuntimeIntentAdmissionReviewData,
    handoff: WorthQueryUnifiedInspectionExecutionHandoff,
    execution_binding: WorthQueryUnifiedInspectionExecutionBinding,
}

impl<'a> WorthQueryAdmittedRuntimeInspectionIntent<'a> {
    pub fn request(&self) -> &crate::intent_admission::WorthQueryRawIntentAdmissionRequest {
        self.review.request()
    }

    pub fn eligibility(&self) -> &WorthQueryIntentAdmissionEligibility {
        self.review.eligibility()
    }

    pub fn decision(&self) -> &WorthQueryIntentAdmissionDecision {
        self.review.decision()
    }

    pub fn handoff(&self) -> &WorthQueryUnifiedInspectionExecutionHandoff {
        &self.handoff
    }

    pub fn execution_binding(&self) -> &WorthQueryUnifiedInspectionExecutionBinding {
        &self.execution_binding
    }

    pub fn execute(
        self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<WorthQueryUnifiedInspectionResult, WorthQueryRuntimeError> {
        self.runtime
            .execute_unified_inspection_execution_binding(execution, self.execution_binding)
    }
}

impl WorthQueryRuntime {
    pub fn inspect_intent<'a, T>(
        &'a self,
        target: T,
    ) -> WorthQueryRuntimeInspectionIntentAuthoring<'a>
    where
        T: WorthQueryGenericInspectionIntentTarget<'a>,
    {
        WorthQueryRuntimeInspectionIntentAuthoring::new(
            self,
            target.into_generic_inspection_intent_seed(),
        )
    }
}
