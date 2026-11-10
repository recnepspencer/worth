use crate::basis_lifecycle::{BasisFamily, ScopedSubscriptionDeclarationBasis};
use crate::evidence_identity::WorthQueryEvidenceIdentity;
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::live::{LivePromotionDescriptor, LiveQueryFamily};
use crate::view_shape_live::LiveViewShapeFamily;

use super::construction_source::QuerySubscriptionConstructionSource;
use super::dimensions::QuerySubscriptionAdmissionDimensions;
use super::evidence_identities::{
    diagnostic_source_identity, lifecycle_context_collection_absent_identity,
    lifecycle_context_policy_identity, lifecycle_context_relationship_proof_identity,
    lifecycle_context_tenant_basis_identity, live_delivery_intent_projection_identity,
    live_relevance_identity,
};
use super::future_selection::QuerySubscriptionFutureSelection;
use super::posture::QuerySubscriptionBasisPosture;
use super::relationship_proof::QuerySubscriptionRelationshipProofPosture;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LiveQueryAdmissionArtifact {
    pub(super) live_family: LiveQueryFamily,
    pub(super) query_identity: WorthQueryEvidenceIdentity,
    pub(super) plan_identity: WorthQueryEvidenceIdentity,
    pub(super) collection_identity: WorthQueryEvidenceIdentity,
    pub(super) view_family: Option<LiveViewShapeFamily>,
    pub(super) scoped_declaration_basis: Option<ScopedSubscriptionDeclarationBasis>,
    pub(super) basis_posture: QuerySubscriptionBasisPosture,
    pub(super) future_selection: QuerySubscriptionFutureSelection,
    pub(super) policy_context_identity: WorthQueryEvidenceIdentity,
    pub(super) tenant_context_identity: WorthQueryEvidenceIdentity,
    pub(super) relationship_proof_context_identity: WorthQueryEvidenceIdentity,
    pub(super) policy_context_width: usize,
    pub(super) tenant_context_width: usize,
    pub(super) relationship_proof_posture: QuerySubscriptionRelationshipProofPosture,
    pub(super) relevance_identity: WorthQueryEvidenceIdentity,
    pub(super) delivery_intent_identity: WorthQueryEvidenceIdentity,
    pub(super) diagnostic_source_identity: WorthQueryEvidenceIdentity,
    pub(super) authorized_projection_width: usize,
    pub(super) ordering_width: usize,
    pub(super) grouping_width: usize,
    pub(super) relation_scope_width: usize,
    pub(super) view_shape_metadata_width: usize,
    pub(super) construction_source: QuerySubscriptionConstructionSource,
}

impl LiveQueryAdmissionArtifact {
    pub fn from_live_promotion(
        descriptor: &LivePromotionDescriptor,
        scoped_declaration_basis: ScopedSubscriptionDeclarationBasis,
        dimensions: QuerySubscriptionAdmissionDimensions,
    ) -> Result<Self, IdentityDerivationDenial> {
        Self::from_live_promotion_with_future_selection(
            descriptor,
            scoped_declaration_basis,
            QuerySubscriptionFutureSelection::ordinary()?,
            dimensions,
        )
    }

    pub fn from_live_promotion_with_future_selection(
        descriptor: &LivePromotionDescriptor,
        scoped_declaration_basis: ScopedSubscriptionDeclarationBasis,
        future_selection: QuerySubscriptionFutureSelection,
        dimensions: QuerySubscriptionAdmissionDimensions,
    ) -> Result<Self, IdentityDerivationDenial> {
        Self::from_promotion_parts(
            descriptor,
            scoped_declaration_basis,
            None,
            dimensions,
            QuerySubscriptionConstructionSource::FacadeLive,
            future_selection,
        )
    }

    pub fn from_live_promotion_with_view(
        descriptor: &LivePromotionDescriptor,
        scoped_declaration_basis: ScopedSubscriptionDeclarationBasis,
        view_family: LiveViewShapeFamily,
        dimensions: QuerySubscriptionAdmissionDimensions,
    ) -> Result<Self, IdentityDerivationDenial> {
        Self::from_live_promotion_with_view_and_future_selection(
            descriptor,
            scoped_declaration_basis,
            view_family,
            QuerySubscriptionFutureSelection::ordinary()?,
            dimensions,
        )
    }

    pub fn from_live_promotion_with_view_and_future_selection(
        descriptor: &LivePromotionDescriptor,
        scoped_declaration_basis: ScopedSubscriptionDeclarationBasis,
        view_family: LiveViewShapeFamily,
        future_selection: QuerySubscriptionFutureSelection,
        dimensions: QuerySubscriptionAdmissionDimensions,
    ) -> Result<Self, IdentityDerivationDenial> {
        Self::from_promotion_parts(
            descriptor,
            scoped_declaration_basis,
            Some(view_family),
            dimensions,
            QuerySubscriptionConstructionSource::FacadeLive,
            future_selection,
        )
    }

    fn from_promotion_parts(
        descriptor: &LivePromotionDescriptor,
        scoped_declaration_basis: ScopedSubscriptionDeclarationBasis,
        view_family: Option<LiveViewShapeFamily>,
        dimensions: QuerySubscriptionAdmissionDimensions,
        construction_source: QuerySubscriptionConstructionSource,
        future_selection: QuerySubscriptionFutureSelection,
    ) -> Result<Self, IdentityDerivationDenial> {
        let basis_posture = subscription_posture_for_basis(&scoped_declaration_basis);
        let query_identity =
            crate::identity::validated_query_evidence_identity(descriptor.query_digest())?;
        let plan_identity = descriptor.plan_digest().evidence_identity().clone();
        let collection_identity = descriptor
            .collection_digest()
            .map(crate::identity::collection_plan_evidence_identity)
            .unwrap_or_else(|| Ok(lifecycle_context_collection_absent_identity()))?;
        let relevance_identity =
            live_relevance_identity(descriptor.family(), &query_identity, &plan_identity);
        let delivery_intent_identity =
            live_delivery_intent_projection_identity(descriptor.family());
        let policy_context_identity = lifecycle_context_policy_identity("none")?;
        let tenant_context_identity = lifecycle_context_tenant_basis_identity("none")?;
        let relationship_proof_context_identity =
            lifecycle_context_relationship_proof_identity("none")?;
        let mut artifact = Self {
            live_family: descriptor.family().clone(),
            query_identity,
            plan_identity,
            collection_identity,
            view_family,
            scoped_declaration_basis: Some(scoped_declaration_basis),
            basis_posture,
            future_selection,
            policy_context_identity,
            tenant_context_identity,
            relationship_proof_context_identity,
            policy_context_width: "none".len(),
            tenant_context_width: "none".len(),
            relationship_proof_posture: QuerySubscriptionRelationshipProofPosture::NotRequired,
            relevance_identity,
            delivery_intent_identity,
            diagnostic_source_identity: WorthQueryEvidenceIdentity::compose(
                crate::evidence_identity::WorthQueryEvidenceScope::SubscriptionActivationReceipt,
            )
            .seal()?,
            authorized_projection_width: dimensions.authorized_projection_width,
            ordering_width: dimensions.ordering_width,
            grouping_width: dimensions.grouping_width,
            relation_scope_width: dimensions.relation_scope_width,
            view_shape_metadata_width: dimensions.view_shape_metadata_width,
            construction_source,
        };
        artifact.diagnostic_source_identity = diagnostic_source_identity(&artifact)?;
        Ok(artifact)
    }

    pub fn live_family(&self) -> &LiveQueryFamily {
        &self.live_family
    }

    pub fn view_family(&self) -> Option<LiveViewShapeFamily> {
        self.view_family
    }

    pub fn basis_posture(&self) -> &QuerySubscriptionBasisPosture {
        &self.basis_posture
    }

    pub fn scoped_declaration_basis(&self) -> Option<&ScopedSubscriptionDeclarationBasis> {
        self.scoped_declaration_basis.as_ref()
    }

    pub fn future_selection(&self) -> &QuerySubscriptionFutureSelection {
        &self.future_selection
    }

    pub fn construction_source(&self) -> &QuerySubscriptionConstructionSource {
        &self.construction_source
    }

    pub fn query_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.query_identity
    }

    pub(super) fn plan_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.plan_identity
    }

    pub(super) fn collection_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.collection_identity
    }

    pub(super) fn ordering_width(&self) -> usize {
        self.ordering_width
    }

    pub(super) fn grouping_width(&self) -> usize {
        self.grouping_width
    }

    pub(super) fn relation_scope_width(&self) -> usize {
        self.relation_scope_width
    }

    pub(super) fn relationship_proof_posture(&self) -> &QuerySubscriptionRelationshipProofPosture {
        &self.relationship_proof_posture
    }

    pub(super) fn policy_context_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.policy_context_identity
    }

    pub(super) fn tenant_context_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.tenant_context_identity
    }

    pub(super) fn relationship_proof_context_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.relationship_proof_context_identity
    }

    pub(super) fn policy_context_width(&self) -> usize {
        self.policy_context_width
    }

    pub(super) fn tenant_context_width(&self) -> usize {
        self.tenant_context_width
    }

    pub fn relevance_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.relevance_identity
    }

    pub fn delivery_intent_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.delivery_intent_identity
    }

    pub fn diagnostic_source_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.diagnostic_source_identity
    }

    pub fn authorized_projection_width(&self) -> usize {
        self.authorized_projection_width
    }

    pub fn view_shape_metadata_width(&self) -> usize {
        self.view_shape_metadata_width
    }
}

fn subscription_posture_for_basis(
    basis: &ScopedSubscriptionDeclarationBasis,
) -> QuerySubscriptionBasisPosture {
    match basis.family() {
        BasisFamily::CurrentHead | BasisFamily::TenantScoped | BasisFamily::PolicyScoped => {
            QuerySubscriptionBasisPosture::CurrentHead
        }
        BasisFamily::BranchHead | BasisFamily::BranchSnapshot => {
            QuerySubscriptionBasisPosture::BranchHead
        }
        BasisFamily::RuntimeSnapshot
        | BasisFamily::HistoricalSnapshot
        | BasisFamily::HistoricalCommit => QuerySubscriptionBasisPosture::RuntimeHistoricalSnapshot,
        BasisFamily::Preview | BasisFamily::PreviewDerived => {
            QuerySubscriptionBasisPosture::PreviewScoped
        }
        BasisFamily::StoreBacked | BasisFamily::DurableReload => {
            QuerySubscriptionBasisPosture::DeniedUnsupportedBasis
        }
    }
}
