use worth_proof::TransitionOutcome;

use crate::domain_capabilities::{
    WorthQueryAdmittedPlanContributionTargetBinding, WorthQueryDomainCapabilityTransitionOutcome,
    WorthQueryMaterializationReadySupportContribution,
};
use crate::intent_admission::{
    WorthQueryIntentAdmissionSupportTraceabilityReport,
    WorthQueryIntentAdmissionSupportTraceabilityRow,
};

pub fn materialize_intent_admission_support_traceability_report<T>(
    contribution: WorthQueryMaterializationReadySupportContribution<T>,
) -> WorthQueryDomainCapabilityTransitionOutcome<WorthQueryIntentAdmissionSupportTraceabilityReport>
where
    T: WorthQueryAdmittedPlanContributionTargetBinding,
{
    match support_traceability_row(&contribution) {
        TransitionOutcome::Success(row) => TransitionOutcome::Success(
            WorthQueryIntentAdmissionSupportTraceabilityReport::from_rows(vec![row]),
        ),
        TransitionOutcome::Denied(denial) => TransitionOutcome::Denied(denial),
        TransitionOutcome::Stale(stale) => TransitionOutcome::Stale(stale),
        TransitionOutcome::RebindRequired(rebind) => TransitionOutcome::RebindRequired(rebind),
        TransitionOutcome::Failed(failure) => TransitionOutcome::Failed(failure),
        TransitionOutcome::Deferred(never) => match never {},
    }
}

#[cfg(test)]
pub fn materialize_intent_admission_support_traceability_row<T>(
    contribution: WorthQueryMaterializationReadySupportContribution<T>,
) -> WorthQueryDomainCapabilityTransitionOutcome<WorthQueryIntentAdmissionSupportTraceabilityRow>
where
    T: WorthQueryAdmittedPlanContributionTargetBinding,
{
    support_traceability_row(&contribution)
}

fn support_traceability_row<T>(
    contribution: &WorthQueryMaterializationReadySupportContribution<T>,
) -> Result<
    WorthQueryDomainCapabilityTransitionOutcome<WorthQueryIntentAdmissionSupportTraceabilityRow>,
    crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial,
>
where
    T: WorthQueryAdmittedPlanContributionTargetBinding,
{
    let domain_contribution = contribution.payload();
    let payload = domain_contribution.payload();
    let Some((family, entrypoint, request_digest, eligibility_digest, decision_digest)) =
        domain_contribution
            .target()
            .semantics()
            .admitted_intent_plan()
    else {
        unreachable!("admitted-plan target should preserve admitted-plan semantics");
    };
    Ok(TransitionOutcome::Success(
        WorthQueryIntentAdmissionSupportTraceabilityRow::new_domain_scoped(
            super::support::support_lane(payload.posture()),
            family.as_str(),
            entrypoint.as_str(),
            super::support::support_detail_label(payload.semantic_code(), payload.detail()),
            Some(
                domain_contribution
                    .target()
                    .binding_identity()?
                    .as_str()
                    .to_string(),
            ),
            Some(request_digest.to_string()),
            Some(eligibility_digest.to_string()),
            Some(decision_digest.to_string()),
        ),
    ))
}
