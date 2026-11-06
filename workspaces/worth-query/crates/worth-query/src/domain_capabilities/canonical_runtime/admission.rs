use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use worth_proof::TransitionOutcome;

use crate::domain_capabilities::denials::{
    WorthQueryDomainCapabilityProgressionDenial, WorthQueryDomainCapabilityProgressionDenialKind,
};
use crate::domain_capabilities::payloads::WorthQueryAdmissionContributionPosture;
#[cfg(test)]
use crate::domain_capabilities::targets::WorthQueryAdmittedPlanBoundContributionTarget;
use crate::domain_capabilities::targets::WorthQueryDomainCapabilityTargetBinding;
use crate::domain_capabilities::{
    WorthQueryCanonicalAdmissionArtifact, WorthQueryDomainCapabilityTransitionOutcome,
    WorthQueryMaterializationReadyAdmissionContribution,
};
use crate::intent_admission::{
    WorthQueryIntentAdmissionDecision, WorthQueryIntentAdvisoryDecision,
    WorthQueryIntentViolationDecision,
};
#[cfg(test)]
use crate::intent_admission::{
    WorthQueryIntentAdmissionSupportTraceabilityReport,
    WorthQueryIntentAdmissionSupportTraceabilityRow,
};

pub fn materialize_canonical_admission_artifact<T>(
    contribution: WorthQueryMaterializationReadyAdmissionContribution<T>,
) -> Result<WorthQueryCanonicalAdmissionArtifact<T>, IdentityDerivationDenial>
where
    T: WorthQueryDomainCapabilityTargetBinding,
{
    Ok(super::artifacts::materialize_domain_capability_canonical_runtime_artifact(contribution)?)
}

pub(crate) fn materialize_runtime_admission_decision<T>(
    contribution: WorthQueryMaterializationReadyAdmissionContribution<T>,
) -> Result<
    WorthQueryDomainCapabilityTransitionOutcome<WorthQueryIntentAdmissionDecision>,
    IdentityDerivationDenial,
>
where
    T: crate::domain_capabilities::WorthQueryAdmittedPlanContributionTargetBinding,
{
    let domain_contribution = contribution.payload();
    let payload = domain_contribution.payload();
    let target = domain_contribution.target();
    let Some((family, entrypoint, request_digest, eligibility_digest, _decision_digest)) =
        target.semantics().admitted_intent_plan()
    else {
        unreachable!("admitted-plan bound target should preserve admitted-plan semantics")
    };

    Ok(match payload.posture() {
        WorthQueryAdmissionContributionPosture::Advisory => TransitionOutcome::Success(
            WorthQueryIntentAdmissionDecision::Advisory(WorthQueryIntentAdvisoryDecision::new(
                family,
                entrypoint,
                payload.decision_stage(),
                payload.detail(),
                request_digest,
                eligibility_digest,
            )),
        ),
        WorthQueryAdmissionContributionPosture::Violation => TransitionOutcome::Success(
            WorthQueryIntentAdmissionDecision::Violation(WorthQueryIntentViolationDecision::new(
                family,
                entrypoint,
                payload.decision_stage(),
                payload.detail(),
                request_digest,
                eligibility_digest,
            )),
        ),
        WorthQueryAdmissionContributionPosture::SupportOnly => {
            TransitionOutcome::Denied(unsupported_decision_posture_denial(
                payload.posture(),
                domain_contribution.request_identity().clone(),
            )?)
        }
    })
}

#[cfg(test)]
pub fn materialize_runtime_admission_support_traceability_report(
    contribution: WorthQueryMaterializationReadyAdmissionContribution<
        WorthQueryAdmittedPlanBoundContributionTarget,
    >,
) -> Result<
    WorthQueryDomainCapabilityTransitionOutcome<WorthQueryIntentAdmissionSupportTraceabilityReport>,
    IdentityDerivationDenial,
> {
    Ok(
        match materialize_runtime_admission_support_traceability_row(contribution)? {
            TransitionOutcome::Success(row) => TransitionOutcome::Success(
                WorthQueryIntentAdmissionSupportTraceabilityReport::from_rows(vec![row]),
            ),
            TransitionOutcome::Denied(denial) => TransitionOutcome::Denied(denial),
            TransitionOutcome::Stale(stale) => TransitionOutcome::Stale(stale),
            TransitionOutcome::RebindRequired(rebind) => TransitionOutcome::RebindRequired(rebind),
            TransitionOutcome::Failed(failure) => TransitionOutcome::Failed(failure),
            TransitionOutcome::Deferred(never) => match never {},
        },
    )
}

#[cfg(test)]
pub fn materialize_runtime_admission_support_traceability_row(
    contribution: WorthQueryMaterializationReadyAdmissionContribution<
        WorthQueryAdmittedPlanBoundContributionTarget,
    >,
) -> Result<
    WorthQueryDomainCapabilityTransitionOutcome<WorthQueryIntentAdmissionSupportTraceabilityRow>,
    IdentityDerivationDenial,
> {
    let domain_contribution = contribution.payload();
    let payload = domain_contribution.payload();
    let Some((family, entrypoint, request_digest, eligibility_digest, decision_digest)) =
        domain_contribution
            .target()
            .semantics()
            .admitted_intent_plan()
    else {
        unreachable!("admitted-plan bound target should preserve admitted-plan semantics")
    };

    if payload.posture() != WorthQueryAdmissionContributionPosture::SupportOnly {
        return Ok(TransitionOutcome::Denied(
            unsupported_support_posture_denial(
                payload.posture(),
                domain_contribution.request_identity().clone(),
            )?,
        ));
    }

    Ok(TransitionOutcome::Success(
        WorthQueryIntentAdmissionSupportTraceabilityRow::new_domain_scoped(
            "admission_local_support",
            family.as_str(),
            entrypoint.as_str(),
            support_detail_label(payload.semantic_code(), payload.detail()),
            Some(
                domain_contribution
                    .target()
                    .binding_identity()
                    .as_str()
                    .to_string(),
            ),
            Some(request_digest.to_string()),
            Some(eligibility_digest.to_string()),
            Some(decision_digest.to_string()),
        ),
    ))
}

#[cfg(test)]
fn support_detail_label(semantic_code: &str, detail: &str) -> String {
    let mut label = String::with_capacity(
        semantic_code
            .len()
            .saturating_add(1)
            .saturating_add(detail.len()),
    );
    label.push_str(semantic_code);
    label.push(':');
    label.push_str(detail);
    label
}

fn unsupported_decision_posture_denial(
    posture: WorthQueryAdmissionContributionPosture,
    request_identity: crate::evidence_identity::WorthQueryEvidenceIdentity,
) -> Result<WorthQueryDomainCapabilityProgressionDenial, IdentityDerivationDenial> {
    Ok(WorthQueryDomainCapabilityProgressionDenial::new(
        WorthQueryDomainCapabilityProgressionDenialKind::UnsupportedCanonicalMaterializationPosture,
        "admission",
        crate::domain_capabilities::WorthQueryDomainCapabilityTargetKind::AdmittedIntentPlan,
        request_identity,
        format!(
            "admission runtime decision materialization only supports advisory and violation postures; got `{}`",
            posture.as_str()
        ),
    )?)
}

#[cfg(test)]
fn unsupported_support_posture_denial(
    posture: WorthQueryAdmissionContributionPosture,
    request_identity: crate::evidence_identity::WorthQueryEvidenceIdentity,
) -> Result<WorthQueryDomainCapabilityProgressionDenial, IdentityDerivationDenial> {
    Ok(WorthQueryDomainCapabilityProgressionDenial::new(
        WorthQueryDomainCapabilityProgressionDenialKind::UnsupportedCanonicalMaterializationPosture,
        "admission",
        crate::domain_capabilities::WorthQueryDomainCapabilityTargetKind::AdmittedIntentPlan,
        request_identity,
        format!(
            "admission support traceability materialization only supports support-only posture; got `{}`",
            posture.as_str()
        ),
    )?)
}
