use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use worth_proof::TransitionOutcome;

use crate::domain_capabilities::identity::domain_capability_scope_encoder;
use crate::{WorthQueryEvidenceIdentity, WorthQueryEvidenceTag};

use crate::domain_capabilities::payloads::WorthQuerySupportContributionPosture;
use crate::domain_capabilities::targets::WorthQueryDomainCapabilityTargetBinding;
use crate::domain_capabilities::{
    WorthQueryCanonicalSupportTraceabilityArtifact, WorthQueryDomainCapabilityTransitionOutcome,
    WorthQueryMaterializationReadySupportContribution,
};

pub fn materialize_canonical_support_traceability_artifact<T>(
    contribution: WorthQueryMaterializationReadySupportContribution<T>,
) -> Result<WorthQueryCanonicalSupportTraceabilityArtifact<T>, IdentityDerivationDenial>
where
    T: WorthQueryDomainCapabilityTargetBinding,
{
    Ok(super::artifacts::materialize_domain_capability_canonical_runtime_artifact(contribution)?)
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryIntentDeclarationSupportTraceabilityArtifact {
    lane: &'static str,
    support_detail: String,
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

impl WorthQueryIntentDeclarationSupportTraceabilityArtifact {
    pub fn lane(&self) -> &'static str {
        self.lane
    }

    pub fn support_detail(&self) -> &str {
        &self.support_detail
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

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryLowerRuntimeBoundarySupportTraceabilityArtifact {
    lane: &'static str,
    support_detail: String,
    seam_key: crate::lower_runtime_routing::WorthQueryLowerRuntimeSeamKey,
    capability_label: &'static str,
    crossing_classification:
        crate::lower_runtime_routing::WorthQueryLowerRuntimeCrossingClassification,
    route_kind: crate::lower_runtime_routing::WorthQueryLowerRuntimeRouteKind,
    support_posture: crate::lower_runtime_routing::WorthQueryLowerRuntimeSupportPosture,
    envelope_identity: WorthQueryEvidenceIdentity,
    target_binding_identity: WorthQueryEvidenceIdentity,
    request_identity: WorthQueryEvidenceIdentity,
    materialization_identity: WorthQueryEvidenceIdentity,
}

impl WorthQueryLowerRuntimeBoundarySupportTraceabilityArtifact {
    pub fn lane(&self) -> &'static str {
        self.lane
    }

    pub fn support_detail(&self) -> &str {
        &self.support_detail
    }

    pub fn seam_key(&self) -> crate::lower_runtime_routing::WorthQueryLowerRuntimeSeamKey {
        self.seam_key
    }

    pub fn capability_label(&self) -> &'static str {
        self.capability_label
    }

    pub fn crossing_classification(
        &self,
    ) -> crate::lower_runtime_routing::WorthQueryLowerRuntimeCrossingClassification {
        self.crossing_classification
    }

    pub fn route_kind(&self) -> crate::lower_runtime_routing::WorthQueryLowerRuntimeRouteKind {
        self.route_kind
    }

    pub fn support_posture(
        &self,
    ) -> crate::lower_runtime_routing::WorthQueryLowerRuntimeSupportPosture {
        self.support_posture
    }

    pub fn envelope_for_reporting(&self) -> &str {
        self.envelope_identity.as_str()
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

pub(crate) fn materialize_intent_declaration_support_traceability_artifact<T>(
    contribution: WorthQueryMaterializationReadySupportContribution<T>,
) -> Result<
    WorthQueryDomainCapabilityTransitionOutcome<
        WorthQueryIntentDeclarationSupportTraceabilityArtifact,
    >,
    IdentityDerivationDenial,
>
where
    T: crate::domain_capabilities::WorthQueryDeclarationContributionTargetBinding,
{
    let domain_contribution = contribution.payload();
    let payload = domain_contribution.payload();
    let Some((name, strategy_name, strategy_version, input_contract, source_lane, target_lane)) =
        domain_contribution
            .target()
            .semantics()
            .intent_declaration()
    else {
        unreachable!("declaration-bound target should preserve declaration semantics");
    };
    Ok(TransitionOutcome::Success(
        WorthQueryIntentDeclarationSupportTraceabilityArtifact {
            lane: support_lane(payload.posture()),
            support_detail: support_detail(payload.semantic_code(), payload.detail()),
            intent_name: name.to_string(),
            strategy_name: strategy_name.to_string(),
            strategy_version: strategy_version.to_string(),
            input_contract: input_contract.to_string(),
            source_lane,
            target_lane,
            target_binding_identity: domain_contribution.target().binding_identity()?,
            request_identity: domain_contribution.request_identity().clone(),
            materialization_identity: domain_capability_scope_encoder(
                "worth_query_intent_declaration_support_traceability_artifact_v1",
            )
            .field_shape(
                WorthQueryEvidenceTag::new("lane"),
                support_lane(payload.posture()),
            )
            .field_shape(
                WorthQueryEvidenceTag::new("semantic_code"),
                payload.semantic_code(),
            )
            .field_shape(WorthQueryEvidenceTag::new("detail"), payload.detail())
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

pub(crate) fn materialize_lower_runtime_support_traceability_artifact<T>(
    contribution: WorthQueryMaterializationReadySupportContribution<T>,
) -> Result<
    WorthQueryDomainCapabilityTransitionOutcome<
        WorthQueryLowerRuntimeBoundarySupportTraceabilityArtifact,
    >,
    IdentityDerivationDenial,
>
where
    T: crate::domain_capabilities::WorthQueryLowerRuntimeContributionTargetBinding,
{
    let domain_contribution = contribution.payload();
    let payload = domain_contribution.payload();
    let Some((
        seam_key,
        capability_label,
        crossing_classification,
        route_kind,
        support_posture,
        _envelope_digest,
    )) = domain_contribution
        .target()
        .semantics()
        .lower_runtime_boundary()
    else {
        unreachable!("lower-runtime target should preserve lower-runtime semantics");
    };
    let envelope_identity = domain_contribution.target().binding_identity();
    let materialization_identity = domain_capability_scope_encoder(
        "worth_query_lower_runtime_support_traceability_artifact_v1",
    )
    .field_shape(
        WorthQueryEvidenceTag::new("lane"),
        support_lane(payload.posture()),
    )
    .field_shape(
        WorthQueryEvidenceTag::new("semantic_code"),
        payload.semantic_code(),
    )
    .field_shape(WorthQueryEvidenceTag::new("detail"), payload.detail())
    .field_shape(WorthQueryEvidenceTag::new("seam_key"), seam_key.as_str())
    .field_shape(WorthQueryEvidenceTag::new("capability"), capability_label)
    .field_shape(
        WorthQueryEvidenceTag::new("crossing"),
        crossing_classification.as_str(),
    )
    .field_shape(WorthQueryEvidenceTag::new("route"), route_kind.as_str())
    .field_shape(
        WorthQueryEvidenceTag::new("support_posture"),
        support_posture.as_str(),
    )
    .field_evidence_identity(WorthQueryEvidenceTag::new("envelope"), &envelope_identity)
    .field_evidence_identity(
        WorthQueryEvidenceTag::new("binding"),
        &domain_contribution.target().binding_identity(),
    )
    .field_evidence_identity(
        WorthQueryEvidenceTag::new("request"),
        domain_contribution.request_identity(),
    )
    .seal()?;
    Ok(TransitionOutcome::Success(
        WorthQueryLowerRuntimeBoundarySupportTraceabilityArtifact {
            lane: support_lane(payload.posture()),
            support_detail: support_detail(payload.semantic_code(), payload.detail()),
            seam_key,
            capability_label,
            crossing_classification,
            route_kind,
            support_posture,
            envelope_identity,
            target_binding_identity: domain_contribution.target().binding_identity()?,
            request_identity: domain_contribution.request_identity().clone(),
            materialization_identity,
        },
    ))
}

pub(super) fn support_lane(posture: WorthQuerySupportContributionPosture) -> &'static str {
    match posture {
        WorthQuerySupportContributionPosture::DeclarationSupport => "domain_support",
        WorthQuerySupportContributionPosture::DeclarationTraceability => "domain_traceability",
        WorthQuerySupportContributionPosture::NarrowedSupport => "domain_narrowed_support",
    }
}

pub(super) fn support_detail_label(semantic_code: &str, detail: &str) -> String {
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

fn support_detail(semantic_code: &str, detail: &str) -> String {
    support_detail_label(semantic_code, detail)
}
