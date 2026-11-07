use super::*;

pub(in crate::runtime::tests) fn compose_effect_intent_receipt_identity(
    receipt: &WorthQueryEffectIntentReceipt,
) -> WorthQueryEvidenceIdentity {
    WorthQueryEvidenceIdentity::compose(WorthQueryEvidenceScope::EffectIntentReceipt)
        .field_shape(
            WorthQueryEvidenceTag::new("effect_name"),
            receipt.effect_name(),
        )
        .field_evidence_identity(
            WorthQueryEvidenceTag::new("trigger_commit_evidence_identity"),
            receipt.trigger_commit_evidence_identity(),
        )
        .field_shape(
            WorthQueryEvidenceTag::new("trigger_source_kind"),
            receipt.trigger_source_kind().as_str(),
        )
        .field_evidence_identity(
            WorthQueryEvidenceTag::new("write_adjacent_trigger"),
            receipt.write_adjacent_trigger().identity(),
        )
        .field_value(
            WorthQueryEvidenceTag::new("pending_intent_target"),
            receipt.pending_intent_target(),
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
        .field_value_sequence(
            WorthQueryEvidenceTag::new("phase"),
            receipt
                .phase_evidence()
                .phases()
                .iter()
                .map(|phase| phase.as_str()),
        )
        .field_shape(
            WorthQueryEvidenceTag::new("loop_prevention"),
            receipt.phase_evidence().loop_prevention().as_str(),
        )
        .field_shape(
            WorthQueryEvidenceTag::new("idempotence"),
            receipt.phase_evidence().idempotence().as_str(),
        )
        .field_evidence_identity(
            WorthQueryEvidenceTag::new("intent_receipt_identity"),
            receipt.intent_receipt().receipt_identity(),
        )
        .seal()
        .expect("fixture digest must fit")
}
