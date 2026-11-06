use super::SUBSCRIPTION_IDENTITY_SCOPE;
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::evidence_identity::{WorthQueryEvidenceIdentity, WorthQueryEvidenceTag};

#[cfg(test)]
pub(in crate::subscription) fn preview_epoch_identity(
    epoch: &str,
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    Ok(
        WorthQueryEvidenceIdentity::compose(SUBSCRIPTION_IDENTITY_SCOPE)
            .field_shape(
                WorthQueryEvidenceTag::new("identity_family"),
                "preview_subscription_epoch_v1",
            )
            .field_shape(WorthQueryEvidenceTag::new("epoch"), epoch)
            .seal()?,
    )
}

#[allow(clippy::too_many_arguments)]
#[cfg(test)]
pub(in crate::subscription) fn preview_isolation_identity(
    lane_identity: &WorthQueryEvidenceIdentity,
    attachment_identity: &WorthQueryEvidenceIdentity,
    future_selection_identity: &WorthQueryEvidenceIdentity,
    basis_binding_identity: &WorthQueryEvidenceIdentity,
    checkpoint_identity: &WorthQueryEvidenceIdentity,
    preview_epoch_identity: &WorthQueryEvidenceIdentity,
    lifecycle_state: &str,
    preview_residue_budget_width: u64,
    counters_identity: &WorthQueryEvidenceIdentity,
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    Ok(
        WorthQueryEvidenceIdentity::compose(SUBSCRIPTION_IDENTITY_SCOPE)
            .field_shape(
                WorthQueryEvidenceTag::new("identity_family"),
                "preview_subscription_isolation_artifact_v1",
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
            .field_evidence_identity(WorthQueryEvidenceTag::new("epoch"), preview_epoch_identity)
            .field_shape(WorthQueryEvidenceTag::new("state"), lifecycle_state)
            .field_usize(
                WorthQueryEvidenceTag::new("residue_budget"),
                preview_residue_budget_width as usize,
            )
            .field_evidence_identity(WorthQueryEvidenceTag::new("counters"), counters_identity)
            .seal()?,
    )
}

pub(in crate::subscription) fn preview_authoritative_sharing_denial_identity(
    isolation_identity: &WorthQueryEvidenceIdentity,
    authoritative_lane_identity: &WorthQueryEvidenceIdentity,
    preview_basis_binding_identity: &WorthQueryEvidenceIdentity,
    authoritative_basis_binding_identity: &WorthQueryEvidenceIdentity,
    preview_checkpoint_identity: &WorthQueryEvidenceIdentity,
    authoritative_checkpoint_identity: &WorthQueryEvidenceIdentity,
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    Ok(
        WorthQueryEvidenceIdentity::compose(SUBSCRIPTION_IDENTITY_SCOPE)
            .field_shape(
                WorthQueryEvidenceTag::new("identity_family"),
                "preview_authoritative_sharing_denial_v1",
            )
            .field_evidence_identity(WorthQueryEvidenceTag::new("preview"), isolation_identity)
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("authoritative"),
                authoritative_lane_identity,
            )
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("preview_basis"),
                preview_basis_binding_identity,
            )
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("authoritative_basis"),
                authoritative_basis_binding_identity,
            )
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("preview_checkpoint"),
                preview_checkpoint_identity,
            )
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("authoritative_checkpoint"),
                authoritative_checkpoint_identity,
            )
            .seal()?,
    )
}

#[allow(clippy::too_many_arguments)]
pub(in crate::subscription) fn preview_discard_closeout_identity(
    lane_identity: &WorthQueryEvidenceIdentity,
    attachment_identity: &WorthQueryEvidenceIdentity,
    future_selection_identity: &WorthQueryEvidenceIdentity,
    basis_binding_identity: &WorthQueryEvidenceIdentity,
    checkpoint_identity: &WorthQueryEvidenceIdentity,
    preview_epoch_identity: &WorthQueryEvidenceIdentity,
    isolation_identity: &WorthQueryEvidenceIdentity,
    residue_report_identity: &WorthQueryEvidenceIdentity,
    performance_receipt_identity: &WorthQueryEvidenceIdentity,
    lifecycle_state: &str,
    counters_identity: &WorthQueryEvidenceIdentity,
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    Ok(
        WorthQueryEvidenceIdentity::compose(SUBSCRIPTION_IDENTITY_SCOPE)
            .field_shape(
                WorthQueryEvidenceTag::new("identity_family"),
                "preview_subscription_discard_closeout_v1",
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
            .field_evidence_identity(WorthQueryEvidenceTag::new("epoch"), preview_epoch_identity)
            .field_evidence_identity(WorthQueryEvidenceTag::new("isolation"), isolation_identity)
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("residue_report"),
                residue_report_identity,
            )
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("performance"),
                performance_receipt_identity,
            )
            .field_shape(WorthQueryEvidenceTag::new("state"), lifecycle_state)
            .field_evidence_identity(WorthQueryEvidenceTag::new("counters"), counters_identity)
            .seal()?,
    )
}

pub(in crate::subscription) fn preview_promotion_authority_identity(
    authority_label: &str,
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    Ok(
        WorthQueryEvidenceIdentity::compose(SUBSCRIPTION_IDENTITY_SCOPE)
            .field_shape(
                WorthQueryEvidenceTag::new("identity_family"),
                "preview_subscription_promotion_authority_v1",
            )
            .field_shape(WorthQueryEvidenceTag::new("authority"), authority_label)
            .seal()?,
    )
}

pub(in crate::subscription) fn preview_promotion_rebinding_identity(
    preview_basis_binding_identity: &WorthQueryEvidenceIdentity,
    authoritative_basis_binding_identity: &WorthQueryEvidenceIdentity,
    preview_checkpoint_identity: &WorthQueryEvidenceIdentity,
    authoritative_checkpoint_identity: &WorthQueryEvidenceIdentity,
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    Ok(
        WorthQueryEvidenceIdentity::compose(SUBSCRIPTION_IDENTITY_SCOPE)
            .field_shape(
                WorthQueryEvidenceTag::new("identity_family"),
                "preview_subscription_promotion_rebinding_v1",
            )
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("preview_basis"),
                preview_basis_binding_identity,
            )
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("authoritative_basis"),
                authoritative_basis_binding_identity,
            )
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("preview_checkpoint"),
                preview_checkpoint_identity,
            )
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("authoritative_checkpoint"),
                authoritative_checkpoint_identity,
            )
            .seal()?,
    )
}

#[allow(clippy::too_many_arguments)]
pub(in crate::subscription) fn preview_promotion_handoff_identity(
    preview_lane_identity: &WorthQueryEvidenceIdentity,
    authoritative_lane_identity: &WorthQueryEvidenceIdentity,
    attachment_identity: &WorthQueryEvidenceIdentity,
    future_selection_identity: &WorthQueryEvidenceIdentity,
    basis_binding_identity: &WorthQueryEvidenceIdentity,
    preview_checkpoint_identity: &WorthQueryEvidenceIdentity,
    authoritative_checkpoint_identity: &WorthQueryEvidenceIdentity,
    preview_epoch_identity: &WorthQueryEvidenceIdentity,
    isolation_identity: &WorthQueryEvidenceIdentity,
    residue_report_identity: &WorthQueryEvidenceIdentity,
    authority_identity: &WorthQueryEvidenceIdentity,
    rebinding_identity: &WorthQueryEvidenceIdentity,
    performance_receipt_identity: &WorthQueryEvidenceIdentity,
    lifecycle_state: &str,
    counters_identity: &WorthQueryEvidenceIdentity,
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    Ok(
        WorthQueryEvidenceIdentity::compose(SUBSCRIPTION_IDENTITY_SCOPE)
            .field_shape(
                WorthQueryEvidenceTag::new("identity_family"),
                "preview_subscription_promotion_handoff_v1",
            )
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("preview_lane"),
                preview_lane_identity,
            )
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("authoritative_lane"),
                authoritative_lane_identity,
            )
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
                WorthQueryEvidenceTag::new("preview_checkpoint"),
                preview_checkpoint_identity,
            )
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("authoritative_checkpoint"),
                authoritative_checkpoint_identity,
            )
            .field_evidence_identity(WorthQueryEvidenceTag::new("epoch"), preview_epoch_identity)
            .field_evidence_identity(WorthQueryEvidenceTag::new("isolation"), isolation_identity)
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("residue_report"),
                residue_report_identity,
            )
            .field_evidence_identity(WorthQueryEvidenceTag::new("authority"), authority_identity)
            .field_evidence_identity(WorthQueryEvidenceTag::new("rebinding"), rebinding_identity)
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("performance"),
                performance_receipt_identity,
            )
            .field_shape(WorthQueryEvidenceTag::new("state"), lifecycle_state)
            .field_evidence_identity(WorthQueryEvidenceTag::new("counters"), counters_identity)
            .seal()?,
    )
}
