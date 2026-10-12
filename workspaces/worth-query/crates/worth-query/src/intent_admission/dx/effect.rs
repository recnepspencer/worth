use crate::intent_admission::{
    WorthQueryEffectTriggeredIntentExecutionBinding,
    WorthQueryEffectTriggeredIntentExecutionHandoff,
};
use crate::runtime::{
    WorthQueryEffectDelivery, WorthQueryEffectHandle, WorthQueryEffectIntentReceipt,
    WorthQueryIntentConsumerInspection, WorthQueryRuntime, WorthQueryRuntimeError,
};

use super::{
    non_admitted_runtime_violation, WorthQueryIntentAdmissionDecision,
    WorthQueryIntentAdmissionEligibility, WorthQueryIntentDecisionTraceEnvelope,
    WorthQueryRawIntentAdmissionRequest, WorthQueryRuntimeIntentAdmissionReviewData,
};

pub struct WorthQueryRuntimeEffectWriteIntentAuthoring<'a> {
    runtime: &'a mut WorthQueryRuntime,
    effect_name: String,
    strategy_version: String,
    input_contract: String,
}

impl<'a> WorthQueryRuntimeEffectWriteIntentAuthoring<'a> {
    pub(crate) fn new<T>(
        runtime: &'a mut WorthQueryRuntime,
        effect: &WorthQueryEffectHandle<T>,
        strategy_version: String,
        input_contract: String,
    ) -> Self {
        Self {
            runtime,
            effect_name: effect.name().to_string(),
            strategy_version,
            input_contract,
        }
    }

    pub fn review(
        self,
    ) -> Result<WorthQueryRuntimeEffectWriteIntentAdmissionReview<'a>, WorthQueryRuntimeError> {
        let (pending_delivery, review) = self.runtime.review_next_effect_write_runtime_intent(
            &self.effect_name,
            self.strategy_version,
            self.input_contract,
        )?;
        Ok(WorthQueryRuntimeEffectWriteIntentAdmissionReview {
            runtime: self.runtime,
            review,
            pending_delivery,
        })
    }

    pub fn admit(
        self,
    ) -> Result<WorthQueryAdmittedRuntimeEffectWriteIntent<'a>, WorthQueryRuntimeError> {
        self.review()?.admit()
    }

    pub fn execute(
        self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<WorthQueryEffectIntentReceipt, WorthQueryRuntimeError> {
        self.admit()?.execute(execution)
    }
}

pub struct WorthQueryRuntimeEffectWriteIntentAdmissionReview<'a> {
    runtime: &'a mut WorthQueryRuntime,
    review: WorthQueryRuntimeIntentAdmissionReviewData,
    pending_delivery: WorthQueryEffectDelivery,
}

impl<'a> WorthQueryRuntimeEffectWriteIntentAdmissionReview<'a> {
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

    pub fn admitted_handoff(&self) -> Option<WorthQueryEffectTriggeredIntentExecutionHandoff> {
        self.runtime
            .resolve_reviewed_admitted_effect_intent_handoff(self.review.clone())
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

    pub fn pending_delivery(&self) -> &WorthQueryEffectDelivery {
        &self.pending_delivery
    }

    pub fn admit(
        self,
    ) -> Result<WorthQueryAdmittedRuntimeEffectWriteIntent<'a>, WorthQueryRuntimeError> {
        let handoff = self
            .runtime
            .resolve_reviewed_admitted_effect_intent_handoff(self.review.clone())
            .map_err(|_| {
                let violation = non_admitted_runtime_violation(&self.review);
                self.runtime.intent_violation_error(
                    self.review
                        .request()
                        .runtime_declaration()
                        .expect("effect runtime review must preserve declaration"),
                    violation,
                    None,
                    self.review.decision_trace_envelope().cloned(),
                    None,
                )
            })?;
        let execution_binding = self
            .runtime
            .prepare_effect_intent_execution_binding(handoff.clone(), &self.pending_delivery);
        Ok(WorthQueryAdmittedRuntimeEffectWriteIntent {
            runtime: self.runtime,
            review: self.review,
            handoff,
            execution_binding,
            pending_delivery: self.pending_delivery,
        })
    }

    pub fn execute(
        self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<WorthQueryEffectIntentReceipt, WorthQueryRuntimeError> {
        self.admit()?.execute(execution)
    }
}

pub struct WorthQueryAdmittedRuntimeEffectWriteIntent<'a> {
    runtime: &'a mut WorthQueryRuntime,
    review: WorthQueryRuntimeIntentAdmissionReviewData,
    handoff: WorthQueryEffectTriggeredIntentExecutionHandoff,
    execution_binding: WorthQueryEffectTriggeredIntentExecutionBinding,
    pending_delivery: WorthQueryEffectDelivery,
}

impl<'a> WorthQueryAdmittedRuntimeEffectWriteIntent<'a> {
    pub fn request(&self) -> &WorthQueryRawIntentAdmissionRequest {
        self.review.request()
    }

    pub fn eligibility(&self) -> &WorthQueryIntentAdmissionEligibility {
        self.review.eligibility()
    }

    pub fn decision(&self) -> &WorthQueryIntentAdmissionDecision {
        self.review.decision()
    }

    pub fn handoff(&self) -> &WorthQueryEffectTriggeredIntentExecutionHandoff {
        &self.handoff
    }

    pub fn execution_binding(&self) -> &WorthQueryEffectTriggeredIntentExecutionBinding {
        &self.execution_binding
    }

    pub fn pending_delivery(&self) -> &WorthQueryEffectDelivery {
        &self.pending_delivery
    }

    pub fn execute(
        self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<WorthQueryEffectIntentReceipt, WorthQueryRuntimeError> {
        self.runtime
            .execute_effect_intent_execution_binding(execution, self.execution_binding)
    }
}
