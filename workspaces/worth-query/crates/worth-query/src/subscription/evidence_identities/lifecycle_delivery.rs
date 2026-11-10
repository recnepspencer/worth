use super::super::active_budget::ActiveSubscriptionAllocationPosture;
use super::SUBSCRIPTION_IDENTITY_SCOPE;
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::evidence_identity::{WorthQueryEvidenceIdentity, WorthQueryEvidenceTag};

pub(in crate::subscription) fn lifecycle_work_packet_identity(
    lane_identity: &WorthQueryEvidenceIdentity,
    attachment_identity: &WorthQueryEvidenceIdentity,
    maintenance_delta_identity: &WorthQueryEvidenceIdentity,
    lowering_report_identity: &WorthQueryEvidenceIdentity,
    density_posture: &str,
    affected_lane_width: u64,
    affected_attachment_width: u64,
    patch_group_width: u64,
    continuation_width: u64,
    preview_residue_width: u64,
    allocation_scope_width: u64,
    allocation_posture: ActiveSubscriptionAllocationPosture,
    performance_receipt_identity: &WorthQueryEvidenceIdentity,
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    WorthQueryEvidenceIdentity::compose(SUBSCRIPTION_IDENTITY_SCOPE)
        .field_shape(
            WorthQueryEvidenceTag::new("identity_family"),
            "active_delivery_work_packet_v1",
        )
        .field_evidence_identity(WorthQueryEvidenceTag::new("lane"), lane_identity)
        .field_evidence_identity(
            WorthQueryEvidenceTag::new("attachment"),
            attachment_identity,
        )
        .field_evidence_identity(
            WorthQueryEvidenceTag::new("maintenance_delta"),
            maintenance_delta_identity,
        )
        .field_evidence_identity(
            WorthQueryEvidenceTag::new("lowering_report"),
            lowering_report_identity,
        )
        .field_shape(WorthQueryEvidenceTag::new("density"), density_posture)
        .field_usize(
            WorthQueryEvidenceTag::new("affected_lane_width"),
            affected_lane_width as usize,
        )
        .field_usize(
            WorthQueryEvidenceTag::new("affected_attachment_width"),
            affected_attachment_width as usize,
        )
        .field_usize(
            WorthQueryEvidenceTag::new("patch_group_width"),
            patch_group_width as usize,
        )
        .field_usize(
            WorthQueryEvidenceTag::new("continuation_width"),
            continuation_width as usize,
        )
        .field_usize(
            WorthQueryEvidenceTag::new("preview_residue_width"),
            preview_residue_width as usize,
        )
        .field_usize(
            WorthQueryEvidenceTag::new("allocation_scope_width"),
            allocation_scope_width as usize,
        )
        .field_shape(
            WorthQueryEvidenceTag::new("allocation_posture"),
            allocation_posture.as_str(),
        )
        .field_evidence_identity(
            WorthQueryEvidenceTag::new("performance"),
            performance_receipt_identity,
        )
        .seal()
}

pub(in crate::subscription) fn lifecycle_delivery_batch_receipt_identity(
    attachment_identity: &WorthQueryEvidenceIdentity,
    sequence: u64,
) -> WorthQueryEvidenceIdentity {
    WorthQueryEvidenceIdentity::compose(SUBSCRIPTION_IDENTITY_SCOPE)
            .field_shape(
                WorthQueryEvidenceTag::new("identity_family"),
                "query_delivery_batch_receipt_v1",
            )
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("attachment"),
                attachment_identity,
            )
            .field_usize(WorthQueryEvidenceTag::new("sequence"), sequence as usize)
            .seal().expect("fixed digest grammar: literal tags and values, primitive values, or sealed identity tokens")
}

pub(in crate::subscription) fn lifecycle_delivery_batch_identity(
    delivery_window_identity: &WorthQueryEvidenceIdentity,
    work_packet_identity: &WorthQueryEvidenceIdentity,
    delivery_cause_identity: &WorthQueryEvidenceIdentity,
    has_relational_patch: bool,
    patch_group_identity: &WorthQueryEvidenceIdentity,
    receipt_identity: &WorthQueryEvidenceIdentity,
    performance_receipt_identity: &WorthQueryEvidenceIdentity,
    delivery_posture: &str,
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    WorthQueryEvidenceIdentity::compose(SUBSCRIPTION_IDENTITY_SCOPE)
        .field_shape(
            WorthQueryEvidenceTag::new("identity_family"),
            "query_delivery_batch_v1",
        )
        .field_evidence_identity(
            WorthQueryEvidenceTag::new("delivery_window"),
            delivery_window_identity,
        )
        .field_evidence_identity(
            WorthQueryEvidenceTag::new("work_packet"),
            work_packet_identity,
        )
        .field_evidence_identity(
            WorthQueryEvidenceTag::new("delivery_cause"),
            delivery_cause_identity,
        )
        .field_shape(
            WorthQueryEvidenceTag::new("relational_patch"),
            if has_relational_patch {
                "true"
            } else {
                "false"
            },
        )
        .field_evidence_identity(
            WorthQueryEvidenceTag::new("patch_group"),
            patch_group_identity,
        )
        .field_evidence_identity(WorthQueryEvidenceTag::new("receipt"), receipt_identity)
        .field_evidence_identity(
            WorthQueryEvidenceTag::new("performance"),
            performance_receipt_identity,
        )
        .field_shape(WorthQueryEvidenceTag::new("posture"), delivery_posture)
        .seal()
}

#[cfg(test)]
pub(in crate::subscription) fn lifecycle_continuation_identity(
    lane_identity: &WorthQueryEvidenceIdentity,
    continuation_class: &str,
    source_identity: &WorthQueryEvidenceIdentity,
    target_identity: &WorthQueryEvidenceIdentity,
    future_selection_identity: &WorthQueryEvidenceIdentity,
    basis_identity: &WorthQueryEvidenceIdentity,
    checkpoint_identity: &WorthQueryEvidenceIdentity,
    authority_identity: &WorthQueryEvidenceIdentity,
    remap_width: u64,
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    WorthQueryEvidenceIdentity::compose(SUBSCRIPTION_IDENTITY_SCOPE)
        .field_shape(
            WorthQueryEvidenceTag::new("identity_family"),
            "subscription_continuation_evidence_v1",
        )
        .field_evidence_identity(WorthQueryEvidenceTag::new("lane"), lane_identity)
        .field_shape(WorthQueryEvidenceTag::new("class"), continuation_class)
        .field_evidence_identity(WorthQueryEvidenceTag::new("source"), source_identity)
        .field_evidence_identity(WorthQueryEvidenceTag::new("target"), target_identity)
        .field_evidence_identity(
            WorthQueryEvidenceTag::new("future_selection"),
            future_selection_identity,
        )
        .field_evidence_identity(WorthQueryEvidenceTag::new("basis"), basis_identity)
        .field_evidence_identity(
            WorthQueryEvidenceTag::new("checkpoint"),
            checkpoint_identity,
        )
        .field_evidence_identity(WorthQueryEvidenceTag::new("authority"), authority_identity)
        .field_usize(
            WorthQueryEvidenceTag::new("remap_width"),
            remap_width as usize,
        )
        .seal()
}

pub(in crate::subscription) fn lifecycle_closeout_identity(
    lane_identity: &WorthQueryEvidenceIdentity,
    attachment_identity: &WorthQueryEvidenceIdentity,
    future_selection_identity: &WorthQueryEvidenceIdentity,
    basis_binding_identity: &WorthQueryEvidenceIdentity,
    checkpoint_identity: &WorthQueryEvidenceIdentity,
    closeout_kind: &str,
    lane_terminal: bool,
    support_identity: &WorthQueryEvidenceIdentity,
    performance_receipt_identity: &WorthQueryEvidenceIdentity,
    counters_identity: &WorthQueryEvidenceIdentity,
    source_identity: &WorthQueryEvidenceIdentity,
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    WorthQueryEvidenceIdentity::compose(SUBSCRIPTION_IDENTITY_SCOPE)
        .field_shape(
            WorthQueryEvidenceTag::new("identity_family"),
            "subscription_lifecycle_closeout_v1",
        )
        .field_evidence_identity(WorthQueryEvidenceTag::new("lane"), lane_identity)
        .field_evidence_identity(
            WorthQueryEvidenceTag::new("attachment"),
            attachment_identity,
        )
        .field_evidence_identity(
            WorthQueryEvidenceTag::new("future_selection"),
            future_selection_identity,
        )
        .field_evidence_identity(WorthQueryEvidenceTag::new("basis"), basis_binding_identity)
        .field_evidence_identity(
            WorthQueryEvidenceTag::new("checkpoint"),
            checkpoint_identity,
        )
        .field_shape(WorthQueryEvidenceTag::new("kind"), closeout_kind)
        .field_shape(
            WorthQueryEvidenceTag::new("lane_terminal"),
            if lane_terminal { "true" } else { "false" },
        )
        .field_evidence_identity(WorthQueryEvidenceTag::new("support"), support_identity)
        .field_evidence_identity(
            WorthQueryEvidenceTag::new("performance"),
            performance_receipt_identity,
        )
        .field_evidence_identity(WorthQueryEvidenceTag::new("counters"), counters_identity)
        .field_evidence_identity(WorthQueryEvidenceTag::new("source"), source_identity)
        .seal()
}
