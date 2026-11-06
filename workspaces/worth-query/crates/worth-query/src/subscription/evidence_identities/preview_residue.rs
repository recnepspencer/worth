use super::SUBSCRIPTION_IDENTITY_SCOPE;
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::evidence_identity::{WorthQueryEvidenceIdentity, WorthQueryEvidenceTag};

pub(in crate::subscription) fn preview_residue_report_identity(
    authoritative_routing_width: u64,
    authoritative_checkpoint_width: u64,
    authoritative_replay_width: u64,
    authoritative_diagnostics_width: u64,
    authoritative_writeback_width: u64,
    temporary_execution_width: u64,
    temporary_diagnostics_width: u64,
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    Ok(
        WorthQueryEvidenceIdentity::compose(SUBSCRIPTION_IDENTITY_SCOPE)
            .field_shape(
                WorthQueryEvidenceTag::new("identity_family"),
                "preview_subscription_residue_report_v1",
            )
            .field_usize(
                WorthQueryEvidenceTag::new("authoritative_routing"),
                authoritative_routing_width as usize,
            )
            .field_usize(
                WorthQueryEvidenceTag::new("authoritative_checkpoint"),
                authoritative_checkpoint_width as usize,
            )
            .field_usize(
                WorthQueryEvidenceTag::new("authoritative_replay"),
                authoritative_replay_width as usize,
            )
            .field_usize(
                WorthQueryEvidenceTag::new("authoritative_diagnostics"),
                authoritative_diagnostics_width as usize,
            )
            .field_usize(
                WorthQueryEvidenceTag::new("authoritative_writeback"),
                authoritative_writeback_width as usize,
            )
            .field_usize(
                WorthQueryEvidenceTag::new("temporary_execution"),
                temporary_execution_width as usize,
            )
            .field_usize(
                WorthQueryEvidenceTag::new("temporary_diagnostics"),
                temporary_diagnostics_width as usize,
            )
            .seal()?,
    )
}
