use crate::evidence_identity::{
    worth_query_evidence_identity, WorthQueryEvidenceIdentity, WorthQueryEvidenceScope,
    WorthQueryEvidenceTag,
};

use super::{
    WorthQueryEffectAdmission, WorthQueryEffectPolicy, WorthQueryIntentDeclaration,
    WorthQueryPreviewBasisAdmission,
};

pub(super) fn preview_intent_admission_identity(
    declaration: &WorthQueryIntentDeclaration,
    effect_policy: WorthQueryEffectPolicy,
    basis_admission: &WorthQueryPreviewBasisAdmission,
    admission: WorthQueryEffectAdmission,
    canonical_input_digest: &str,
) -> Result<WorthQueryEvidenceIdentity, crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial> {
    Ok(worth_query_evidence_identity(WorthQueryEvidenceScope::PreviewIntentAdmission)
        .field_shape(
            WorthQueryEvidenceTag::new("intent_name"),
            declaration.name(),
        )
        .field_shape(
            WorthQueryEvidenceTag::new("strategy_identity"),
            declaration.strategy_name(),
        )
        .field_shape(
            WorthQueryEvidenceTag::new("strategy_version"),
            declaration.strategy_version(),
        )
        .field_value(
            WorthQueryEvidenceTag::new("canonical_input_digest"),
            canonical_input_digest,
        )
        .field_shape(
            WorthQueryEvidenceTag::new("source_lane"),
            declaration.source_lane().as_str(),
        )
        .field_shape(
            WorthQueryEvidenceTag::new("target_lane"),
            declaration.target_lane().as_str(),
        )
        .field_shape(
            WorthQueryEvidenceTag::new("effect_policy"),
            effect_policy.as_str(),
        )
        .field_shape(
            WorthQueryEvidenceTag::new("admitted_action"),
            admission.action().as_str(),
        )
        .field_shape(
            WorthQueryEvidenceTag::new("admitted_lane"),
            admission.target_lane().as_str(),
        )
        .field_evidence_identity(
            WorthQueryEvidenceTag::new("basis_admission_identity"),
            basis_admission.admission_identity(),
        )
        .seal()?)
}

pub(super) fn preview_intent_receipt_identity(
    admission_identity: &WorthQueryEvidenceIdentity,
) -> Result<WorthQueryEvidenceIdentity, crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial> {
    Ok(worth_query_evidence_identity(WorthQueryEvidenceScope::PreviewIntentReceipt)
        .field_evidence_identity(
            WorthQueryEvidenceTag::new("admission_identity"),
            admission_identity,
        )
        .field_shape(
            WorthQueryEvidenceTag::new("posture"),
            "preview-local-staged-no-authoritative-execution",
        )
        .seal()?)
}
