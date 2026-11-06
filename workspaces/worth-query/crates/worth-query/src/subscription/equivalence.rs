use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::evidence_identity::{
    WorthQueryEvidenceIdentity, WorthQueryEvidenceScope, WorthQueryEvidenceTag,
};
use crate::live::LiveQueryFamily;
use crate::view_shape_live::LiveViewShapeFamily;

use super::family::QuerySubscriptionFamily;
use super::future_selection::QuerySubscriptionFutureSelection;
use super::input::LiveQueryAdmissionArtifact;
use super::posture::{
    QuerySubscriptionBasisPosture, QuerySubscriptionBridgePosture, QuerySubscriptionCostPosture,
};
use super::selection::FamilyClassification;

const EQUIVALENCE_IDENTITY_SCOPE: WorthQueryEvidenceScope =
    WorthQueryEvidenceScope::SubscriptionActivationReceipt;

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub(crate) struct QuerySubscriptionMeaningDigest(String);

impl QuerySubscriptionMeaningDigest {
    fn from_evidence_identity(identity: &WorthQueryEvidenceIdentity) -> Self {
        Self(identity.as_str().to_string())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QuerySubscriptionEquivalenceBasis {
    digest: QuerySubscriptionMeaningDigest,
    equivalence_identity: WorthQueryEvidenceIdentity,
    digest_part_count: usize,
    family: QuerySubscriptionFamily,
    live_family: LiveQueryFamily,
    view_family: Option<LiveViewShapeFamily>,
    future_selection: QuerySubscriptionFutureSelection,
    cost_posture: QuerySubscriptionCostPosture,
    basis_posture: QuerySubscriptionBasisPosture,
    bridge_posture: QuerySubscriptionBridgePosture,
}

impl QuerySubscriptionEquivalenceBasis {
    pub(super) fn new(
        input: &LiveQueryAdmissionArtifact,
        classification: &FamilyClassification,
    ) -> Self {
        let equivalence_identity = compose_equivalence_identity(input, classification);
        let digest = QuerySubscriptionMeaningDigest::from_evidence_identity(&equivalence_identity);
        let digest_part_count = equivalence_digest_part_count(input, classification);
        Self {
            digest,
            equivalence_identity,
            digest_part_count,
            family: classification.family.clone(),
            live_family: input.live_family.clone(),
            view_family: input.view_family,
            future_selection: input.future_selection.clone(),
            cost_posture: classification.cost_posture.clone(),
            basis_posture: input.basis_posture.clone(),
            bridge_posture: classification.bridge_posture.clone(),
        }
    }

    pub fn evidence_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.equivalence_identity
    }
    pub fn digest_part_count(&self) -> usize {
        self.digest_part_count
    }

    pub fn family(&self) -> &QuerySubscriptionFamily {
        &self.family
    }

    pub fn live_family(&self) -> &LiveQueryFamily {
        &self.live_family
    }

    pub fn view_family(&self) -> Option<LiveViewShapeFamily> {
        self.view_family
    }

    pub fn cost_posture(&self) -> &QuerySubscriptionCostPosture {
        &self.cost_posture
    }

    pub fn future_selection(&self) -> &QuerySubscriptionFutureSelection {
        &self.future_selection
    }

    pub fn basis_posture(&self) -> &QuerySubscriptionBasisPosture {
        &self.basis_posture
    }

    pub fn bridge_posture(&self) -> &QuerySubscriptionBridgePosture {
        &self.bridge_posture
    }
}

/// Must stay aligned with the field cardinality of `compose_equivalence_identity`.
const EQUIVALENCE_DIGEST_PART_COUNT: usize = 23;

fn equivalence_digest_part_count(
    _input: &LiveQueryAdmissionArtifact,
    _classification: &FamilyClassification,
) -> usize {
    EQUIVALENCE_DIGEST_PART_COUNT
}

fn compose_equivalence_identity(
    input: &LiveQueryAdmissionArtifact,
    classification: &FamilyClassification,
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    Ok(
        WorthQueryEvidenceIdentity::compose(EQUIVALENCE_IDENTITY_SCOPE)
            .field_shape(
                WorthQueryEvidenceTag::new("identity_family"),
                "subscription_lifecycle_equivalence_v1",
            )
            .field_shape(
                WorthQueryEvidenceTag::new("family"),
                classification.family.as_str(),
            )
            .field_shape(
                WorthQueryEvidenceTag::new("live_family"),
                input.live_family.as_str(),
            )
            .field_shape(
                WorthQueryEvidenceTag::new("view_family"),
                input
                    .view_family
                    .map(|family| family.as_str())
                    .unwrap_or("none"),
            )
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("future_selection"),
                input.future_selection.projection_identity(),
            )
            .field_shape(
                WorthQueryEvidenceTag::new("basis"),
                input.basis_posture.as_str(),
            )
            .field_shape(
                WorthQueryEvidenceTag::new("scoped_declaration_basis"),
                input
                    .scoped_declaration_basis()
                    .map(|basis| basis.scoped_basis_digest())
                    .unwrap_or("missing"),
            )
            .field_shape(
                WorthQueryEvidenceTag::new("cost"),
                classification.cost_posture.as_str(),
            )
            .field_shape(
                WorthQueryEvidenceTag::new("bridge"),
                classification.bridge_posture.as_str(),
            )
            .field_evidence_identity(WorthQueryEvidenceTag::new("query"), input.query_identity())
            .field_evidence_identity(WorthQueryEvidenceTag::new("plan"), input.plan_identity())
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("collection"),
                input.collection_identity(),
            )
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("policy"),
                input.policy_context_identity(),
            )
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("tenant"),
                input.tenant_context_identity(),
            )
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("relationship_proof"),
                input.relationship_proof_context_identity(),
            )
            .field_shape(
                WorthQueryEvidenceTag::new("relationship_proof_posture"),
                input.relationship_proof_posture.as_str(),
            )
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("relevance"),
                input.relevance_identity(),
            )
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("delivery_intent"),
                input.delivery_intent_identity(),
            )
            .field_usize(
                WorthQueryEvidenceTag::new("authorized_width"),
                input.authorized_projection_width,
            )
            .field_usize(
                WorthQueryEvidenceTag::new("ordering_width"),
                input.ordering_width,
            )
            .field_usize(
                WorthQueryEvidenceTag::new("grouping_width"),
                input.grouping_width,
            )
            .field_usize(
                WorthQueryEvidenceTag::new("relation_scope_width"),
                input.relation_scope_width,
            )
            .field_usize(
                WorthQueryEvidenceTag::new("view_metadata_width"),
                input.view_shape_metadata_width,
            )
            .seal()?,
    )
}
