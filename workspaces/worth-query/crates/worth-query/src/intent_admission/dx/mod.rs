mod authoritative;
mod basis_projection;
mod batch_mutation;
mod effect;
mod inspection;
mod live_read;
mod mutation;
mod read;
mod routing;
mod unified_inspection;

pub(crate) const INTENT_ADMISSION_DX_MODULE_ROOT: &str = "intent_admission/dx/mod.rs";
pub(crate) const INTENT_ADMISSION_DX_CHILD_MODULES: &[&str] = &[
    "authoritative",
    "basis_projection",
    "batch_mutation",
    "effect",
    "inspection",
    "live_read",
    "mutation",
    "read",
    "routing",
    "unified_inspection",
];
pub(crate) const INTENT_ADMISSION_DX_EXPORTED_SURFACE: &[&str] = &[
    "WorthQueryAdmittedRuntimeIntent",
    "WorthQueryRuntimeIntentAdmissionReview",
    "WorthQueryRuntimeIntentAuthoring",
    "worth_query_basis_observation_intent",
    "worth_query_projection_consumption_intent",
    "WorthQueryBasisObservationAdmittedIntent",
    "WorthQueryBasisObservationIntentAuthoring",
    "WorthQueryBasisObservationIntentReview",
    "WorthQueryProjectionConsumptionAdmittedIntent",
    "WorthQueryProjectionConsumptionIntentAuthoring",
    "WorthQueryProjectionConsumptionIntentReview",
    "WorthQueryAdmittedRuntimeWriteBatchIntent",
    "WorthQueryRuntimeWriteBatchIntentAdmissionReview",
    "WorthQueryRuntimeWriteBatchIntentAuthoring",
    "WorthQueryAdmittedRuntimeEffectWriteIntent",
    "WorthQueryRuntimeEffectWriteIntentAdmissionReview",
    "WorthQueryRuntimeEffectWriteIntentAuthoring",
    "WorthQueryAdmittedWorkspaceDerivedInspectionIntent",
    "WorthQueryAdmittedWorkspaceDerivedMaterializationIntent",
    "WorthQueryWorkspaceDerivedInspectionIntentAdmissionReview",
    "WorthQueryWorkspaceDerivedInspectionIntentAuthoring",
    "WorthQueryWorkspaceDerivedMaterializationIntentAdmissionReview",
    "WorthQueryWorkspaceDerivedMaterializationIntentAuthoring",
    "WorthQueryAdmittedWorkspaceLiveReadIntent",
    "WorthQueryWorkspaceLiveReadIntentAdmissionReview",
    "WorthQueryWorkspaceLiveReadIntentAuthoring",
    "WorthQueryAdmittedRuntimeWriteIntent",
    "WorthQueryRuntimeWriteIntentAdmissionReview",
    "WorthQueryRuntimeWriteIntentAuthoring",
    "WorthQueryAdmittedWorkspaceReadIntent",
    "WorthQueryWorkspaceReadIntentAdmissionReview",
    "WorthQueryWorkspaceReadIntentAuthoring",
    "WorthQueryAdmittedRuntimeExistingTruthProbeIntent",
    "WorthQueryRuntimeExistingTruthProbeIntentAdmissionReview",
    "WorthQueryRuntimeExistingTruthProbeIntentAuthoring",
    "WorthQueryAdmittedRuntimeInspectionIntent",
    "WorthQueryRuntimeInspectionIntentAdmissionReview",
    "WorthQueryRuntimeInspectionIntentAuthoring",
];

use super::{
    admit_runtime_intent_request, WorthQueryAdmittedIntentPlan, WorthQueryIntentAdmissionDecision,
    WorthQueryIntentAdmissionEligibility, WorthQueryIntentDecisionTraceEnvelope,
    WorthQueryIntentNonAdmittedStop, WorthQueryIntentViolationDecision,
    WorthQueryRawIntentAdmissionRequest,
};

pub use authoritative::{
    WorthQueryAdmittedRuntimeIntent, WorthQueryRuntimeIntentAdmissionReview,
    WorthQueryRuntimeIntentAuthoring,
};
pub use basis_projection::{
    worth_query_basis_observation_intent, worth_query_projection_consumption_intent,
    WorthQueryBasisObservationAdmittedIntent, WorthQueryBasisObservationIntentAuthoring,
    WorthQueryBasisObservationIntentReview, WorthQueryProjectionConsumptionAdmittedIntent,
    WorthQueryProjectionConsumptionIntentAuthoring, WorthQueryProjectionConsumptionIntentReview,
};
pub use batch_mutation::WorthQueryRuntimeWriteBatchIntentAuthoring;
pub use effect::{
    WorthQueryAdmittedRuntimeEffectWriteIntent, WorthQueryRuntimeEffectWriteIntentAdmissionReview,
    WorthQueryRuntimeEffectWriteIntentAuthoring,
};
pub use inspection::{
    WorthQueryWorkspaceDerivedInspectionIntentAdmissionReview,
    WorthQueryWorkspaceDerivedInspectionIntentAuthoring,
    WorthQueryWorkspaceDerivedMaterializationIntentAdmissionReview,
    WorthQueryWorkspaceDerivedMaterializationIntentAuthoring,
};
pub use live_read::{
    WorthQueryWorkspaceLiveReadIntentAdmissionReview, WorthQueryWorkspaceLiveReadIntentAuthoring,
};
pub use mutation::WorthQueryRuntimeWriteIntentAuthoring;
pub use read::{
    WorthQueryAdmittedWorkspaceReadIntent, WorthQueryWorkspaceReadIntentAdmissionReview,
    WorthQueryWorkspaceReadIntentAuthoring,
};
pub use routing::{
    WorthQueryAdmittedRuntimeExistingTruthProbeIntent,
    WorthQueryRuntimeExistingTruthProbeIntentAdmissionReview,
    WorthQueryRuntimeExistingTruthProbeIntentAuthoring,
};
pub use unified_inspection::{
    WorthQueryAdmittedRuntimeInspectionIntent, WorthQueryRuntimeInspectionIntentAdmissionReview,
    WorthQueryRuntimeInspectionIntentAuthoring,
};

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct WorthQueryRuntimeIntentAdmissionReviewData {
    request: WorthQueryRawIntentAdmissionRequest,
    eligibility: WorthQueryIntentAdmissionEligibility,
    decision: WorthQueryIntentAdmissionDecision,
    non_admitted_trace: Option<WorthQueryIntentDecisionTraceEnvelope>,
}

impl WorthQueryRuntimeIntentAdmissionReviewData {
    pub(crate) fn from_request(request: WorthQueryRawIntentAdmissionRequest) -> Self {
        let decision = admit_runtime_intent_request(request.clone());
        Self::from_decision(request, decision)
    }

    pub(crate) fn from_decision(
        request: WorthQueryRawIntentAdmissionRequest,
        decision: WorthQueryIntentAdmissionDecision,
    ) -> Self {
        let eligibility = WorthQueryIntentAdmissionEligibility::from_request(request.clone());
        let non_admitted_trace = match &decision {
            WorthQueryIntentAdmissionDecision::Admitted(_) => None,
            WorthQueryIntentAdmissionDecision::Advisory(advisory) => {
                Some(WorthQueryIntentDecisionTraceEnvelope::for_request_advisory(
                    &request,
                    &eligibility.trace_evidence(),
                    advisory,
                ))
            }
            WorthQueryIntentAdmissionDecision::Violation(violation) => Some(
                WorthQueryIntentDecisionTraceEnvelope::for_request_violation(
                    &request,
                    &eligibility.trace_evidence(),
                    violation,
                ),
            ),
        };
        Self {
            request,
            eligibility,
            decision,
            non_admitted_trace,
        }
    }

    pub(crate) fn admitted_plan(&self) -> Option<&WorthQueryAdmittedIntentPlan> {
        match &self.decision {
            WorthQueryIntentAdmissionDecision::Admitted(plan) => Some(plan),
            WorthQueryIntentAdmissionDecision::Advisory(_)
            | WorthQueryIntentAdmissionDecision::Violation(_) => None,
        }
    }

    pub(crate) fn non_admitted_stop(&self) -> Option<WorthQueryIntentNonAdmittedStop> {
        self.decision.clone().into_non_admitted_stop()
    }

    pub(crate) fn request(&self) -> &WorthQueryRawIntentAdmissionRequest {
        &self.request
    }

    pub(crate) fn decision(&self) -> &WorthQueryIntentAdmissionDecision {
        &self.decision
    }

    pub(crate) fn eligibility(&self) -> &WorthQueryIntentAdmissionEligibility {
        &self.eligibility
    }

    pub(crate) fn decision_trace_envelope(&self) -> Option<&WorthQueryIntentDecisionTraceEnvelope> {
        self.non_admitted_trace.as_ref()
    }
}

pub(crate) fn non_admitted_runtime_violation(
    review: &WorthQueryRuntimeIntentAdmissionReviewData,
) -> WorthQueryIntentViolationDecision {
    review
        .non_admitted_stop()
        .expect("non-admitted runtime review should preserve a stop artifact")
        .into_violation_stop()
        .violation()
        .clone()
}
#[cfg(test)]
#[path = "mod_inline_type_budget.rs"]
pub(crate) mod inline_type_budget;
