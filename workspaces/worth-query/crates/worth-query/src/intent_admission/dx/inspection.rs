use crate::intent_admission::dx::WorthQueryRuntimeIntentAdmissionReviewData;
use crate::intent_admission::{
    WorthQueryDerivedInspectionExecutionBinding, WorthQueryDerivedInspectionExecutionHandoff,
    WorthQueryDerivedMaterializationExecutionBinding,
    WorthQueryDerivedMaterializationExecutionHandoff, WorthQueryIntentAdmissionDecision,
    WorthQueryIntentAdmissionEligibility, WorthQueryIntentDecisionTraceEnvelope,
    WorthQueryRawIntentAdmissionRequest,
};
use crate::runtime::{
    WorthQueryDerivedInspectionResult, WorthQueryDerivedMaterializationResult,
    WorthQueryIntentConsumerInspection, WorthQueryRuntimeError, WorthQueryWorkspace,
};

pub struct WorthQueryWorkspaceDerivedMaterializationIntentAuthoring<'a> {
    workspace: &'a mut WorthQueryWorkspace,
    view_name: String,
}

impl<'a> WorthQueryWorkspaceDerivedMaterializationIntentAuthoring<'a> {
    pub(crate) fn new(workspace: &'a mut WorthQueryWorkspace, view_name: String) -> Self {
        Self {
            workspace,
            view_name,
        }
    }

    pub fn review(
        self,
    ) -> Result<
        WorthQueryWorkspaceDerivedMaterializationIntentAdmissionReview<'a>,
        WorthQueryRuntimeError,
    > {
        let review = self
            .workspace
            .review_derived_materialization(self.view_name)?;
        Ok(
            WorthQueryWorkspaceDerivedMaterializationIntentAdmissionReview {
                workspace: self.workspace,
                review,
            },
        )
    }

    pub fn admit(
        self,
    ) -> Result<WorthQueryAdmittedWorkspaceDerivedMaterializationIntent<'a>, WorthQueryRuntimeError>
    {
        self.review()?.admit()
    }

    pub fn execute(
        self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<WorthQueryDerivedMaterializationResult, WorthQueryRuntimeError> {
        self.admit()?.execute(execution)
    }
}

pub struct WorthQueryWorkspaceDerivedMaterializationIntentAdmissionReview<'a> {
    workspace: &'a mut WorthQueryWorkspace,
    review: WorthQueryRuntimeIntentAdmissionReviewData,
}

impl<'a> WorthQueryWorkspaceDerivedMaterializationIntentAdmissionReview<'a> {
    pub fn request(&self) -> &WorthQueryRawIntentAdmissionRequest {
        self.review.request()
    }

    pub fn eligibility(&self) -> &WorthQueryIntentAdmissionEligibility {
        self.review.eligibility()
    }

    pub fn decision(&self) -> &WorthQueryIntentAdmissionDecision {
        self.review.decision()
    }

    pub fn admitted_handoff(&self) -> Option<WorthQueryDerivedMaterializationExecutionHandoff> {
        self.workspace
            .resolve_reviewed_admitted_derived_materialization_handoff(self.review.clone())
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
    ) -> Result<WorthQueryAdmittedWorkspaceDerivedMaterializationIntent<'a>, WorthQueryRuntimeError>
    {
        let handoff = self
            .workspace
            .resolve_reviewed_admitted_derived_materialization_handoff(self.review.clone())
            .map_err(|_| {
                self.workspace
                    .derived_materialization_non_admitted_error(&self.review)
            })?;
        let execution_binding = self
            .workspace
            .into_runtime_derived_materialization_binding(handoff.clone());
        Ok(WorthQueryAdmittedWorkspaceDerivedMaterializationIntent {
            workspace: self.workspace,
            review: self.review,
            handoff,
            execution_binding,
        })
    }
}

pub struct WorthQueryAdmittedWorkspaceDerivedMaterializationIntent<'a> {
    workspace: &'a mut WorthQueryWorkspace,
    review: WorthQueryRuntimeIntentAdmissionReviewData,
    handoff: WorthQueryDerivedMaterializationExecutionHandoff,
    execution_binding: WorthQueryDerivedMaterializationExecutionBinding,
}

impl<'a> WorthQueryAdmittedWorkspaceDerivedMaterializationIntent<'a> {
    pub fn request(&self) -> &WorthQueryRawIntentAdmissionRequest {
        self.review.request()
    }

    pub fn eligibility(&self) -> &WorthQueryIntentAdmissionEligibility {
        self.review.eligibility()
    }

    pub fn decision(&self) -> &WorthQueryIntentAdmissionDecision {
        self.review.decision()
    }

    pub fn handoff(&self) -> &WorthQueryDerivedMaterializationExecutionHandoff {
        &self.handoff
    }

    pub fn execution_binding(&self) -> &WorthQueryDerivedMaterializationExecutionBinding {
        &self.execution_binding
    }

    pub fn execute(
        self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<WorthQueryDerivedMaterializationResult, WorthQueryRuntimeError> {
        self.workspace
            .execute_bound_derived_materialization(execution, self.execution_binding)
    }
}

pub struct WorthQueryWorkspaceDerivedInspectionIntentAuthoring<'a> {
    workspace: &'a mut WorthQueryWorkspace,
    view_name: String,
}

impl<'a> WorthQueryWorkspaceDerivedInspectionIntentAuthoring<'a> {
    pub(crate) fn new(workspace: &'a mut WorthQueryWorkspace, view_name: String) -> Self {
        Self {
            workspace,
            view_name,
        }
    }

    pub fn review(
        self,
    ) -> Result<WorthQueryWorkspaceDerivedInspectionIntentAdmissionReview<'a>, WorthQueryRuntimeError>
    {
        let review = self.workspace.review_derived_inspection(self.view_name)?;
        Ok(WorthQueryWorkspaceDerivedInspectionIntentAdmissionReview {
            workspace: self.workspace,
            review,
        })
    }

    pub fn admit(
        self,
    ) -> Result<WorthQueryAdmittedWorkspaceDerivedInspectionIntent<'a>, WorthQueryRuntimeError>
    {
        self.review()?.admit()
    }

    pub fn execute(
        self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<WorthQueryDerivedInspectionResult, WorthQueryRuntimeError> {
        self.admit()?.execute(execution)
    }
}

pub struct WorthQueryWorkspaceDerivedInspectionIntentAdmissionReview<'a> {
    workspace: &'a mut WorthQueryWorkspace,
    review: WorthQueryRuntimeIntentAdmissionReviewData,
}

impl<'a> WorthQueryWorkspaceDerivedInspectionIntentAdmissionReview<'a> {
    pub fn request(&self) -> &WorthQueryRawIntentAdmissionRequest {
        self.review.request()
    }

    pub fn eligibility(&self) -> &WorthQueryIntentAdmissionEligibility {
        self.review.eligibility()
    }

    pub fn decision(&self) -> &WorthQueryIntentAdmissionDecision {
        self.review.decision()
    }

    pub fn admitted_handoff(&self) -> Option<WorthQueryDerivedInspectionExecutionHandoff> {
        self.workspace
            .resolve_reviewed_admitted_derived_inspection_handoff(self.review.clone())
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
    ) -> Result<WorthQueryAdmittedWorkspaceDerivedInspectionIntent<'a>, WorthQueryRuntimeError>
    {
        let handoff = self
            .workspace
            .resolve_reviewed_admitted_derived_inspection_handoff(self.review.clone())
            .map_err(|_| {
                self.workspace
                    .derived_inspection_non_admitted_error(&self.review)
            })?;
        let execution_binding = self
            .workspace
            .into_runtime_derived_inspection_binding(handoff.clone());
        Ok(WorthQueryAdmittedWorkspaceDerivedInspectionIntent {
            workspace: self.workspace,
            review: self.review,
            handoff,
            execution_binding,
        })
    }
}

pub struct WorthQueryAdmittedWorkspaceDerivedInspectionIntent<'a> {
    workspace: &'a mut WorthQueryWorkspace,
    review: WorthQueryRuntimeIntentAdmissionReviewData,
    handoff: WorthQueryDerivedInspectionExecutionHandoff,
    execution_binding: WorthQueryDerivedInspectionExecutionBinding,
}

impl<'a> WorthQueryAdmittedWorkspaceDerivedInspectionIntent<'a> {
    pub fn request(&self) -> &WorthQueryRawIntentAdmissionRequest {
        self.review.request()
    }

    pub fn eligibility(&self) -> &WorthQueryIntentAdmissionEligibility {
        self.review.eligibility()
    }

    pub fn decision(&self) -> &WorthQueryIntentAdmissionDecision {
        self.review.decision()
    }

    pub fn handoff(&self) -> &WorthQueryDerivedInspectionExecutionHandoff {
        &self.handoff
    }

    pub fn execution_binding(&self) -> &WorthQueryDerivedInspectionExecutionBinding {
        &self.execution_binding
    }

    pub fn execute(
        self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<WorthQueryDerivedInspectionResult, WorthQueryRuntimeError> {
        self.workspace
            .execute_bound_derived_inspection(execution, self.execution_binding)
    }
}
