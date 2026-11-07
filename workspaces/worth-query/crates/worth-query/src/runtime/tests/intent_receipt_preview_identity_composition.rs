use super::support::*;
use crate::{WorthQueryEvidenceIdentity, WorthQueryEvidenceScope, WorthQueryEvidenceTag};

pub(super) fn compose_preview_intent_receipt_identity(
    receipt: &WorthQueryPreviewIntentReceipt,
) -> WorthQueryEvidenceIdentity {
    WorthQueryEvidenceIdentity::compose(WorthQueryEvidenceScope::PreviewIntentReceipt)
        .field_evidence_identity(
            WorthQueryEvidenceTag::new("admission_identity"),
            receipt.admission_identity(),
        )
        .field_shape(
            WorthQueryEvidenceTag::new("posture"),
            "preview-local-staged-no-authoritative-execution",
        )
        .seal()
        .expect("fixture digest must fit")
}

pub(super) fn compose_preview_intent_receipt_inspection_basis_identity(
    receipt: &WorthQueryPreviewIntentReceipt,
) -> WorthQueryEvidenceIdentity {
    WorthQueryEvidenceIdentity::compose(
        WorthQueryEvidenceScope::PreviewIntentReceiptInspectionBasis,
    )
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
    .expect("fixture digest must fit")
}

pub(super) fn compose_preview_intent_receipt_inspection_identity(
    receipt: &WorthQueryPreviewIntentReceipt,
    basis_identity: &WorthQueryEvidenceIdentity,
) -> WorthQueryEvidenceIdentity {
    WorthQueryEvidenceIdentity::compose(WorthQueryEvidenceScope::PreviewIntentReceiptInspection)
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
        .expect("fixture digest must fit")
}

pub(super) fn compose_preview_intent_receipt_inspection_identity_for_inspection(
    inspection: &WorthQueryPreviewIntentReceiptInspection,
) -> WorthQueryEvidenceIdentity {
    WorthQueryEvidenceIdentity::compose(WorthQueryEvidenceScope::PreviewIntentReceiptInspection)
        .field_shape(
            WorthQueryEvidenceTag::new("intent_name"),
            inspection.intent_name(),
        )
        .field_value(
            WorthQueryEvidenceTag::new("strategy_identity"),
            inspection.strategy_identity(),
        )
        .field_shape(
            WorthQueryEvidenceTag::new("strategy_version"),
            inspection.strategy_version(),
        )
        .field_value(
            WorthQueryEvidenceTag::new("canonical_input_digest"),
            inspection.canonical_input_digest(),
        )
        .field_shape(
            WorthQueryEvidenceTag::new("source_lane"),
            inspection.source_lane().as_str(),
        )
        .field_shape(
            WorthQueryEvidenceTag::new("target_lane"),
            inspection.target_lane().as_str(),
        )
        .field_shape(
            WorthQueryEvidenceTag::new("effect_policy"),
            inspection.effect_policy().as_str(),
        )
        .field_evidence_identity(
            WorthQueryEvidenceTag::new("basis_identity"),
            inspection.basis_identity(),
        )
        .field_evidence_identity(
            WorthQueryEvidenceTag::new("admission_identity"),
            inspection.admission_identity(),
        )
        .field_evidence_identity(
            WorthQueryEvidenceTag::new("receipt_identity"),
            inspection.receipt_identity(),
        )
        .seal()
        .expect("fixture digest must fit")
}
