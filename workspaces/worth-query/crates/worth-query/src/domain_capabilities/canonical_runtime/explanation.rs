use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use worth_proof::TransitionOutcome;

use crate::domain_capabilities::denials::{
    WorthQueryDomainCapabilityProgressionDenial, WorthQueryDomainCapabilityProgressionDenialKind,
};
use crate::domain_capabilities::targets::WorthQueryDomainCapabilityTargetBinding;
use crate::domain_capabilities::{
    WorthQueryCanonicalExplanationArtifact, WorthQueryDomainCapabilityTransitionOutcome,
    WorthQueryMaterializationReadyExplanationContribution,
};
use crate::evidence_identity::WorthQueryEvidenceIdentity;
use crate::runtime::{
    materialize_admitted_causal_inspection, materialize_advisory_causal_inspection,
    materialize_denied_causal_inspection, request_causal_inspection, CausalInspectionPlan,
    CausalInspectionProofFlow, QueryCausalInspectionArtifact,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryExplanationInspectionReview {
    semantic_code: String,
    request_identity: WorthQueryEvidenceIdentity,
    target_kind: crate::domain_capabilities::WorthQueryDomainCapabilityTargetKind,
    bridge_envelope: Option<worth_runtime_bridge::facade::BridgeCausalExplanationEnvelope>,
    plan: CausalInspectionPlan,
}

impl WorthQueryExplanationInspectionReview {
    pub fn semantic_code(&self) -> &str {
        &self.semantic_code
    }

    pub fn request_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.request_identity
    }

    pub fn request_for_reporting(&self) -> &str {
        self.request_identity.as_str()
    }

    pub fn target_kind(&self) -> crate::domain_capabilities::WorthQueryDomainCapabilityTargetKind {
        self.target_kind
    }

    pub fn bridge_envelope(
        &self,
    ) -> Option<&worth_runtime_bridge::facade::BridgeCausalExplanationEnvelope> {
        self.bridge_envelope.as_ref()
    }

    pub fn plan(&self) -> &CausalInspectionPlan {
        &self.plan
    }
}

pub fn materialize_canonical_explanation_artifact<T>(
    contribution: WorthQueryMaterializationReadyExplanationContribution<T>,
) -> WorthQueryCanonicalExplanationArtifact<T>
where
    T: WorthQueryDomainCapabilityTargetBinding,
{
    super::artifacts::materialize_domain_capability_canonical_runtime_artifact(contribution)
}

pub(crate) fn materialize_query_causal_inspection_artifact<T>(
    contribution: WorthQueryMaterializationReadyExplanationContribution<T>,
) -> Result<
    WorthQueryDomainCapabilityTransitionOutcome<QueryCausalInspectionArtifact>,
    IdentityDerivationDenial,
>
where
    T: crate::domain_capabilities::WorthQueryLowerRuntimeContributionTargetBinding,
{
    let review = match materialize_query_causal_inspection_review(contribution)? {
        TransitionOutcome::Success(review) => review,
        TransitionOutcome::Denied(denial) => return Ok(TransitionOutcome::Denied(denial)),
        TransitionOutcome::Stale(stale) => return Ok(TransitionOutcome::Stale(stale)),
        TransitionOutcome::RebindRequired(rebind) => {
            return Ok(TransitionOutcome::RebindRequired(rebind));
        }
        TransitionOutcome::Failed(failure) => return Ok(TransitionOutcome::Failed(failure)),
        TransitionOutcome::Deferred(never) => match never {},
    };
    let domain_plan = review.plan();
    let domain_contribution_semantic_code = review.semantic_code().to_string();
    let request_identity = review.request_identity().clone();

    let domain_target_kind = review.target_kind();
    let materialization_policy = domain_plan.materialization_policy();
    let redaction_policy = domain_plan.redaction_policy();
    Ok(match domain_plan.admission() {
        CausalInspectionProofFlow::Admitted(inspection) => {
            let Some(envelope) = review.bridge_envelope() else {
                return Ok(TransitionOutcome::Denied(missing_bridge_envelope_denial(
                    &domain_contribution_semantic_code,
                    request_identity.clone(),
                )?));
            };
            match materialize_admitted_causal_inspection(
                inspection,
                envelope,
                redaction_policy,
                materialization_policy,
            ) {
                Ok(artifact) => TransitionOutcome::Success(artifact),
                Err(error) => TransitionOutcome::Denied(
                    WorthQueryDomainCapabilityProgressionDenial::new(
                        WorthQueryDomainCapabilityProgressionDenialKind::UnsupportedCanonicalMaterializationPosture,
                        "explanation-inspection",
                        domain_target_kind,
                        request_identity.clone(),
                        format!(
                            "admitted causal inspection materialization denied for `{}` with `{}`",
                            domain_contribution_semantic_code,
                            error.kind().as_str(),
                        ),
                    )?,
                ),
            }
        }
        CausalInspectionProofFlow::Advisory(inspection) => {
            let Some(envelope) = review.bridge_envelope() else {
                return Ok(TransitionOutcome::Denied(missing_bridge_envelope_denial(
                    &domain_contribution_semantic_code,
                    request_identity.clone(),
                )?));
            };
            match materialize_advisory_causal_inspection(
                inspection,
                envelope,
                redaction_policy,
                materialization_policy,
            ) {
                Ok(artifact) => TransitionOutcome::Success(artifact),
                Err(error) => TransitionOutcome::Denied(
                    WorthQueryDomainCapabilityProgressionDenial::new(
                        WorthQueryDomainCapabilityProgressionDenialKind::UnsupportedCanonicalMaterializationPosture,
                        "explanation-inspection",
                        domain_target_kind,
                        request_identity.clone(),
                        format!(
                            "advisory causal inspection materialization denied for `{}` with `{}`",
                            domain_contribution_semantic_code,
                            error.kind().as_str(),
                        ),
                    )?,
                ),
            }
        }
        CausalInspectionProofFlow::Denied(inspection) => {
            TransitionOutcome::Success(materialize_denied_causal_inspection(
                inspection,
                None,
                redaction_policy,
                materialization_policy,
            ))
        }
    })
}

pub(crate) fn materialize_query_causal_inspection_review<T>(
    contribution: WorthQueryMaterializationReadyExplanationContribution<T>,
) -> Result<
    WorthQueryDomainCapabilityTransitionOutcome<WorthQueryExplanationInspectionReview>,
    IdentityDerivationDenial,
>
where
    T: crate::domain_capabilities::WorthQueryLowerRuntimeContributionTargetBinding,
{
    let domain_contribution = contribution.payload();
    let payload = domain_contribution.payload();
    let Some(runtime_semantics) = payload.runtime_semantics() else {
        return Ok(TransitionOutcome::Denied(missing_runtime_semantics_denial(
            payload.semantic_code(),
            domain_contribution.request_identity().clone(),
        )?));
    };

    let request = match request_causal_inspection(
        runtime_semantics.reference_set().clone(),
        runtime_semantics.target().clone(),
        runtime_semantics.explanation_family(),
        runtime_semantics.requested_richness(),
        runtime_semantics.requested_evidence_families(),
    ) {
        Ok(request) => request,
        Err(error) => {
            return Ok(TransitionOutcome::Denied(WorthQueryDomainCapabilityProgressionDenial::new(
                WorthQueryDomainCapabilityProgressionDenialKind::InconsistentCanonicalMaterializationSemantics,
                "explanation-inspection",
                domain_contribution.target().kind(),
                domain_contribution.request_identity().clone(),
                format!(
                    "causal inspection request denied for `{}` with `{}`: {}",
                    payload.semantic_code(),
                    error.kind().as_str(),
                    error.message(),
                ),
            )?));
        }
    };

    Ok(TransitionOutcome::Success(
        WorthQueryExplanationInspectionReview {
            semantic_code: payload.semantic_code().to_string(),
            request_identity: domain_contribution.request_identity().clone(),
            target_kind: domain_contribution.target().kind(),
            bridge_envelope: runtime_semantics.bridge_envelope().cloned(),
            plan: CausalInspectionPlan::from_resolved_request(
                runtime_semantics.reference_set().clone(),
                request,
                runtime_semantics.redaction_policy(),
                runtime_semantics.materialization_policy(),
            ),
        },
    ))
}

fn missing_runtime_semantics_denial(
    semantic_code: &str,
    request_identity: WorthQueryEvidenceIdentity,
) -> Result<WorthQueryDomainCapabilityProgressionDenial, IdentityDerivationDenial> {
    Ok(WorthQueryDomainCapabilityProgressionDenial::new(
        WorthQueryDomainCapabilityProgressionDenialKind::MissingCanonicalMaterializationSemantics,
        "explanation-inspection",
        crate::domain_capabilities::WorthQueryDomainCapabilityTargetKind::LowerRuntimeBoundaryEnvelope,
        request_identity,
        format!(
            "causal inspection materialization requires runtime explanation semantics for `{semantic_code}`"
        ),
    )?)
}

fn missing_bridge_envelope_denial(
    semantic_code: &str,
    request_identity: WorthQueryEvidenceIdentity,
) -> Result<WorthQueryDomainCapabilityProgressionDenial, IdentityDerivationDenial> {
    Ok(WorthQueryDomainCapabilityProgressionDenial::new(
        WorthQueryDomainCapabilityProgressionDenialKind::MissingCanonicalMaterializationSemantics,
        "explanation-inspection",
        crate::domain_capabilities::WorthQueryDomainCapabilityTargetKind::LowerRuntimeBoundaryEnvelope,
        request_identity,
        format!(
            "bridge-backed causal inspection materialization requires a bridge envelope for `{semantic_code}`"
        ),
    )?)
}
