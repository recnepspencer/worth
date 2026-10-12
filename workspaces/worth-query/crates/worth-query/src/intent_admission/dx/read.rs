use crate::intent_admission::dx::WorthQueryRuntimeIntentAdmissionReviewData;
use crate::intent_admission::{
    WorthQueryIntentAdmissionDecision, WorthQueryIntentAdmissionEligibility,
    WorthQueryIntentDecisionTraceEnvelope, WorthQueryRawIntentAdmissionRequest,
    WorthQueryReadExecutionBinding, WorthQueryReadExecutionHandoff,
};
use crate::query_context::ScopedQueryBasisContext;
#[cfg(test)]
use crate::runtime::WorthQueryAdmittedGraphReadAccessPlan;
use crate::runtime::{
    WorthQueryCountResult, WorthQueryGraphIndexInventoryMatchReport,
    WorthQueryGraphReadAccessAdmission, WorthQueryGraphReadAccessAuthorityContext,
    WorthQueryGraphReadAccessPlanExplanation, WorthQueryIntentConsumerInspection,
    WorthQueryReadFamily, WorthQueryReadResult, WorthQueryRuntimeError, WorthQueryWorkspace,
};

pub struct WorthQueryWorkspaceReadIntentAuthoring<'a> {
    workspace: &'a mut WorthQueryWorkspace,
    read_family: WorthQueryReadFamily,
    basis_context: Option<ScopedQueryBasisContext>,
    graph_read_authority: Option<WorthQueryGraphReadAccessAuthorityContext>,
}

impl<'a> WorthQueryWorkspaceReadIntentAuthoring<'a> {
    pub(crate) fn new(
        workspace: &'a mut WorthQueryWorkspace,
        read_family: WorthQueryReadFamily,
        basis_context: Option<ScopedQueryBasisContext>,
        graph_read_authority: Option<WorthQueryGraphReadAccessAuthorityContext>,
    ) -> Self {
        Self {
            workspace,
            read_family,
            basis_context,
            graph_read_authority,
        }
    }

    pub fn review(
        self,
    ) -> Result<WorthQueryWorkspaceReadIntentAdmissionReview<'a>, WorthQueryRuntimeError> {
        let review = self
            .workspace
            .review_read_execution(self.read_family, self.basis_context)?;
        Ok(WorthQueryWorkspaceReadIntentAdmissionReview {
            workspace: self.workspace,
            review,
            graph_read_authority: self.graph_read_authority,
        })
    }

    pub fn admit(
        self,
    ) -> Result<WorthQueryAdmittedWorkspaceReadIntent<'a>, WorthQueryRuntimeError> {
        self.review()?.admit()
    }

    pub fn execute(
        self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<WorthQueryReadResult, WorthQueryRuntimeError> {
        self.admit()?.execute(execution)
    }
}

pub struct WorthQueryWorkspaceReadIntentAdmissionReview<'a> {
    workspace: &'a mut WorthQueryWorkspace,
    review: WorthQueryRuntimeIntentAdmissionReviewData,
    graph_read_authority: Option<WorthQueryGraphReadAccessAuthorityContext>,
}

impl<'a> WorthQueryWorkspaceReadIntentAdmissionReview<'a> {
    pub fn request(&self) -> &WorthQueryRawIntentAdmissionRequest {
        self.review.request()
    }

    pub fn eligibility(&self) -> &WorthQueryIntentAdmissionEligibility {
        self.review.eligibility()
    }

    pub fn decision(&self) -> &WorthQueryIntentAdmissionDecision {
        self.review.decision()
    }

    pub fn admitted_handoff(&self) -> Option<WorthQueryReadExecutionHandoff> {
        self.workspace
            .resolve_reviewed_admitted_read_execution_handoff(self.review.clone())
            .ok()
    }

    pub fn graph_read_access_admission(
        &self,
    ) -> Result<WorthQueryGraphReadAccessAdmission, WorthQueryRuntimeError> {
        let handoff = self
            .workspace
            .resolve_reviewed_admitted_read_execution_handoff(self.review.clone())
            .map_err(|_| {
                self.workspace
                    .read_execution_non_admitted_error(&self.review)
            })?;
        admit_graph_read_access_for_review_authority(
            self.workspace,
            handoff.read_family(),
            self.graph_read_authority.as_ref(),
        )
        .map_err(|error| {
            WorthQueryRuntimeError::ReadCompositionDenied(
                crate::runtime::WorthQueryReadDenial::new(
                    crate::runtime::WorthQueryReadDenialKind::AuthoringDenied,
                    error.as_str(),
                ),
            )
        })
    }

    #[cfg(test)]
    pub(crate) fn graph_read_access_plan(
        &self,
    ) -> Result<WorthQueryAdmittedGraphReadAccessPlan, WorthQueryRuntimeError> {
        let handoff = self
            .workspace
            .resolve_reviewed_admitted_read_execution_handoff(self.review.clone())
            .map_err(|_| {
                self.workspace
                    .read_execution_non_admitted_error(&self.review)
            })?;
        let admission = admit_graph_read_access_for_review_authority(
            self.workspace,
            handoff.read_family(),
            self.graph_read_authority.as_ref(),
        )
        .map_err(|error| {
            WorthQueryRuntimeError::ReadCompositionDenied(
                crate::runtime::WorthQueryReadDenial::new(
                    crate::runtime::WorthQueryReadDenialKind::AuthoringDenied,
                    error.as_str(),
                ),
            )
        })?;
        WorthQueryAdmittedGraphReadAccessPlan::from_admission(admission.clone()).ok_or_else(|| {
            let detail = admission
                .denial()
                .map(|denial| denial.kind().as_str())
                .unwrap_or("graph_read_access_not_admitted");
                WorthQueryRuntimeError::ReadCompositionDenied(
                    crate::runtime::WorthQueryReadDenial::new(
                        crate::runtime::WorthQueryReadDenialKind::BasisPreflightDenied,
                        detail,
                    )
                    .with_graph_read_persistent_artifact_audit_for_admission(&admission)
                    .with_graph_read_access_admission(admission)
                    .with_graph_read_access_execution_counters(
                        crate::runtime::WorthQueryGraphReadAccessExecutionCounters::pre_execution_denial(
                    ),
                ),
            )
        })
    }

    pub fn graph_index_support(
        &self,
    ) -> Result<WorthQueryGraphIndexInventoryMatchReport, WorthQueryRuntimeError> {
        Ok(self
            .graph_read_access_admission()?
            .graph_index_inventory_match_report()
            .clone())
    }

    pub fn graph_read_access_explanation(
        &self,
    ) -> Result<WorthQueryGraphReadAccessPlanExplanation, WorthQueryRuntimeError> {
        Ok(WorthQueryGraphReadAccessPlanExplanation::from_admission(
            &self.graph_read_access_admission()?,
        ))
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
    ) -> Result<WorthQueryAdmittedWorkspaceReadIntent<'a>, WorthQueryRuntimeError> {
        let handoff = self
            .workspace
            .resolve_reviewed_admitted_read_execution_handoff(self.review.clone())
            .map_err(|_| {
                self.workspace
                    .read_execution_non_admitted_error(&self.review)
            })?;
        let execution_binding = self
            .workspace
            .into_runtime_read_execution_binding_in_authority(
                handoff.clone(),
                self.graph_read_authority.as_ref(),
            )?;
        Ok(WorthQueryAdmittedWorkspaceReadIntent {
            workspace: self.workspace,
            review: self.review,
            handoff,
            execution_binding,
        })
    }

    pub fn execute(
        self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<WorthQueryReadResult, WorthQueryRuntimeError> {
        self.admit()?.execute(execution)
    }
}

fn admit_graph_read_access_for_review_authority(
    workspace: &WorthQueryWorkspace,
    family: &WorthQueryReadFamily,
    authority: Option<&WorthQueryGraphReadAccessAuthorityContext>,
) -> Result<
    WorthQueryGraphReadAccessAdmission,
    crate::runtime::WorthQueryGraphReadAccessShapeExplanationError,
> {
    match authority {
        Some(authority) => {
            workspace.admit_graph_read_access_for_family_in_authority(family, authority)
        }
        None => workspace.admit_graph_read_access_for_family(family),
    }
}

pub struct WorthQueryAdmittedWorkspaceReadIntent<'a> {
    workspace: &'a mut WorthQueryWorkspace,
    review: WorthQueryRuntimeIntentAdmissionReviewData,
    handoff: WorthQueryReadExecutionHandoff,
    execution_binding: WorthQueryReadExecutionBinding,
}

impl<'a> WorthQueryAdmittedWorkspaceReadIntent<'a> {
    pub fn request(&self) -> &WorthQueryRawIntentAdmissionRequest {
        self.review.request()
    }

    pub fn eligibility(&self) -> &WorthQueryIntentAdmissionEligibility {
        self.review.eligibility()
    }

    pub fn decision(&self) -> &WorthQueryIntentAdmissionDecision {
        self.review.decision()
    }

    pub fn handoff(&self) -> &WorthQueryReadExecutionHandoff {
        &self.handoff
    }

    pub fn execution_binding(&self) -> &WorthQueryReadExecutionBinding {
        &self.execution_binding
    }

    pub fn execute(
        self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<WorthQueryReadResult, WorthQueryRuntimeError> {
        self.workspace
            .execute_bound_read_execution(execution, self.execution_binding)
    }

    pub(crate) fn execute_count(
        self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<WorthQueryCountResult, WorthQueryRuntimeError> {
        self.workspace
            .execute_bound_count_read_execution(execution, self.execution_binding)
    }

    pub(crate) fn open_managed_live<T>(
        self,
        request_execution: &worth_execution::ExecutionRequest<'_, '_>,
        name: impl Into<String>,
    ) -> Result<crate::runtime::WorthQueryLiveView<T>, WorthQueryRuntimeError> {
        self.workspace
            .open_admitted_live_binding(request_execution, name, self.execution_binding)
    }
}
