use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use worth_proof::TransitionOutcome;

use crate::domain_capabilities::denials::{
    WorthQueryDomainCapabilityProgressionDenial, WorthQueryDomainCapabilityProgressionDenialKind,
};
use crate::domain_capabilities::identity::domain_capability_scope_encoder;
use crate::domain_capabilities::payloads::{
    WorthQueryContinuityContributionPayload, WorthQueryContinuityContributionPosture,
    WorthQueryContinuityRuntimeSemantics,
};
use crate::domain_capabilities::targets::{
    WorthQueryDomainCapabilityTargetBinding, WorthQueryDomainCapabilityTargetKind,
};
use crate::domain_capabilities::{
    WorthQueryCanonicalContinuityArtifact, WorthQueryDomainCapabilityTransitionOutcome,
    WorthQueryMaterializationReadyContinuityContribution,
};
use crate::runtime::{WorthQueryContinuityMutationEvidence, WorthQueryContinuityMutationIntent};
use crate::{WorthQueryEvidenceIdentity, WorthQueryEvidenceTag};

pub fn materialize_canonical_continuity_artifact<T>(
    contribution: WorthQueryMaterializationReadyContinuityContribution<T>,
) -> Result<WorthQueryCanonicalContinuityArtifact<T>, IdentityDerivationDenial>
where
    T: WorthQueryDomainCapabilityTargetBinding,
{
    Ok(super::artifacts::materialize_domain_capability_canonical_runtime_artifact(contribution)?)
}

pub(crate) fn materialize_runtime_continuity_evidence<T>(
    contribution: WorthQueryMaterializationReadyContinuityContribution<T>,
) -> Result<
    WorthQueryDomainCapabilityTransitionOutcome<WorthQueryContinuityMutationEvidence>,
    IdentityDerivationDenial,
>
where
    T: crate::domain_capabilities::WorthQueryAdmittedPlanContributionTargetBinding,
{
    let domain_contribution = contribution.payload();
    let payload = domain_contribution.payload();
    let binding_identity = domain_capability_continuity_binding_identity(
        domain_contribution.target().kind(),
        &domain_contribution.target().binding_identity()?,
    )?;
    let Some(runtime_semantics) = payload.runtime_semantics() else {
        return Ok(TransitionOutcome::Denied(missing_runtime_semantics_denial(
            payload,
            domain_contribution.request_identity().clone(),
        )));
    };
    if !runtime_semantics_match_posture(runtime_semantics, payload.posture()) {
        return Ok(TransitionOutcome::Denied(
            inconsistent_runtime_semantics_denial(
                payload,
                runtime_semantics,
                domain_contribution.request_identity().clone(),
            ),
        ));
    }
    let Some(intent) = continuity_intent_from_runtime_semantics(runtime_semantics, payload) else {
        return Ok(TransitionOutcome::Denied(unsupported_posture_denial(
            payload,
            domain_contribution.request_identity().clone(),
        )));
    };

    Ok(TransitionOutcome::Success(
        WorthQueryContinuityMutationEvidence::from_intent(
            &intent,
            Some(&binding_identity),
            None,
            None,
        )?,
    ))
}

fn domain_capability_continuity_binding_identity(
    target_kind: WorthQueryDomainCapabilityTargetKind,
    binding_identity: &WorthQueryEvidenceIdentity,
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    Ok(
        domain_capability_scope_encoder("domain_capability_continuity_binding_v1")
            .field_shape(
                WorthQueryEvidenceTag::new("target_kind"),
                target_kind.as_str(),
            )
            .field_evidence_identity(WorthQueryEvidenceTag::new("binding"), binding_identity)
            .seal()?,
    )
}

fn runtime_semantics_match_posture(
    runtime_semantics: &WorthQueryContinuityRuntimeSemantics,
    posture: WorthQueryContinuityContributionPosture,
) -> bool {
    match posture {
        WorthQueryContinuityContributionPosture::Preserved => {
            runtime_semantics.family()
                == crate::runtime::WorthQueryContinuityMutationFamily::RebindExistingTarget
                && runtime_semantics.outcome_class()
                    == crate::runtime::WorthQueryContinuityMutationOutcomeClass::ContinuesAsSingleSuccessor
        }
        WorthQueryContinuityContributionPosture::Split => {
            runtime_semantics.family()
                == crate::runtime::WorthQueryContinuityMutationFamily::SplitExistingTarget
                && runtime_semantics.outcome_class()
                    == crate::runtime::WorthQueryContinuityMutationOutcomeClass::ContinuesAsSplitSuccessors
        }
        WorthQueryContinuityContributionPosture::Replaced => {
            runtime_semantics.family()
                == crate::runtime::WorthQueryContinuityMutationFamily::RebindExistingTarget
                && runtime_semantics.outcome_class()
                    == crate::runtime::WorthQueryContinuityMutationOutcomeClass::ContinuesViaTruthLoweredCanonicalMergeSuccessor
        }
        WorthQueryContinuityContributionPosture::CorrespondenceOnly => false,
    }
}

fn continuity_intent_from_runtime_semantics(
    runtime_semantics: &WorthQueryContinuityRuntimeSemantics,
    payload: &WorthQueryContinuityContributionPayload,
) -> Option<WorthQueryContinuityMutationIntent> {
    let prior_authority =
        crate::runtime::WorthQueryMutationAuthorityIdentity::continuity_prior_authority(
            crate::runtime::WorthQueryContinuityPriorAuthorityLabel::new(
                runtime_semantics.prior_authoritative_source_label_for_reporting(),
            )
            .ok()?,
        )
        .ok()?;
    match payload.posture() {
        WorthQueryContinuityContributionPosture::Preserved => {
            let successor_authority = crate::runtime::WorthQueryMutationAuthorityIdentity::continuity_successor_authority(
                crate::runtime::WorthQueryContinuitySuccessorAuthorityLabel::new(
                    runtime_semantics
                        .successor_authoritative_source_labels_for_reporting()
                        .first()?,
                )
                .ok()?,
            )
            .ok()?;
            WorthQueryContinuityMutationIntent::rebind_existing_target(
                prior_authority,
                successor_authority,
            )
            .ok()
        }
        WorthQueryContinuityContributionPosture::Split => {
            let successor_authorities = runtime_semantics
                .successor_authoritative_source_labels_for_reporting()
                .iter()
                .map(|label| {
                    crate::runtime::WorthQueryMutationAuthorityIdentity::continuity_successor_authority(
                        crate::runtime::WorthQueryContinuitySuccessorAuthorityLabel::new(label)?,
                    )
                })
                .collect::<Result<Vec<_>, _>>()
                .ok()?;
            WorthQueryContinuityMutationIntent::split_existing_target(
                prior_authority,
                successor_authorities,
            )
            .ok()
        }
        WorthQueryContinuityContributionPosture::Replaced => {
            let successor_authority = crate::runtime::WorthQueryMutationAuthorityIdentity::continuity_successor_authority(
                crate::runtime::WorthQueryContinuitySuccessorAuthorityLabel::new(
                    runtime_semantics
                        .successor_authoritative_source_labels_for_reporting()
                        .first()?,
                )
                .ok()?,
            )
            .ok()?;
            WorthQueryContinuityMutationIntent::rebind_merge_successor(
                prior_authority,
                successor_authority,
            )
            .ok()
        }
        WorthQueryContinuityContributionPosture::CorrespondenceOnly => None,
    }
}

fn missing_runtime_semantics_denial(
    payload: &WorthQueryContinuityContributionPayload,
    request_identity: WorthQueryEvidenceIdentity,
) -> WorthQueryDomainCapabilityProgressionDenial {
    WorthQueryDomainCapabilityProgressionDenial::new(
        WorthQueryDomainCapabilityProgressionDenialKind::MissingCanonicalMaterializationSemantics,
        "continuity-lineage",
        crate::domain_capabilities::WorthQueryDomainCapabilityTargetKind::AdmittedIntentPlan,
        request_identity,
        format!(
            "continuity runtime materialization requires continuity intent semantics for `{}`",
            payload.semantic_code()
        ),
    )
}

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
            "continuity runtime materialization does not support `{}` posture",
            payload.posture().as_str()
        ),
    )
}

fn inconsistent_runtime_semantics_denial(
    payload: &WorthQueryContinuityContributionPayload,
    runtime_semantics: &WorthQueryContinuityRuntimeSemantics,
    request_identity: WorthQueryEvidenceIdentity,
) -> WorthQueryDomainCapabilityProgressionDenial {
    WorthQueryDomainCapabilityProgressionDenial::new(
        WorthQueryDomainCapabilityProgressionDenialKind::InconsistentCanonicalMaterializationSemantics,
        "continuity-lineage",
        crate::domain_capabilities::WorthQueryDomainCapabilityTargetKind::AdmittedIntentPlan,
        request_identity,
        format!(
            "continuity runtime semantics `{}:{}` do not match `{}` posture",
            runtime_semantics.family().as_str(),
            runtime_semantics.outcome_class().as_str(),
            payload.posture().as_str()
        ),
    )
}
