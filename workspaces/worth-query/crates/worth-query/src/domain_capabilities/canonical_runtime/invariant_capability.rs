use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use worth_proof::TransitionOutcome;
use worth_relational::facade::runtime::InvariantCatalog;

use crate::domain_capabilities::identity::domain_capability_scope_encoder;
use crate::{WorthQueryEvidenceIdentity, WorthQueryEvidenceTag};

use crate::domain_capabilities::denials::{
    WorthQueryDomainCapabilityProgressionDenial, WorthQueryDomainCapabilityProgressionDenialKind,
};
use crate::domain_capabilities::payloads::{
    compose_invariant_registration_identity, WorthQueryGraphInvariantDenialRuntimeSemantics,
    WorthQueryInvariantCapabilityContributionPayload,
    WorthQueryInvariantCapabilityContributionPosture,
};
use crate::domain_capabilities::targets::WorthQueryDomainCapabilityTargetBinding;
use crate::domain_capabilities::{
    WorthQueryCanonicalInvariantCapabilityArtifact, WorthQueryDomainCapabilityTransitionOutcome,
    WorthQueryMaterializationReadyInvariantCapabilityContribution,
};
use crate::runtime::WorthQueryGraphCompositionCapabilitySupportRow;
use crate::runtime::{
    WorthQueryGraphCompositionDomainInvariantDenial,
    WorthQueryGraphCompositionDomainInvariantSummary,
};

pub fn materialize_canonical_invariant_capability_artifact<T>(
    contribution: WorthQueryMaterializationReadyInvariantCapabilityContribution<T>,
) -> WorthQueryCanonicalInvariantCapabilityArtifact<T>
where
    T: WorthQueryDomainCapabilityTargetBinding,
{
    super::artifacts::materialize_domain_capability_canonical_runtime_artifact(contribution)
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryInvariantCatalogRegistrationArtifact {
    lane: &'static str,
    semantic_code: String,
    detail: String,
    invariant_catalog: InvariantCatalog,
    intent_name: String,
    strategy_name: String,
    strategy_version: String,
    input_contract: String,
    source_lane: crate::runtime::WorthQueryIntentSourceLane,
    target_lane: crate::runtime::WorthQueryAuthorityLane,
    target_binding_identity: WorthQueryEvidenceIdentity,
    request_identity: WorthQueryEvidenceIdentity,
    materialization_identity: WorthQueryEvidenceIdentity,
}

impl WorthQueryInvariantCatalogRegistrationArtifact {
    pub fn lane(&self) -> &'static str {
        self.lane
    }

    pub fn semantic_code(&self) -> &str {
        &self.semantic_code
    }

    pub fn detail(&self) -> &str {
        &self.detail
    }

    pub fn invariant_catalog(&self) -> &InvariantCatalog {
        &self.invariant_catalog
    }

    pub fn intent_name(&self) -> &str {
        &self.intent_name
    }

    pub fn strategy_name(&self) -> &str {
        &self.strategy_name
    }

    pub fn strategy_version(&self) -> &str {
        &self.strategy_version
    }

    pub fn input_contract(&self) -> &str {
        &self.input_contract
    }

    pub fn source_lane(&self) -> crate::runtime::WorthQueryIntentSourceLane {
        self.source_lane
    }

    pub fn target_lane(&self) -> crate::runtime::WorthQueryAuthorityLane {
        self.target_lane
    }

    pub fn target_binding_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.target_binding_identity
    }

    pub fn target_binding_for_reporting(&self) -> &str {
        self.target_binding_identity.as_str()
    }

    pub fn request_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.request_identity
    }

    pub fn request_for_reporting(&self) -> &str {
        self.request_identity.as_str()
    }

    pub fn materialization_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.materialization_identity
    }

    pub fn materialization_digest(&self) -> &str {
        self.materialization_identity.as_str()
    }
}

pub(crate) fn materialize_query_invariant_catalog_registration_artifact<T>(
    contribution: WorthQueryMaterializationReadyInvariantCapabilityContribution<T>,
) -> Result<
    WorthQueryDomainCapabilityTransitionOutcome<WorthQueryInvariantCatalogRegistrationArtifact>,
    IdentityDerivationDenial,
>
where
    T: crate::domain_capabilities::WorthQueryDeclarationContributionTargetBinding,
{
    let domain_contribution = contribution.payload();
    let payload = domain_contribution.payload();
    let Some(invariant_registration) = payload.invariant_registration() else {
        return Ok(TransitionOutcome::Denied(missing_runtime_semantics_denial(
            payload,
            domain_contribution.request_identity().clone(),
            domain_contribution.target().kind(),
            "invariant registration",
        )?));
    };

    if payload.posture() != WorthQueryInvariantCapabilityContributionPosture::InvariantRegistration
    {
        return Ok(TransitionOutcome::Denied(unsupported_posture_denial(
            payload,
            domain_contribution.request_identity().clone(),
            domain_contribution.target().kind(),
            "invariant registration",
            "invariant-registration",
        )?));
    }

    let Some((name, strategy_name, strategy_version, input_contract, source_lane, target_lane)) =
        domain_contribution
            .target()
            .semantics()
            .intent_declaration()
    else {
        unreachable!("declaration-bound target should preserve declaration semantics");
    };
    let invariant_catalog = invariant_registration.canonical_invariant_catalog();

    Ok(TransitionOutcome::Success(
        WorthQueryInvariantCatalogRegistrationArtifact {
            lane: "query_invariant_catalog_registration",
            semantic_code: payload.semantic_code().to_string(),
            detail: payload.detail().to_string(),
            invariant_catalog: invariant_catalog.clone(),
            intent_name: name.to_string(),
            strategy_name: strategy_name.to_string(),
            strategy_version: strategy_version.to_string(),
            input_contract: input_contract.to_string(),
            source_lane,
            target_lane,
            target_binding_identity: domain_contribution.target().binding_identity(),
            request_identity: domain_contribution.request_identity().clone(),
            materialization_identity: domain_capability_scope_encoder(
                "worth_query_invariant_catalog_registration_artifact_v1",
            )
            .field_shape(
                WorthQueryEvidenceTag::new("lane"),
                "query_invariant_catalog_registration",
            )
            .field_shape(
                WorthQueryEvidenceTag::new("semantic_code"),
                payload.semantic_code(),
            )
            .field_shape(WorthQueryEvidenceTag::new("detail"), payload.detail())
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("catalog"),
                &compose_invariant_registration_identity(invariant_registration),
            )
            .field_shape(WorthQueryEvidenceTag::new("intent"), name)
            .field_shape(WorthQueryEvidenceTag::new("strategy"), strategy_name)
            .field_shape(
                WorthQueryEvidenceTag::new("strategy_version"),
                strategy_version,
            )
            .field_shape(WorthQueryEvidenceTag::new("input_contract"), input_contract)
            .field_shape(
                WorthQueryEvidenceTag::new("source_lane"),
                source_lane.as_str(),
            )
            .field_shape(
                WorthQueryEvidenceTag::new("target_lane"),
                target_lane.as_str(),
            )
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("binding"),
                &domain_contribution.target().binding_identity(),
            )
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("request"),
                domain_contribution.request_identity(),
            )
            .seal()?,
        },
    ))
}

pub fn materialize_graph_composition_capability_support_row<T>(
    contribution: WorthQueryMaterializationReadyInvariantCapabilityContribution<T>,
) -> Result<
    WorthQueryDomainCapabilityTransitionOutcome<WorthQueryGraphCompositionCapabilitySupportRow>,
    IdentityDerivationDenial,
>
where
    T: crate::domain_capabilities::WorthQueryLowerRuntimeContributionTargetBinding,
{
    let domain_contribution = contribution.payload();
    let payload = domain_contribution.payload();
    let Some(graph_capability) = payload.graph_capability() else {
        return Ok(TransitionOutcome::Denied(missing_runtime_semantics_denial(
            payload,
            domain_contribution.request_identity().clone(),
            domain_contribution.target().kind(),
            "graph capability",
        )?));
    };

    Ok(match payload.posture() {
        WorthQueryInvariantCapabilityContributionPosture::CapabilityGap
        | WorthQueryInvariantCapabilityContributionPosture::SupportSummary => {
            TransitionOutcome::Success(WorthQueryGraphCompositionCapabilitySupportRow::new(
                graph_capability.capability_family(),
                graph_capability.capability_class(),
            ))
        }
        WorthQueryInvariantCapabilityContributionPosture::InvariantDenial
        | WorthQueryInvariantCapabilityContributionPosture::InvariantRegistration => {
            TransitionOutcome::Denied(unsupported_posture_denial(
                payload,
                domain_contribution.request_identity().clone(),
                domain_contribution.target().kind(),
                "graph capability",
                "capability-gap and support-summary",
            )?)
        }
    })
}

pub(crate) fn materialize_graph_composition_domain_invariant_denial<T>(
    contribution: WorthQueryMaterializationReadyInvariantCapabilityContribution<T>,
) -> Result<
    WorthQueryDomainCapabilityTransitionOutcome<WorthQueryGraphCompositionDomainInvariantDenial>,
    IdentityDerivationDenial,
>
where
    T: crate::domain_capabilities::WorthQueryLowerRuntimeContributionTargetBinding,
{
    let domain_contribution = contribution.payload();
    let payload = domain_contribution.payload();
    let Some(graph_invariant_denial) = payload.graph_invariant_denial() else {
        return Ok(TransitionOutcome::Denied(missing_runtime_semantics_denial(
            payload,
            domain_contribution.request_identity().clone(),
            domain_contribution.target().kind(),
            "graph invariant denial",
        )?));
    };

    Ok(match payload.posture() {
        WorthQueryInvariantCapabilityContributionPosture::InvariantDenial => {
            TransitionOutcome::Success(
                WorthQueryGraphCompositionDomainInvariantDenial::from_contributed(
                    graph_invariant_denial.invariant_family(),
                    payload.detail(),
                    graph_invariant_summary(graph_invariant_denial)?,
                )?,
            )
        }
        WorthQueryInvariantCapabilityContributionPosture::CapabilityGap
        | WorthQueryInvariantCapabilityContributionPosture::SupportSummary
        | WorthQueryInvariantCapabilityContributionPosture::InvariantRegistration => {
            TransitionOutcome::Denied(unsupported_posture_denial(
                payload,
                domain_contribution.request_identity().clone(),
                domain_contribution.target().kind(),
                "graph invariant denial",
                "invariant-denial",
            )?)
        }
    })
}

fn graph_invariant_summary(
    semantics: &WorthQueryGraphInvariantDenialRuntimeSemantics,
) -> Result<WorthQueryGraphCompositionDomainInvariantSummary, IdentityDerivationDenial> {
    Ok(
        WorthQueryGraphCompositionDomainInvariantSummary::from_parts(
            semantics.declared_collections().to_vec(),
            semantics.declared_symbols().to_vec(),
            semantics.target_combination_families().to_vec(),
            semantics.lifecycle_families().to_vec(),
            semantics.program_identity().clone(),
            semantics.breadth_identity().clone(),
            semantics.counter_snapshot().to_string(),
        )?,
    )
}

fn missing_runtime_semantics_denial(
    payload: &WorthQueryInvariantCapabilityContributionPayload,
    request_identity: WorthQueryEvidenceIdentity,
    target_kind: crate::domain_capabilities::WorthQueryDomainCapabilityTargetKind,
    runtime_family: &str,
) -> Result<WorthQueryDomainCapabilityProgressionDenial, IdentityDerivationDenial> {
    Ok(WorthQueryDomainCapabilityProgressionDenial::new(
        WorthQueryDomainCapabilityProgressionDenialKind::MissingCanonicalMaterializationSemantics,
        "invariant-capability",
        target_kind,
        request_identity,
        format!(
            "{runtime_family} runtime materialization requires matching runtime semantics for `{}`",
            payload.semantic_code()
        ),
    )?)
}

fn unsupported_posture_denial(
    payload: &WorthQueryInvariantCapabilityContributionPayload,
    request_identity: WorthQueryEvidenceIdentity,
    target_kind: crate::domain_capabilities::WorthQueryDomainCapabilityTargetKind,
    runtime_family: &str,
    supported_postures: &str,
) -> Result<WorthQueryDomainCapabilityProgressionDenial, IdentityDerivationDenial> {
    Ok(WorthQueryDomainCapabilityProgressionDenial::new(
        WorthQueryDomainCapabilityProgressionDenialKind::UnsupportedCanonicalMaterializationPosture,
        "invariant-capability",
        target_kind,
        request_identity,
        format!(
            "{runtime_family} runtime materialization only supports {supported_postures} postures; got `{}`",
            payload.posture().as_str()
        ),
    )?)
}
