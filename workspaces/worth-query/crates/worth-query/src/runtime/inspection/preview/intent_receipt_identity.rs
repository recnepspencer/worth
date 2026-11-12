use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::evidence_identity::{
    worth_query_evidence_identity, WorthQueryEvidenceIdentity, WorthQueryEvidenceScope,
    WorthQueryEvidenceTag,
};
use crate::runtime::WorthQueryPreviewIntentReceipt;

pub(super) fn preview_intent_receipt_inspection_basis_identity(
    receipt: &WorthQueryPreviewIntentReceipt,
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    worth_query_evidence_identity(WorthQueryEvidenceScope::PreviewIntentReceiptInspectionBasis)
        .field_shape(
            WorthQueryEvidenceTag::new("intent_name"),
            receipt.intent_name(),
        )
        .field_usize(
            WorthQueryEvidenceTag::new("basis_evidence_count"),
            receipt.basis_evidence().len(),
        )
        .field_evidence_identity(
            WorthQueryEvidenceTag::new("basis_evidence"),
            receipt.basis_evidence_identity(),
        )
        .seal()
}

pub(super) fn preview_intent_receipt_inspection_identity(
    receipt: &WorthQueryPreviewIntentReceipt,
    basis_identity: &WorthQueryEvidenceIdentity,
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    worth_query_evidence_identity(WorthQueryEvidenceScope::PreviewIntentReceiptInspection)
        .field_shape(
            WorthQueryEvidenceTag::new("intent_name"),
            receipt.intent_name(),
        )
        .field_value(
            WorthQueryEvidenceTag::new("strategy_identity"),
            receipt.strategy_identity(),
        )
        .field_shape(
            WorthQueryEvidenceTag::new("strategy_version"),
            receipt.strategy_version(),
        )
        .field_value(
            WorthQueryEvidenceTag::new("canonical_input_digest"),
            receipt.canonical_input_digest(),
        )
        .field_shape(
            WorthQueryEvidenceTag::new("source_lane"),
            receipt.source_lane().as_str(),
        )
        .field_shape(
            WorthQueryEvidenceTag::new("target_lane"),
            receipt.target_lane().as_str(),
        )
        .field_shape(
            WorthQueryEvidenceTag::new("effect_policy"),
            receipt.effect_policy().as_str(),
        )
        .field_evidence_identity(WorthQueryEvidenceTag::new("basis_identity"), basis_identity)
        .field_evidence_identity(
            WorthQueryEvidenceTag::new("admission_identity"),
            receipt.admission_identity(),
        )
        .field_evidence_identity(
            WorthQueryEvidenceTag::new("receipt_identity"),
            receipt.receipt_identity(),
        )
        .seal()
}
