#[cfg(test)]
use worth_foundational::FoundationalDiagnosticDeliveryClass;
use worth_foundational::{
    boundary_evidence, derive_foundational_profile_identity, BoundaryHandle,
    CanonicalizationRuleVersion, FoundationalBoundaryEvidenceCanonicalDigestBasis,
    FoundationalBoundaryEvidenceFreshnessPosture, FoundationalBoundaryEvidenceProfileBasis,
    FoundationalBoundaryEvidenceProvenanceArtifact, FoundationalBoundaryEvidenceSourceBasis,
    FoundationalBoundaryEvidenceStrategyBasis,
    FoundationalBoundaryEvidenceSupportContextAttachment, FoundationalTransitionStrategyFamily,
    FoundationalTransitionStrategyId, FoundationalTransitionStrategyIdentity,
    FoundationalTransitionStrategyOwnershipClass, FoundationalTransitionStrategySemanticName,
    FoundationalTransitionStrategyVersion,
};
use worth_proof::TransitionOutcome;

use super::super::materialization::{
    WorthQueryDomainCapabilityDescriptiveArtifactKind,
    WorthQueryDomainCapabilityDescriptiveMaterializationDenial,
    WorthQueryDomainCapabilityProfileProgression,
};
use super::super::targets::WorthQueryDomainCapabilityTargetBinding;
use super::super::{
    WorthQueryDomainCapabilityPayload, WorthQueryDomainCapabilitySemanticPosture,
    WorthQueryMaterializationReadyDomainCapabilityContribution,
};
use super::rows::WorthQueryDomainCapabilityDiagnosticRows;

#[derive(Clone, Copy)]
pub(crate) enum WorthQueryDomainCapabilityProvenanceFreshnessPolicy {
    #[cfg(test)]
    SupportSurface(FoundationalDiagnosticDeliveryClass),
    SummaryReduction,
    #[cfg(test)]
    TraceRetention,
}

pub(crate) fn build_provenance<P, T>(
    contribution: &WorthQueryMaterializationReadyDomainCapabilityContribution<P, T>,
    profile_progression: &WorthQueryDomainCapabilityProfileProgression,
    rows: &WorthQueryDomainCapabilityDiagnosticRows,
    artifact_kind: WorthQueryDomainCapabilityDescriptiveArtifactKind,
    freshness_policy: WorthQueryDomainCapabilityProvenanceFreshnessPolicy,
) -> Result<
    FoundationalBoundaryEvidenceProvenanceArtifact,
    WorthQueryDomainCapabilityDescriptiveMaterializationDenial,
>
where
    P: WorthQueryDomainCapabilityPayload,
    T: WorthQueryDomainCapabilityTargetBinding,
{
    let category = contribution.payload().category();
    let freshness_posture = freshness_for(freshness_policy, contribution.payload().target().kind());
    let profile_identity = match derive_foundational_profile_identity(
        CanonicalizationRuleVersion::new("worth.query.domain-capabilities.v1")
            .expect("valid canonicalization rule version"),
        profile_progression.admitted(),
    ) {
        TransitionOutcome::Success(identity) => identity,
        TransitionOutcome::Denied(denial) => {
            return Err(
                WorthQueryDomainCapabilityDescriptiveMaterializationDenial::ProfileIdentity {
                    category,
                    artifact_kind,
                    denial,
                },
            )
        }
        outcome => panic!("unexpected foundational profile identity outcome: {outcome:?}"),
    };

    let semantic_posture = contribution.payload().payload().semantic_posture();
    let step = match freshness_posture {
        FoundationalBoundaryEvidenceFreshnessPosture::ReconstructedFromReplay => {
            boundary_evidence()
                .provenance()
                .replay_derived(source_basis_for(contribution)?)
        }
        FoundationalBoundaryEvidenceFreshnessPosture::RestoredFromCheckpoint => boundary_evidence()
            .provenance()
            .restored_readmitted(source_basis_for(contribution)?),
        FoundationalBoundaryEvidenceFreshnessPosture::FreshRetained
        | FoundationalBoundaryEvidenceFreshnessPosture::StaleRetained
        | FoundationalBoundaryEvidenceFreshnessPosture::ReducedRetained => boundary_evidence()
            .provenance()
            .current(source_basis_for(contribution)?),
    }
    .profile_basis(FoundationalBoundaryEvidenceProfileBasis::profile(
        profile_identity.clone(),
    ))
    .canonical_digest_basis(FoundationalBoundaryEvidenceCanonicalDigestBasis::digest(
        profile_identity.digest().clone(),
    ));
    let step = if semantic_posture.is_policy_or_inferred() {
        step.strategy_basis(strategy_basis_for(contribution, semantic_posture))
    } else {
        step
    }
    .attach_support_context(
        FoundationalBoundaryEvidenceSupportContextAttachment::diagnostic_scope(rows.scope.clone()),
    )
    .attach_support_context(
        FoundationalBoundaryEvidenceSupportContextAttachment::diagnostic_code(
            rows.primary_code.clone(),
        ),
    );

    match step.with_freshness(freshness_posture) {
        TransitionOutcome::Success(provenance) => Ok(provenance),
        TransitionOutcome::Denied(denial) => panic!("unexpected provenance denial: {denial:?}"),
        outcome => panic!("unexpected provenance outcome: {outcome:?}"),
    }
}

fn source_basis_for<P, T>(
    contribution: &WorthQueryMaterializationReadyDomainCapabilityContribution<P, T>,
) -> Result<
    FoundationalBoundaryEvidenceSourceBasis,
    crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial,
>
where
    P: WorthQueryDomainCapabilityPayload,
    T: WorthQueryDomainCapabilityTargetBinding,
{
    let locator = worth_foundational::BoundaryArtifactLocator::new(
        super::identity::boundary_artifact_id(&contribution.payload().target().binding_identity()?),
        worth_foundational::BoundaryArtifactField::Basis,
    );
    Ok(FoundationalBoundaryEvidenceSourceBasis::boundary_artifact(
        locator,
    ))
}

fn freshness_for(
    freshness_policy: WorthQueryDomainCapabilityProvenanceFreshnessPolicy,
    target_kind: crate::domain_capabilities::WorthQueryDomainCapabilityTargetKind,
) -> FoundationalBoundaryEvidenceFreshnessPosture {
    if target_kind == crate::domain_capabilities::WorthQueryDomainCapabilityTargetKind::LowerRuntimeBoundaryEnvelope
    {
        return FoundationalBoundaryEvidenceFreshnessPosture::ReconstructedFromReplay;
    }

    match freshness_policy {
        #[cfg(test)]
        WorthQueryDomainCapabilityProvenanceFreshnessPolicy::SupportSurface(delivery_class) => {
            match delivery_class {
                FoundationalDiagnosticDeliveryClass::MustBeHot => {
                    FoundationalBoundaryEvidenceFreshnessPosture::FreshRetained
                }
                FoundationalDiagnosticDeliveryClass::CanDefer
                | FoundationalDiagnosticDeliveryClass::UnavailableByPolicy => {
                    FoundationalBoundaryEvidenceFreshnessPosture::StaleRetained
                }
                FoundationalDiagnosticDeliveryClass::ReconstructableFromReplay => {
                    FoundationalBoundaryEvidenceFreshnessPosture::ReconstructedFromReplay
                }
            }
        }
        WorthQueryDomainCapabilityProvenanceFreshnessPolicy::SummaryReduction => {
            FoundationalBoundaryEvidenceFreshnessPosture::ReducedRetained
        }
        #[cfg(test)]
        WorthQueryDomainCapabilityProvenanceFreshnessPolicy::TraceRetention => {
            FoundationalBoundaryEvidenceFreshnessPosture::FreshRetained
        }
    }
}

fn strategy_basis_for<P, T>(
    contribution: &WorthQueryMaterializationReadyDomainCapabilityContribution<P, T>,
    semantic_posture: WorthQueryDomainCapabilitySemanticPosture,
) -> FoundationalBoundaryEvidenceStrategyBasis
where
    P: WorthQueryDomainCapabilityPayload,
    T: WorthQueryDomainCapabilityTargetBinding,
{
    let handle = BoundaryHandle::new(
        super::identity::boundary_artifact_id(contribution.payload().request_identity()).get(),
    );
    let family =
        FoundationalTransitionStrategyFamily::new(contribution.payload().category().as_str())
            .expect("category names are valid strategy families");
    let semantic_name = FoundationalTransitionStrategySemanticName::new(semantic_posture.as_str())
        .expect("semantic posture names are valid strategy semantic names");
    let version = FoundationalTransitionStrategyVersion::new("v1")
        .expect("static strategy version should always be valid");

    FoundationalBoundaryEvidenceStrategyBasis::strategy(
        FoundationalTransitionStrategyIdentity::new(
            FoundationalTransitionStrategyId::new(handle),
            family,
            semantic_name,
            version,
            FoundationalTransitionStrategyOwnershipClass::CompatibilityLowered,
        ),
    )
}
