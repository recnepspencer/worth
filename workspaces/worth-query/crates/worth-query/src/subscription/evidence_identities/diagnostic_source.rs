use super::super::input::LiveQueryAdmissionArtifact;
use super::SUBSCRIPTION_IDENTITY_SCOPE;
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::evidence_identity::{WorthQueryEvidenceIdentity, WorthQueryEvidenceTag};

pub(in crate::subscription) fn diagnostic_source_identity(
    live: &LiveQueryAdmissionArtifact,
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    WorthQueryEvidenceIdentity::compose(SUBSCRIPTION_IDENTITY_SCOPE)
        .field_shape(
            WorthQueryEvidenceTag::new("identity_family"),
            "query_subscription_live_admission_source_v1",
        )
        .field_shape(
            WorthQueryEvidenceTag::new("live_family"),
            live.live_family().as_str(),
        )
        .field_evidence_identity(WorthQueryEvidenceTag::new("query"), live.query_identity())
        .field_evidence_identity(WorthQueryEvidenceTag::new("plan"), live.plan_identity())
        .field_evidence_identity(
            WorthQueryEvidenceTag::new("collection"),
            live.collection_identity(),
        )
        .field_shape(
            WorthQueryEvidenceTag::new("view_family"),
            live.view_family()
                .map(|family| family.as_str())
                .unwrap_or("none"),
        )
        .field_shape(
            WorthQueryEvidenceTag::new("basis"),
            live.basis_posture().as_str(),
        )
        .field_evidence_identity(
            WorthQueryEvidenceTag::new("future_selection"),
            live.future_selection().projection_identity(),
        )
        .field_evidence_identity(
            WorthQueryEvidenceTag::new("policy"),
            live.policy_context_identity(),
        )
        .field_evidence_identity(
            WorthQueryEvidenceTag::new("tenant"),
            live.tenant_context_identity(),
        )
        .field_evidence_identity(
            WorthQueryEvidenceTag::new("relationship_proof"),
            live.relationship_proof_context_identity(),
        )
        .field_shape(
            WorthQueryEvidenceTag::new("relationship_proof_posture"),
            live.relationship_proof_posture().as_str(),
        )
        .field_evidence_identity(
            WorthQueryEvidenceTag::new("relevance"),
            live.relevance_identity(),
        )
        .field_evidence_identity(
            WorthQueryEvidenceTag::new("delivery"),
            live.delivery_intent_identity(),
        )
        .field_usize(
            WorthQueryEvidenceTag::new("projection_width"),
            live.authorized_projection_width(),
        )
        .field_usize(
            WorthQueryEvidenceTag::new("ordering_width"),
            live.ordering_width(),
        )
        .field_usize(
            WorthQueryEvidenceTag::new("grouping_width"),
            live.grouping_width(),
        )
        .field_usize(
            WorthQueryEvidenceTag::new("relation_scope_width"),
            live.relation_scope_width(),
        )
        .field_usize(
            WorthQueryEvidenceTag::new("metadata_width"),
            live.view_shape_metadata_width(),
        )
        .field_shape(
            WorthQueryEvidenceTag::new("source"),
            live.construction_source().as_str(),
        )
        .seal()
}
