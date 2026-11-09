use crate::evidence_identity::WorthQueryEvidenceIdentity;
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::identity_authority::{QueryProjectionIdentity, QuerySubscriptionIdentityKind};

use super::certification::{
    QuerySubscriptionCertificationBundle, QuerySubscriptionCertificationError,
    SubscriptionLifecycleCertificationBundle, SubscriptionLifecycleCertificationContext,
    SubscriptionLifecycleCertificationError,
};
use super::evidence_projection::subscription_evidence_projection;

impl QuerySubscriptionCertificationBundle {
    pub fn certification_bundle_projection(
        &self,
    ) -> QueryProjectionIdentity<String, QuerySubscriptionIdentityKind> {
        subscription_evidence_projection(&self.certification_bundle_identity)
    }

    pub fn certification_bundle_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.certification_bundle_identity
    }

    pub fn admission_projection(
        &self,
    ) -> QueryProjectionIdentity<String, QuerySubscriptionIdentityKind> {
        subscription_evidence_projection(&self.admission_identity)
    }

    pub fn admission_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.admission_identity
    }

    pub fn activation_projection(
        &self,
    ) -> QueryProjectionIdentity<String, QuerySubscriptionIdentityKind> {
        subscription_evidence_projection(&self.activation_identity)
    }

    pub fn activation_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.activation_identity
    }

    pub fn query_declaration_projection(
        &self,
    ) -> QueryProjectionIdentity<String, QuerySubscriptionIdentityKind> {
        subscription_evidence_projection(&self.query_declaration_identity)
    }

    pub fn query_declaration_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.query_declaration_identity
    }

    pub fn bridge_declaration_projection(
        &self,
    ) -> QueryProjectionIdentity<String, QuerySubscriptionIdentityKind> {
        subscription_evidence_projection(&self.bridge_declaration_identity)
    }

    pub fn bridge_declaration_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.bridge_declaration_identity
    }

    pub fn basis_binding_projection(
        &self,
    ) -> QueryProjectionIdentity<String, QuerySubscriptionIdentityKind> {
        subscription_evidence_projection(&self.basis_binding_identity)
    }

    pub fn basis_binding_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.basis_binding_identity
    }

    pub fn signal_strategy_projection(
        &self,
    ) -> QueryProjectionIdentity<String, QuerySubscriptionIdentityKind> {
        subscription_evidence_projection(&self.signal_strategy_identity)
    }

    pub fn signal_strategy_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.signal_strategy_identity
    }

    pub fn diagnostics_projection(
        &self,
    ) -> QueryProjectionIdentity<String, QuerySubscriptionIdentityKind> {
        subscription_evidence_projection(&self.diagnostics_identity)
    }

    pub fn diagnostics_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.diagnostics_identity
    }

    pub fn support_profile_projection(
        &self,
    ) -> QueryProjectionIdentity<String, QuerySubscriptionIdentityKind> {
        subscription_evidence_projection(&self.support_profile_identity)
    }

    pub fn support_profile_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.support_profile_identity
    }

    pub fn admission_counter_projection(
        &self,
    ) -> QueryProjectionIdentity<String, QuerySubscriptionIdentityKind> {
        subscription_evidence_projection(&self.admission_counter_identity)
    }

    pub fn admission_counter_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.admission_counter_identity
    }

    pub fn activation_counter_projection(
        &self,
    ) -> QueryProjectionIdentity<String, QuerySubscriptionIdentityKind> {
        subscription_evidence_projection(&self.activation_counter_identity)
    }

    pub fn activation_counter_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.activation_counter_identity
    }

    pub fn scale_slope_projection(
        &self,
    ) -> QueryProjectionIdentity<String, QuerySubscriptionIdentityKind> {
        subscription_evidence_projection(&self.scale_slope_identity)
    }

    pub fn scale_slope_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.scale_slope_identity
    }

    pub fn scale_activation_projection(
        &self,
    ) -> QueryProjectionIdentity<String, QuerySubscriptionIdentityKind> {
        subscription_evidence_projection(&self.scale_activation_identity)
    }

    pub fn scale_activation_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.scale_activation_identity
    }

    pub fn scale_admission_projection(
        &self,
    ) -> QueryProjectionIdentity<String, QuerySubscriptionIdentityKind> {
        subscription_evidence_projection(&self.scale_admission_identity)
    }

    pub fn scale_admission_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.scale_admission_identity
    }
}

impl SubscriptionLifecycleCertificationContext {
    pub fn subscription_family_projection(
        &self,
    ) -> QueryProjectionIdentity<String, QuerySubscriptionIdentityKind> {
        subscription_evidence_projection(&self.subscription_family_identity)
    }

    pub fn subscription_equivalence_projection(
        &self,
    ) -> QueryProjectionIdentity<String, QuerySubscriptionIdentityKind> {
        subscription_evidence_projection(&self.subscription_equivalence_identity)
    }

    pub fn policy_projection(
        &self,
    ) -> QueryProjectionIdentity<String, QuerySubscriptionIdentityKind> {
        subscription_evidence_projection(&self.policy_identity)
    }

    pub fn tenant_basis_projection(
        &self,
    ) -> QueryProjectionIdentity<String, QuerySubscriptionIdentityKind> {
        subscription_evidence_projection(&self.tenant_basis_identity)
    }

    pub fn relationship_proof_projection(
        &self,
    ) -> QueryProjectionIdentity<String, QuerySubscriptionIdentityKind> {
        subscription_evidence_projection(&self.relationship_proof_identity)
    }

    pub fn view_shape_projection(
        &self,
    ) -> QueryProjectionIdentity<String, QuerySubscriptionIdentityKind> {
        subscription_evidence_projection(&self.view_shape_identity)
    }

    pub fn basis_posture_projection(
        &self,
    ) -> QueryProjectionIdentity<String, QuerySubscriptionIdentityKind> {
        subscription_evidence_projection(&self.basis_posture_identity)
    }

    pub fn query_scope_projection(
        &self,
    ) -> QueryProjectionIdentity<String, QuerySubscriptionIdentityKind> {
        subscription_evidence_projection(&self.query_scope_identity)
    }
}

macro_rules! lifecycle_bundle_projection {
    ($method:ident, $field:ident) => {
        pub fn $method(&self) -> QueryProjectionIdentity<String, QuerySubscriptionIdentityKind> {
            subscription_evidence_projection(&self.$field)
        }
    };
}

impl SubscriptionLifecycleCertificationBundle {
    pub fn certification_bundle_projection(
        &self,
    ) -> QueryProjectionIdentity<String, QuerySubscriptionIdentityKind> {
        subscription_evidence_projection(self.certification_bundle_identity())
    }

    lifecycle_bundle_projection!(subscription_family_projection, subscription_family_identity);
    lifecycle_bundle_projection!(
        subscription_declaration_projection,
        subscription_declaration_identity
    );
    lifecycle_bundle_projection!(
        subscription_equivalence_projection,
        subscription_equivalence_identity
    );
    lifecycle_bundle_projection!(admission_projection, admission_identity);
    lifecycle_bundle_projection!(active_lane_projection, active_lane_identity);
    lifecycle_bundle_projection!(active_lane_handle_projection, active_lane_handle_identity);
    lifecycle_bundle_projection!(
        active_lane_lookup_class_projection,
        active_lane_lookup_class_identity
    );
    lifecycle_bundle_projection!(subscription_budget_projection, subscription_budget_identity);
    lifecycle_bundle_projection!(
        subscription_performance_receipt_projection,
        subscription_performance_receipt_identity
    );
    lifecycle_bundle_projection!(consumer_attachment_projection, consumer_attachment_identity);
    lifecycle_bundle_projection!(
        acknowledgement_frontier_projection,
        acknowledgement_frontier_identity
    );
    lifecycle_bundle_projection!(delivery_window_projection, delivery_window_identity);
    lifecycle_bundle_projection!(maintenance_delta_projection, maintenance_delta_identity);
    lifecycle_bundle_projection!(
        active_delivery_work_packet_projection,
        active_delivery_work_packet_identity
    );
    lifecycle_bundle_projection!(
        active_delivery_density_posture_projection,
        active_delivery_density_posture_identity
    );
    lifecycle_bundle_projection!(allocation_posture_projection, allocation_posture_identity);
    lifecycle_bundle_projection!(delivery_batch_projection, delivery_batch_identity);
    lifecycle_bundle_projection!(patch_group_projection, patch_group_identity);
    lifecycle_bundle_projection!(delivery_receipt_projection, delivery_receipt_identity);
    lifecycle_bundle_projection!(continuation_projection, continuation_identity);
    lifecycle_bundle_projection!(preview_isolation_projection, preview_isolation_identity);
    lifecycle_bundle_projection!(preview_residue_projection, preview_residue_identity);
    lifecycle_bundle_projection!(policy_projection, policy_identity);
    lifecycle_bundle_projection!(tenant_basis_projection, tenant_basis_identity);
    lifecycle_bundle_projection!(relationship_proof_projection, relationship_proof_identity);
    lifecycle_bundle_projection!(view_shape_projection, view_shape_identity);
    lifecycle_bundle_projection!(basis_posture_projection, basis_posture_identity);
    lifecycle_bundle_projection!(bridge_declaration_projection, bridge_declaration_identity);
    lifecycle_bundle_projection!(signal_strategy_projection, signal_strategy_identity);
    lifecycle_bundle_projection!(counter_snapshot_projection, counter_sequence_identity);
    lifecycle_bundle_projection!(
        subscription_lifecycle_scale_slope_projection,
        subscription_lifecycle_scale_slope_identity
    );
    lifecycle_bundle_projection!(support_matrix_projection, support_matrix_identity);

    pub fn query_scope_projection(
        &self,
    ) -> QueryProjectionIdentity<String, QuerySubscriptionIdentityKind> {
        subscription_evidence_projection(&self.query_scope_identity)
    }
}

impl QuerySubscriptionCertificationError {
    pub fn failure_projection(
        &self,
    ) -> Result<
        QueryProjectionIdentity<String, QuerySubscriptionIdentityKind>,
        IdentityDerivationDenial,
    > {
        Ok(subscription_evidence_projection(self.failure_identity()?))
    }
}

impl SubscriptionLifecycleCertificationError {
    pub fn failure_projection(
        &self,
    ) -> Result<
        QueryProjectionIdentity<String, QuerySubscriptionIdentityKind>,
        IdentityDerivationDenial,
    > {
        Ok(subscription_evidence_projection(self.failure_identity()?))
    }
}
