use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::runtime::WorthQueryWriteReceipt;
use crate::{WorthQueryEvidenceIdentity, WorthQueryEvidenceScope, WorthQueryEvidenceTag};

pub(super) fn write_receipt_digest(
    receipt: &WorthQueryWriteReceipt,
) -> Result<String, IdentityDerivationDenial> {
    Ok(
        WorthQueryEvidenceIdentity::compose(
            WorthQueryEvidenceScope::WriteReceiptInspectionArtifact,
        )
        .field_evidence_identity(
            WorthQueryEvidenceTag::new("commit_evidence_identity"),
            receipt.commit_evidence_identity(),
        )
        .field_evidence_identity(
            WorthQueryEvidenceTag::new("snapshot_evidence_identity"),
            receipt.snapshot_evidence_identity(),
        )
        .field_shape(
            WorthQueryEvidenceTag::new("mutation_family"),
            receipt.mutation_family().as_str(),
        )
        .field_shape(
            WorthQueryEvidenceTag::new("authority_lane"),
            receipt.authority_lane().as_str(),
        )
        .seal()?
        .terminal_projection_for_reporting()
        .to_string(),
    )
}
