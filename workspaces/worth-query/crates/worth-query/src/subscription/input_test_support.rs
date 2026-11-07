use crate::basis_lifecycle::{basis_lifecycle, ScopedSubscriptionDeclarationBasis};
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::identity::CollectionPlanDigest;
use crate::live::LiveQueryFamily;
use crate::view_shape_live::LiveViewShapeFamily;

use super::construction_source::QuerySubscriptionConstructionSource;
use super::evidence_identities::{
    diagnostic_source_identity, lifecycle_context_policy_identity,
    lifecycle_context_relationship_proof_identity, lifecycle_context_tenant_basis_identity,
    live_delivery_intent_projection_identity, live_relevance_identity,
};
use super::future_selection::QuerySubscriptionFutureSelection;
use super::input::LiveQueryAdmissionArtifact;
use super::posture::QuerySubscriptionBasisPosture;
use super::relationship_proof::QuerySubscriptionRelationshipProofPosture;

impl LiveQueryAdmissionArtifact {
    pub(crate) fn for_test(
        live_family: LiveQueryFamily,
        view_family: Option<LiveViewShapeFamily>,
        construction_source: QuerySubscriptionConstructionSource,
    ) -> Result<Self, IdentityDerivationDenial> {
        Ok(Self::for_test_with_basis(
            live_family,
            view_family,
            construction_source,
            QuerySubscriptionBasisPosture::CurrentHead,
        )?)
    }

    pub(crate) fn for_test_with_basis(
        live_family: LiveQueryFamily,
        view_family: Option<LiveViewShapeFamily>,
        construction_source: QuerySubscriptionConstructionSource,
        basis_posture: QuerySubscriptionBasisPosture,
    ) -> Result<Self, IdentityDerivationDenial> {
        Ok(Self::for_test_with_context(
            live_family,
            view_family,
            construction_source,
            basis_posture,
            QuerySubscriptionFutureSelection::ordinary()?,
            Some("policy".to_string()),
            Some("tenant".to_string()),
            Some("relationship-proof".to_string()),
            QuerySubscriptionRelationshipProofPosture::Admitted,
        )?)
    }

    pub(crate) fn for_test_with_context(
        live_family: LiveQueryFamily,
        view_family: Option<LiveViewShapeFamily>,
        construction_source: QuerySubscriptionConstructionSource,
        basis_posture: QuerySubscriptionBasisPosture,
        future_selection: QuerySubscriptionFutureSelection,
        policy_label: Option<String>,
        tenant_label: Option<String>,
        relationship_proof_label: Option<String>,
        relationship_proof_posture: QuerySubscriptionRelationshipProofPosture,
    ) -> Result<Self, IdentityDerivationDenial> {
        let (
            authorized_projection_width,
            ordering_width,
            grouping_width,
            relation_scope_width,
            view_shape_metadata_width,
        ) = match (live_family.clone(), view_family) {
            (_, Some(LiveViewShapeFamily::Table)) => (2, 1, 0, 0, 0),
            (_, Some(LiveViewShapeFamily::KanbanGrouped)) => (2, 1, 1, 0, 1),
            (_, Some(LiveViewShapeFamily::Detail)) => (2, 0, 0, 0, 0),
            (_, Some(LiveViewShapeFamily::InspectorDetailObserved))
            | (_, Some(LiveViewShapeFamily::InspectorDetailFocused)) => (2, 0, 0, 0, 1),
            (LiveQueryFamily::Detail, None) => (2, 0, 0, 0, 0),
            (LiveQueryFamily::OrderedCollection, None) => (2, 1, 0, 0, 0),
            (LiveQueryFamily::BoundedMaterialization, None) => (2, 1, 0, 1, 0),
        };
        let query_identity = crate::evidence_identity::WorthQueryEvidenceIdentity::compose(
            crate::evidence_identity::WorthQueryEvidenceScope::MutationEvidenceSourceDigest,
        )
        .field_shape(
            crate::evidence_identity::WorthQueryEvidenceTag::new("identity_family"),
            "validated_query_digest_v1",
        )
        .field_value(
            crate::evidence_identity::WorthQueryEvidenceTag::new("validated_query_digest"),
            "query-digest",
        )
        .seal()?;
        let plan_identity = crate::evidence_identity::WorthQueryEvidenceIdentity::compose(
            crate::evidence_identity::WorthQueryEvidenceScope::MutationEvidenceSourceDigest,
        )
        .field_shape(
            crate::evidence_identity::WorthQueryEvidenceTag::new("identity_family"),
            "execution_plan_digest_v1",
        )
        .field_value(
            crate::evidence_identity::WorthQueryEvidenceTag::new("plan_digest"),
            "plan-digest",
        )
        .seal()?;
        let collection_digest =
            CollectionPlanDigest::from_parts(&["collection-digest".to_string()]);
        let collection_identity =
            crate::identity::collection_plan_evidence_identity(&collection_digest)?;
        let relevance_identity =
            live_relevance_identity(&live_family, &query_identity, &plan_identity);
        let delivery_intent_identity = live_delivery_intent_projection_identity(&live_family);
        let policy_context_identity =
            lifecycle_context_policy_identity(policy_label.as_deref().unwrap_or("none"))?;
        let tenant_context_identity =
            lifecycle_context_tenant_basis_identity(tenant_label.as_deref().unwrap_or("none"))?;
        let relationship_proof_context_identity = lifecycle_context_relationship_proof_identity(
            relationship_proof_label.as_deref().unwrap_or("none"),
        )?;
        let policy_context_width = policy_label.as_deref().unwrap_or("none").len();
        let tenant_context_width = tenant_label.as_deref().unwrap_or("none").len();
        let scoped_declaration_basis = scoped_declaration_basis_for_test(&basis_posture);
        let mut artifact = Self {
            live_family,
            query_identity,
            plan_identity,
            collection_identity,
            view_family,
            scoped_declaration_basis,
            basis_posture,
            future_selection,
            policy_context_identity,
            tenant_context_identity,
            relationship_proof_context_identity,
            policy_context_width,
            tenant_context_width,
            relationship_proof_posture,
            relevance_identity,
            delivery_intent_identity,
            diagnostic_source_identity: crate::evidence_identity::WorthQueryEvidenceIdentity::compose(
                crate::evidence_identity::WorthQueryEvidenceScope::SubscriptionActivationReceipt,
            )
            .seal()?,
            authorized_projection_width,
            ordering_width,
            grouping_width,
            relation_scope_width,
            view_shape_metadata_width,
            construction_source,
        };
        artifact.diagnostic_source_identity = diagnostic_source_identity(&artifact)?;
        Ok(artifact)
    }
    pub(crate) fn for_test_grouped_with_missing_grouping_width(
    ) -> Result<Self, IdentityDerivationDenial> {
        let mut artifact = Self::for_test(
            LiveQueryFamily::OrderedCollection,
            Some(LiveViewShapeFamily::KanbanGrouped),
            QuerySubscriptionConstructionSource::FacadeLive,
        )?;
        artifact.grouping_width = 0;
        Ok(artifact)
    }

    pub(crate) fn for_test_with_relationship_proof_posture(
        live_family: LiveQueryFamily,
        view_family: Option<LiveViewShapeFamily>,
        construction_source: QuerySubscriptionConstructionSource,
        relationship_proof_posture: QuerySubscriptionRelationshipProofPosture,
    ) -> Result<Self, IdentityDerivationDenial> {
        Ok(Self::for_test_with_context(
            live_family,
            view_family,
            construction_source,
            QuerySubscriptionBasisPosture::CurrentHead,
            QuerySubscriptionFutureSelection::ordinary()?,
            Some("policy".to_string()),
            Some("tenant".to_string()),
            Some("relationship-proof".to_string()),
            relationship_proof_posture,
        )?)
    }

    pub(crate) fn for_test_with_future_selection(
        live_family: LiveQueryFamily,
        view_family: Option<LiveViewShapeFamily>,
        construction_source: QuerySubscriptionConstructionSource,
        future_selection: QuerySubscriptionFutureSelection,
    ) -> Result<Self, IdentityDerivationDenial> {
        Ok(Self::for_test_with_context(
            live_family,
            view_family,
            construction_source,
            QuerySubscriptionBasisPosture::CurrentHead,
            future_selection,
            Some("policy".to_string()),
            Some("tenant".to_string()),
            Some("relationship-proof".to_string()),
            QuerySubscriptionRelationshipProofPosture::Admitted,
        )?)
    }
}

fn scoped_declaration_basis_for_test(
    posture: &QuerySubscriptionBasisPosture,
) -> Option<ScopedSubscriptionDeclarationBasis> {
    let lifecycle = basis_lifecycle();
    let declaration = match posture {
        QuerySubscriptionBasisPosture::CurrentHead => lifecycle.current_head(),
        QuerySubscriptionBasisPosture::BranchHead => lifecycle.branch_head("test-branch", true),
        QuerySubscriptionBasisPosture::RuntimeHistoricalSnapshot => {
            lifecycle.historical_snapshot("test-snapshot", true)
        }
        QuerySubscriptionBasisPosture::PreviewScoped => lifecycle.preview("test-preview"),
        QuerySubscriptionBasisPosture::DeniedUnsupportedBasis => return None,
    };
    Some(
        declaration
            .declare_subscription()
            .expect("test subscription basis should admit"),
    )
}
