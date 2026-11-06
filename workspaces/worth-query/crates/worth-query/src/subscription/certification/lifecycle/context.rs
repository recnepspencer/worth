use crate::evidence_identity::WorthQueryEvidenceIdentity;
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;

use super::super::super::evidence_identities::{
    lifecycle_context_basis_posture_identity, lifecycle_context_query_identity,
    lifecycle_context_view_shape_identity, lifecycle_subscription_equivalence_identity,
    lifecycle_subscription_family_identity,
};
use super::super::super::input::LiveQueryAdmissionArtifact;
use super::super::super::selection::QuerySubscriptionFamilySelection;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SubscriptionLifecycleCertificationContext {
    pub(in crate::subscription) query_scope_identity: WorthQueryEvidenceIdentity,
    pub(in crate::subscription) subscription_family_identity: WorthQueryEvidenceIdentity,
    pub(in crate::subscription) subscription_equivalence_identity: WorthQueryEvidenceIdentity,
    pub(in crate::subscription) policy_identity: WorthQueryEvidenceIdentity,
    pub(in crate::subscription) tenant_basis_identity: WorthQueryEvidenceIdentity,
    pub(in crate::subscription) relationship_proof_identity: WorthQueryEvidenceIdentity,
    pub(in crate::subscription) view_shape_identity: WorthQueryEvidenceIdentity,
    pub(in crate::subscription) basis_posture_identity: WorthQueryEvidenceIdentity,
}

impl SubscriptionLifecycleCertificationContext {
    pub(crate) fn admitted(
        query_scope_identity: WorthQueryEvidenceIdentity,
        subscription_family_identity: WorthQueryEvidenceIdentity,
        subscription_equivalence_identity: WorthQueryEvidenceIdentity,
        policy_identity: WorthQueryEvidenceIdentity,
        tenant_basis_identity: WorthQueryEvidenceIdentity,
        relationship_proof_identity: WorthQueryEvidenceIdentity,
        view_shape_identity: WorthQueryEvidenceIdentity,
        basis_posture_identity: WorthQueryEvidenceIdentity,
    ) -> Self {
        Self {
            query_scope_identity,
            subscription_family_identity,
            subscription_equivalence_identity,
            policy_identity,
            tenant_basis_identity,
            relationship_proof_identity,
            view_shape_identity,
            basis_posture_identity,
        }
    }

    pub fn from_live_selection(
        live: &LiveQueryAdmissionArtifact,
        selection: &QuerySubscriptionFamilySelection,
    ) -> Result<Self, IdentityDerivationDenial> {
        let subscription_family_identity =
            lifecycle_subscription_family_identity(selection.family())?;
        let subscription_equivalence_identity =
            lifecycle_subscription_equivalence_identity(selection.equivalence_basis());
        let query_scope_identity = lifecycle_context_query_identity(live)?;
        Ok(Self::admitted(
            query_scope_identity,
            subscription_family_identity,
            subscription_equivalence_identity,
            live.policy_context_identity().clone(),
            live.tenant_context_identity().clone(),
            live.relationship_proof_context_identity().clone(),
            lifecycle_context_view_shape_identity(
                live.view_family().map(|family| family.as_str()),
            )?,
            lifecycle_context_basis_posture_identity(live.basis_posture().as_str())?,
        ))
    }

    pub fn query_scope_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.query_scope_identity
    }

    pub fn subscription_family_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.subscription_family_identity
    }

    pub fn subscription_equivalence_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.subscription_equivalence_identity
    }

    pub fn policy_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.policy_identity
    }

    pub fn tenant_basis_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.tenant_basis_identity
    }

    pub fn relationship_proof_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.relationship_proof_identity
    }

    pub fn view_shape_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.view_shape_identity
    }

    pub fn basis_posture_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.basis_posture_identity
    }
}
