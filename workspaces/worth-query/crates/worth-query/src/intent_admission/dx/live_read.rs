use crate::intent_admission::dx::WorthQueryRuntimeIntentAdmissionReviewData;
use crate::intent_admission::{
    WorthQueryIntentAdmissionDecision, WorthQueryIntentAdmissionEligibility,
    WorthQueryIntentDecisionTraceEnvelope, WorthQueryLiveReadExecutionBinding,
    WorthQueryLiveReadExecutionHandoff,
};
use crate::runtime::{
    WorthQueryIntentConsumerInspection, WorthQueryLiveReadResult, WorthQueryLiveView,
    WorthQueryRuntimeError, WorthQueryWorkspace,
};

pub struct WorthQueryWorkspaceLiveReadIntentAuthoring<'a, T> {
    workspace: &'a mut WorthQueryWorkspace,
    live_view: WorthQueryLiveView<T>,
}

impl<'a, T> WorthQueryWorkspaceLiveReadIntentAuthoring<'a, T> {
    pub(crate) fn new(
        workspace: &'a mut WorthQueryWorkspace,
        live_view: WorthQueryLiveView<T>,
    ) -> Self {
        Self {
            workspace,
            live_view,
        }
    }

    pub fn review(
        self,
    ) -> Result<WorthQueryWorkspaceLiveReadIntentAdmissionReview<'a>, WorthQueryRuntimeError> {
        let review = self.workspace.review_live_read_execution(self.live_view)?;
        Ok(WorthQueryWorkspaceLiveReadIntentAdmissionReview {
            workspace: self.workspace,
            review,
        })
    }

    pub fn admit(
        self,
    ) -> Result<WorthQueryAdmittedWorkspaceLiveReadIntent<'a>, WorthQueryRuntimeError> {
        self.review()?.admit()
    }

    pub fn execute(
        self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<WorthQueryLiveReadResult, WorthQueryRuntimeError> {
        self.admit()?.execute(execution)
    }
}

pub struct WorthQueryWorkspaceLiveReadIntentAdmissionReview<'a> {
    workspace: &'a mut WorthQueryWorkspace,
    review: WorthQueryRuntimeIntentAdmissionReviewData,
}

impl<'a> WorthQueryWorkspaceLiveReadIntentAdmissionReview<'a> {
    pub fn request(&self) -> &crate::intent_admission::WorthQueryRawIntentAdmissionRequest {
        self.review.request()
    }

    pub fn eligibility(&self) -> &WorthQueryIntentAdmissionEligibility {
        self.review.eligibility()
    }

    pub fn decision(&self) -> &WorthQueryIntentAdmissionDecision {
        self.review.decision()
    }

    pub fn admitted_handoff(&self) -> Option<WorthQueryLiveReadExecutionHandoff> {
        self.workspace
            .resolve_reviewed_admitted_live_read_execution_handoff(self.review.clone())
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
    ) -> Result<WorthQueryAdmittedWorkspaceLiveReadIntent<'a>, WorthQueryRuntimeError> {
        let handoff = self
            .workspace
            .resolve_reviewed_admitted_live_read_execution_handoff(self.review.clone())
            .map_err(|_| {
                self.workspace
                    .live_read_execution_non_admitted_error(&self.review)
            })?;
        let execution_binding = self
            .workspace
            .into_runtime_live_read_execution_binding(handoff.clone())?;
        Ok(WorthQueryAdmittedWorkspaceLiveReadIntent {
            workspace: self.workspace,
            review: self.review,
            handoff,
            execution_binding,
        })
    }

    pub fn execute(
        self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<WorthQueryLiveReadResult, WorthQueryRuntimeError> {
        self.admit()?.execute(execution)
    }
}

pub struct WorthQueryAdmittedWorkspaceLiveReadIntent<'a> {
    workspace: &'a mut WorthQueryWorkspace,
    review: WorthQueryRuntimeIntentAdmissionReviewData,
    handoff: WorthQueryLiveReadExecutionHandoff,
    execution_binding: WorthQueryLiveReadExecutionBinding,
}

impl<'a> WorthQueryAdmittedWorkspaceLiveReadIntent<'a> {
    pub fn request(&self) -> &crate::intent_admission::WorthQueryRawIntentAdmissionRequest {
        self.review.request()
    }

    pub fn eligibility(&self) -> &WorthQueryIntentAdmissionEligibility {
        self.review.eligibility()
    }

    pub fn decision(&self) -> &WorthQueryIntentAdmissionDecision {
        self.review.decision()
    }

    pub fn handoff(&self) -> &WorthQueryLiveReadExecutionHandoff {
        &self.handoff
    }

    pub fn execution_binding(&self) -> &WorthQueryLiveReadExecutionBinding {
        &self.execution_binding
    }

    pub fn execute(
        self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<WorthQueryLiveReadResult, WorthQueryRuntimeError> {
        self.workspace
            .execute_bound_live_read_execution(execution, self.execution_binding)
    }
}
