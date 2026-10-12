use crate::runtime::{
    WorthQueryBatchWriteReceipt, WorthQueryRuntime, WorthQueryRuntimeError, WorthQueryWriteCommand,
};

use super::super::{
    WorthQueryAuthoritativeMutationBatchExecutionHandoff, WorthQueryIntentAdmissionDecision,
};
use super::WorthQueryRuntimeIntentAdmissionReviewData;

pub struct WorthQueryRuntimeWriteBatchIntentAuthoring<'a> {
    runtime: &'a mut WorthQueryRuntime,
    commands: Vec<WorthQueryWriteCommand>,
}

pub struct WorthQueryRuntimeWriteBatchIntentAdmissionReview<'a> {
    runtime: &'a mut WorthQueryRuntime,
    review: WorthQueryRuntimeIntentAdmissionReviewData,
}

pub struct WorthQueryAdmittedRuntimeWriteBatchIntent<'a> {
    runtime: &'a mut WorthQueryRuntime,
    handoff: WorthQueryAuthoritativeMutationBatchExecutionHandoff,
}

impl<'a> WorthQueryRuntimeWriteBatchIntentAuthoring<'a> {
    pub(crate) fn new(
        runtime: &'a mut WorthQueryRuntime,
        commands: Vec<WorthQueryWriteCommand>,
    ) -> Self {
        Self { runtime, commands }
    }

    pub fn review(
        self,
    ) -> Result<WorthQueryRuntimeWriteBatchIntentAdmissionReview<'a>, WorthQueryRuntimeError> {
        let review = self
            .runtime
            .review_authoritative_runtime_write_batch(self.commands)?;
        Ok(WorthQueryRuntimeWriteBatchIntentAdmissionReview {
            runtime: self.runtime,
            review,
        })
    }

    pub fn execute(
        self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<WorthQueryBatchWriteReceipt, WorthQueryRuntimeError> {
        self.review()?.admit()?.execute(execution)
    }
}

impl<'a> WorthQueryRuntimeWriteBatchIntentAdmissionReview<'a> {
    pub fn request(&self) -> &super::super::WorthQueryRawIntentAdmissionRequest {
        self.review.request()
    }

    pub fn eligibility(&self) -> &super::super::WorthQueryIntentAdmissionEligibility {
        self.review.eligibility()
    }

    pub fn decision(&self) -> &WorthQueryIntentAdmissionDecision {
        self.review.decision()
    }

    pub fn decision_trace_envelope(
        &self,
    ) -> Option<&super::super::WorthQueryIntentDecisionTraceEnvelope> {
        self.review.decision_trace_envelope()
    }

    pub fn admit(
        self,
    ) -> Result<WorthQueryAdmittedRuntimeWriteBatchIntent<'a>, WorthQueryRuntimeError> {
        let handoff = self
            .runtime
            .resolve_reviewed_admitted_authoritative_write_batch_handoff(self.review)?;
        Ok(WorthQueryAdmittedRuntimeWriteBatchIntent {
            runtime: self.runtime,
            handoff,
        })
    }
}

impl<'a> WorthQueryAdmittedRuntimeWriteBatchIntent<'a> {
    pub fn handoff(&self) -> &WorthQueryAuthoritativeMutationBatchExecutionHandoff {
        &self.handoff
    }

    pub fn execute(
        self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<WorthQueryBatchWriteReceipt, WorthQueryRuntimeError> {
        let binding = self
            .runtime
            .prepare_authoritative_mutation_batch_execution_binding(self.handoff);
        self.runtime
            .execute_authoritative_mutation_batch_execution_binding(execution, binding)
    }
}
