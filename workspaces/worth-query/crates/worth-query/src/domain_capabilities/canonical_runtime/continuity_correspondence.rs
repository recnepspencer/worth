use worth_proof::TransitionOutcome;

use crate::correspondence::{
    resolve_correspondence_evidence, CorrespondenceEvaluationFailureClass,
    CorrespondenceEvidenceResolved,
};
use crate::domain_capabilities::denials::{
    WorthQueryDomainCapabilityProgressionDenial, WorthQueryDomainCapabilityProgressionDenialKind,
};
use crate::domain_capabilities::payloads::{
    WorthQueryContinuityContributionPayload, WorthQueryContinuityContributionPosture,
};
use crate::domain_capabilities::targets::WorthQueryAdmittedPlanBoundContributionTarget;
use crate::domain_capabilities::{
    WorthQueryDomainCapabilityTransitionOutcome,
    WorthQueryMaterializationReadyContinuityContribution,
};
use crate::evidence_identity::WorthQueryEvidenceIdentity;

#[cfg(test)]
pub fn materialize_correspondence_evidence_resolved(
    contribution: WorthQueryMaterializationReadyContinuityContribution<
        WorthQueryAdmittedPlanBoundContributionTarget,
    >,
) -> WorthQueryDomainCapabilityTransitionOutcome<CorrespondenceEvidenceResolved> {
    let domain_contribution = contribution.payload();
    let payload = domain_contribution.payload();

    if payload.posture() != WorthQueryContinuityContributionPosture::CorrespondenceOnly {
        return TransitionOutcome::Denied(unsupported_posture_denial(
            payload,
            domain_contribution.request_identity().clone(),
        ));
    }

    let Some(correspondence_semantics) = payload.correspondence_semantics() else {
        return TransitionOutcome::Denied(missing_correspondence_semantics_denial(
            payload,
            domain_contribution.request_identity().clone(),
        ));
    };

    match resolve_correspondence_evidence(correspondence_semantics.to_request()) {
        Ok(resolved) => TransitionOutcome::Success(resolved),
        Err(error) => TransitionOutcome::Denied(correspondence_error_denial(
            payload,
            domain_contribution.request_identity().clone(),
            error.failure_class(),
        )),
    }
}

#[cfg(test)]
fn missing_correspondence_semantics_denial(
    payload: &WorthQueryContinuityContributionPayload,
    request_identity: WorthQueryEvidenceIdentity,
) -> WorthQueryDomainCapabilityProgressionDenial {
    WorthQueryDomainCapabilityProgressionDenial::new(
        WorthQueryDomainCapabilityProgressionDenialKind::MissingCanonicalMaterializationSemantics,
        "continuity-lineage",
        crate::domain_capabilities::WorthQueryDomainCapabilityTargetKind::AdmittedIntentPlan,
        request_identity,
        format!(
            "continuity correspondence materialization requires correspondence semantics for `{}`",
            payload.semantic_code()
        ),
    )
}

#[cfg(test)]
fn unsupported_posture_denial(
    payload: &WorthQueryContinuityContributionPayload,
    request_identity: WorthQueryEvidenceIdentity,
) -> WorthQueryDomainCapabilityProgressionDenial {
    WorthQueryDomainCapabilityProgressionDenial::new(
        WorthQueryDomainCapabilityProgressionDenialKind::UnsupportedCanonicalMaterializationPosture,
        "continuity-lineage",
        crate::domain_capabilities::WorthQueryDomainCapabilityTargetKind::AdmittedIntentPlan,
        request_identity,
        format!(
            "continuity correspondence materialization only supports `correspondence-only`; got `{}`",
            payload.posture().as_str()
        ),
    )
}

#[cfg(test)]
fn correspondence_error_denial(
    payload: &WorthQueryContinuityContributionPayload,
    request_identity: WorthQueryEvidenceIdentity,
    failure_class: CorrespondenceEvaluationFailureClass,
) -> WorthQueryDomainCapabilityProgressionDenial {
    let denial_kind = match failure_class {
        CorrespondenceEvaluationFailureClass::InvalidRequest => {
            WorthQueryDomainCapabilityProgressionDenialKind::InconsistentCanonicalMaterializationSemantics
        }
        CorrespondenceEvaluationFailureClass::UnsupportedTopology
        | CorrespondenceEvaluationFailureClass::UnsupportedStructuralFamily
        | CorrespondenceEvaluationFailureClass::UnsupportedMixedEvidence
        | CorrespondenceEvaluationFailureClass::BroadStructuralScanRequired
        | CorrespondenceEvaluationFailureClass::StructuralBreadthExceeded => {
            WorthQueryDomainCapabilityProgressionDenialKind::UnsupportedCanonicalMaterializationPosture
        }
    };

    WorthQueryDomainCapabilityProgressionDenial::new(
        denial_kind,
        "continuity-lineage",
        crate::domain_capabilities::WorthQueryDomainCapabilityTargetKind::AdmittedIntentPlan,
        request_identity,
        format!(
            "continuity correspondence materialization denied for `{}` with `{}`",
            payload.semantic_code(),
            failure_class_label(failure_class),
        ),
    )
}

#[cfg(test)]
fn failure_class_label(failure_class: CorrespondenceEvaluationFailureClass) -> &'static str {
    match failure_class {
        CorrespondenceEvaluationFailureClass::InvalidRequest => "invalid-request",
        CorrespondenceEvaluationFailureClass::UnsupportedTopology => "unsupported-topology",
        CorrespondenceEvaluationFailureClass::UnsupportedStructuralFamily => {
            "unsupported-structural-family"
        }
        CorrespondenceEvaluationFailureClass::UnsupportedMixedEvidence => {
            "unsupported-mixed-evidence"
        }
        CorrespondenceEvaluationFailureClass::BroadStructuralScanRequired => {
            "broad-structural-scan-required"
        }
        CorrespondenceEvaluationFailureClass::StructuralBreadthExceeded => {
            "structural-breadth-exceeded"
        }
    }
}
