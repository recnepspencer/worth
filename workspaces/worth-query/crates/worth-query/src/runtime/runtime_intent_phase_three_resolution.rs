use super::*;
use crate::intent_admission::{
    WorthQueryAuthoritativeIntentExecutionHandoff, WorthQueryEffectTriggeredIntentExecutionHandoff,
    WorthQueryIntentAdmissionDecision, WorthQueryIntentViolationDecision,
};

impl WorthQueryRuntime {
    pub(crate) fn resolve_reviewed_admitted_authoritative_intent_handoff(
        &self,
        review: crate::intent_admission::dx::WorthQueryRuntimeIntentAdmissionReviewData,
    ) -> Result<WorthQueryAuthoritativeIntentExecutionHandoff, WorthQueryRuntimeError> {
        let declaration = review
            .request()
            .runtime_declaration()
            .expect("authoritative runtime phase-three review must preserve declaration")
            .clone();
        let non_admitted_trace = review.decision_trace_envelope().cloned();
        match review.decision().clone() {
            WorthQueryIntentAdmissionDecision::Admitted(
                crate::intent_admission::WorthQueryAdmittedIntentPlan::Authoritative(plan),
            ) => Ok(
                crate::intent_admission::WorthQueryAuthoritativeIntentExecutionHandoff::from_plan(
                    plan,
                ),
            ),
            WorthQueryIntentAdmissionDecision::Admitted(
                crate::intent_admission::WorthQueryAdmittedIntentPlan::EffectTriggered(_)
                | crate::intent_admission::WorthQueryAdmittedIntentPlan::AuthoritativeMutation(_)
                | crate::intent_admission::WorthQueryAdmittedIntentPlan::AuthoritativeMutationBatch(
                    _,
                )
                | crate::intent_admission::WorthQueryAdmittedIntentPlan::ReadExecution(_)
                | crate::intent_admission::WorthQueryAdmittedIntentPlan::LiveReadExecution(_)
                | crate::intent_admission::WorthQueryAdmittedIntentPlan::DerivedMaterialization(_)
                | crate::intent_admission::WorthQueryAdmittedIntentPlan::DerivedInspection(_)
                | crate::intent_admission::WorthQueryAdmittedIntentPlan::UnifiedInspection(_)
                | crate::intent_admission::WorthQueryAdmittedIntentPlan::ExistingTruthProbeRouting(_)
                | crate::intent_admission::WorthQueryAdmittedIntentPlan::BasisObservation(_)
                | crate::intent_admission::WorthQueryAdmittedIntentPlan::ProjectionConsumption(_),
            ) => Err(self.intent_violation_error(
                &declaration,
                phase_three_family_violation(
                    &review,
                    "authoritative-runtime-path-requires-authoritative-handoff-proof",
                ),
                None,
                non_admitted_trace,
                None,
            )),
            WorthQueryIntentAdmissionDecision::Advisory(advisory) => Err(self
                .intent_violation_error(
                    &declaration,
                    advisory.into_violation(),
                    None,
                    non_admitted_trace,
                    None,
                )),
            WorthQueryIntentAdmissionDecision::Violation(violation) => Err(self
                .intent_violation_error(&declaration, violation, None, non_admitted_trace, None)),
        }
    }

    pub(crate) fn resolve_reviewed_admitted_effect_intent_handoff(
        &self,
        review: crate::intent_admission::dx::WorthQueryRuntimeIntentAdmissionReviewData,
    ) -> Result<WorthQueryEffectTriggeredIntentExecutionHandoff, WorthQueryRuntimeError> {
        let declaration = review
            .request()
            .runtime_declaration()
            .expect("effect runtime phase-three review must preserve declaration")
            .clone();
        let non_admitted_trace = review.decision_trace_envelope().cloned();
        match review.decision().clone() {
            WorthQueryIntentAdmissionDecision::Admitted(
                crate::intent_admission::WorthQueryAdmittedIntentPlan::EffectTriggered(plan),
            ) => Ok(
                crate::intent_admission::WorthQueryEffectTriggeredIntentExecutionHandoff::from_plan(
                    plan,
                ),
            ),
            WorthQueryIntentAdmissionDecision::Admitted(
                crate::intent_admission::WorthQueryAdmittedIntentPlan::Authoritative(_)
                | crate::intent_admission::WorthQueryAdmittedIntentPlan::AuthoritativeMutation(_)
                | crate::intent_admission::WorthQueryAdmittedIntentPlan::AuthoritativeMutationBatch(
                    _,
                )
                | crate::intent_admission::WorthQueryAdmittedIntentPlan::ReadExecution(_)
                | crate::intent_admission::WorthQueryAdmittedIntentPlan::LiveReadExecution(_)
                | crate::intent_admission::WorthQueryAdmittedIntentPlan::DerivedMaterialization(_)
                | crate::intent_admission::WorthQueryAdmittedIntentPlan::DerivedInspection(_)
                | crate::intent_admission::WorthQueryAdmittedIntentPlan::UnifiedInspection(_)
                | crate::intent_admission::WorthQueryAdmittedIntentPlan::ExistingTruthProbeRouting(_)
                | crate::intent_admission::WorthQueryAdmittedIntentPlan::BasisObservation(_)
                | crate::intent_admission::WorthQueryAdmittedIntentPlan::ProjectionConsumption(_),
            ) => Err(self.intent_violation_error(
                &declaration,
                phase_three_family_violation(
                    &review,
                    "effect-runtime-path-requires-effect-handoff-proof",
                ),
                None,
                non_admitted_trace,
                None,
            )),
            WorthQueryIntentAdmissionDecision::Advisory(advisory) => Err(self
                .intent_violation_error(
                    &declaration,
                    advisory.into_violation(),
                    None,
                    non_admitted_trace,
                    None,
                )),
            WorthQueryIntentAdmissionDecision::Violation(violation) => Err(self
                .intent_violation_error(&declaration, violation, None, non_admitted_trace, None)),
        }
    }
}

fn phase_three_family_violation(
    review: &crate::intent_admission::dx::WorthQueryRuntimeIntentAdmissionReviewData,
    message: &'static str,
) -> WorthQueryIntentViolationDecision {
    WorthQueryIntentViolationDecision::new(
        review.request().family(),
        review.request().entrypoint(),
        "phase-three-family-proof",
        message,
        review.request().request_digest(),
        review.eligibility().eligibility_digest(),
    )
}
