use super::SUBSCRIPTION_IDENTITY_SCOPE;
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::evidence_identity::{WorthQueryEvidenceIdentity, WorthQueryEvidenceTag};

#[allow(clippy::too_many_arguments)]
pub(in crate::subscription) fn lifecycle_certification_bundle_identity(
    base_bundle_identity: &WorthQueryEvidenceIdentity,
    admission_identity: &WorthQueryEvidenceIdentity,
    query_identity: &WorthQueryEvidenceIdentity,
    bridge_identity: &WorthQueryEvidenceIdentity,
    signal_strategy_identity: &WorthQueryEvidenceIdentity,
    query_scope_identity: &WorthQueryEvidenceIdentity,
    subscription_family_identity: &WorthQueryEvidenceIdentity,
    subscription_equivalence_identity: &WorthQueryEvidenceIdentity,
    policy_identity: &WorthQueryEvidenceIdentity,
    tenant_basis_identity: &WorthQueryEvidenceIdentity,
    relationship_proof_identity: &WorthQueryEvidenceIdentity,
    view_shape_identity: &WorthQueryEvidenceIdentity,
    basis_posture_identity: &WorthQueryEvidenceIdentity,
    active_lane_identity: &WorthQueryEvidenceIdentity,
    active_lane_handle_identity: &WorthQueryEvidenceIdentity,
    performance_sequence_identity: &WorthQueryEvidenceIdentity,
    attachment_identity: &WorthQueryEvidenceIdentity,
    delivery_window_identity: &WorthQueryEvidenceIdentity,
    maintenance_delta_identity: &WorthQueryEvidenceIdentity,
    work_packet_identity: &WorthQueryEvidenceIdentity,
    delivery_batch_identity: &WorthQueryEvidenceIdentity,
    delivery_receipt_identity: &WorthQueryEvidenceIdentity,
    continuation_identity: &WorthQueryEvidenceIdentity,
    closeout_identity: &WorthQueryEvidenceIdentity,
    support_matrix_identity: &WorthQueryEvidenceIdentity,
    counter_sequence_identity: &WorthQueryEvidenceIdentity,
) -> WorthQueryEvidenceIdentity {
    WorthQueryEvidenceIdentity::compose(SUBSCRIPTION_IDENTITY_SCOPE)
            .field_shape(
                WorthQueryEvidenceTag::new("identity_family"),
                "subscription_lifecycle_certification_bundle_v1",
            )
            .field_evidence_identity(WorthQueryEvidenceTag::new("base"), base_bundle_identity)
            .field_evidence_identity(WorthQueryEvidenceTag::new("admission"), admission_identity)
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("query_declaration"),
                query_identity,
            )
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("bridge_declaration"),
                bridge_identity,
            )
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("signal_strategy"),
                signal_strategy_identity,
            )
            .field_evidence_identity(WorthQueryEvidenceTag::new("query"), query_scope_identity)
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("subscription_family"),
                subscription_family_identity,
            )
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("subscription_equivalence"),
                subscription_equivalence_identity,
            )
            .field_evidence_identity(WorthQueryEvidenceTag::new("policy"), policy_identity)
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("tenant_basis"),
                tenant_basis_identity,
            )
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("relationship_proof"),
                relationship_proof_identity,
            )
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("view_shape"),
                view_shape_identity,
            )
            .field_evidence_identity(WorthQueryEvidenceTag::new("basis"), basis_posture_identity)
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("active_lane"),
                active_lane_identity,
            )
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("active_lane_handle"),
                active_lane_handle_identity,
            )
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("performance"),
                performance_sequence_identity,
            )
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("attachment"),
                attachment_identity,
            )
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("delivery_window"),
                delivery_window_identity,
            )
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("maintenance_delta"),
                maintenance_delta_identity,
            )
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("work_packet"),
                work_packet_identity,
            )
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("delivery_batch"),
                delivery_batch_identity,
            )
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("delivery_receipt"),
                delivery_receipt_identity,
            )
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("continuation"),
                continuation_identity,
            )
            .field_evidence_identity(WorthQueryEvidenceTag::new("closeout"), closeout_identity)
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("support"),
                support_matrix_identity,
            )
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("counters"),
                counter_sequence_identity,
            )
            .seal().expect("fixed digest grammar: literal tags and values, primitive values, or sealed identity tokens")
}

pub(in crate::subscription) fn lifecycle_active_lane_lookup_class_identity(
    lookup_class: &str,
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    Ok(
        WorthQueryEvidenceIdentity::compose(SUBSCRIPTION_IDENTITY_SCOPE)
            .field_shape(
                WorthQueryEvidenceTag::new("identity_family"),
                "subscription_active_lane_lookup_class_v1",
            )
            .field_shape(WorthQueryEvidenceTag::new("lookup_class"), lookup_class)
            .seal()?,
    )
}

#[allow(clippy::too_many_arguments)]
pub(in crate::subscription) fn lifecycle_subscription_budget_identity(
    registry_lookup_width: u64,
    fanout_width: u64,
    allocation_scope_width: u64,
    lookup_class: &str,
    allocation_posture: &str,
    durable_checkpoint_requested: bool,
    store_backed_restart_requested: bool,
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    Ok(
        WorthQueryEvidenceIdentity::compose(SUBSCRIPTION_IDENTITY_SCOPE)
            .field_shape(
                WorthQueryEvidenceTag::new("identity_family"),
                "active_subscription_budget_v1",
            )
            .field_usize(
                WorthQueryEvidenceTag::new("lookup_width"),
                registry_lookup_width as usize,
            )
            .field_usize(
                WorthQueryEvidenceTag::new("fanout_width"),
                fanout_width as usize,
            )
            .field_usize(
                WorthQueryEvidenceTag::new("allocation_scope_width"),
                allocation_scope_width as usize,
            )
            .field_shape(WorthQueryEvidenceTag::new("lookup_class"), lookup_class)
            .field_shape(
                WorthQueryEvidenceTag::new("allocation_posture"),
                allocation_posture,
            )
            .field_shape(
                WorthQueryEvidenceTag::new("durable_checkpoint_requested"),
                if durable_checkpoint_requested {
                    "true"
                } else {
                    "false"
                },
            )
            .field_shape(
                WorthQueryEvidenceTag::new("store_backed_restart_requested"),
                if store_backed_restart_requested {
                    "true"
                } else {
                    "false"
                },
            )
            .seal()?,
    )
}

pub(in crate::subscription) fn lifecycle_allocation_posture_identity(
    posture: &str,
    allocation_scope_width: u64,
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    Ok(
        WorthQueryEvidenceIdentity::compose(SUBSCRIPTION_IDENTITY_SCOPE)
            .field_shape(
                WorthQueryEvidenceTag::new("identity_family"),
                "subscription_allocation_posture_v1",
            )
            .field_shape(WorthQueryEvidenceTag::new("posture"), posture)
            .field_usize(
                WorthQueryEvidenceTag::new("allocation_scope_width"),
                allocation_scope_width as usize,
            )
            .seal()?,
    )
}

pub(in crate::subscription) fn lifecycle_active_delivery_density_posture_identity(
    posture: &str,
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    Ok(
        WorthQueryEvidenceIdentity::compose(SUBSCRIPTION_IDENTITY_SCOPE)
            .field_shape(
                WorthQueryEvidenceTag::new("identity_family"),
                "subscription_active_delivery_density_posture_v1",
            )
            .field_shape(WorthQueryEvidenceTag::new("posture"), posture)
            .seal()?,
    )
}

pub(in crate::subscription) fn lifecycle_context_view_shape_identity(
    view_family: Option<&str>,
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    Ok(
        WorthQueryEvidenceIdentity::compose(SUBSCRIPTION_IDENTITY_SCOPE)
            .field_shape(
                WorthQueryEvidenceTag::new("identity_family"),
                "subscription_lifecycle_view_shape_v1",
            )
            .field_shape(
                WorthQueryEvidenceTag::new("view"),
                view_family.unwrap_or("none"),
            )
            .seal()?,
    )
}

pub(in crate::subscription) fn lifecycle_context_basis_posture_identity(
    basis: &str,
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    Ok(
        WorthQueryEvidenceIdentity::compose(SUBSCRIPTION_IDENTITY_SCOPE)
            .field_shape(
                WorthQueryEvidenceTag::new("identity_family"),
                "subscription_lifecycle_basis_v1",
            )
            .field_shape(WorthQueryEvidenceTag::new("basis"), basis)
            .seal()?,
    )
}

pub(in crate::subscription) fn lifecycle_preview_promotion_residue_identity(
    residue_identity: &WorthQueryEvidenceIdentity,
    handoff_identity: &WorthQueryEvidenceIdentity,
    authoritative_lane_identity: &WorthQueryEvidenceIdentity,
) -> WorthQueryEvidenceIdentity {
    WorthQueryEvidenceIdentity::compose(SUBSCRIPTION_IDENTITY_SCOPE)
            .field_shape(
                WorthQueryEvidenceTag::new("identity_family"),
                "subscription_preview_residue_v1",
            )
            .field_evidence_identity(WorthQueryEvidenceTag::new("residue"), residue_identity)
            .field_evidence_identity(WorthQueryEvidenceTag::new("handoff"), handoff_identity)
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("authoritative_lane"),
                authoritative_lane_identity,
            )
            .seal().expect("fixed digest grammar: literal tags and values, primitive values, or sealed identity tokens")
}
