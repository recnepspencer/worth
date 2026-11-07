use super::*;

pub(in crate::runtime::tests) fn compose_authoritative_intent_receipt_identity(
    receipt: &WorthQueryIntentReceipt,
) -> WorthQueryEvidenceIdentity {
    compose_authoritative_intent_receipt_identity_with_effect_trigger(receipt, None)
}

pub(in crate::runtime::tests) fn compose_authoritative_effect_triggered_intent_receipt_identity(
    receipt: &WorthQueryEffectIntentReceipt,
) -> WorthQueryEvidenceIdentity {
    compose_authoritative_intent_receipt_identity_with_effect_trigger(
        receipt.intent_receipt(),
        Some(receipt.write_adjacent_trigger().digest()),
    )
}

fn compose_authoritative_intent_receipt_identity_with_effect_trigger(
    receipt: &WorthQueryIntentReceipt,
    effect_trigger_digest: Option<&str>,
) -> WorthQueryEvidenceIdentity {
    let invariant_evidence_identities = receipt_value_identities(
        "authoritative-receipt-invariant-evidence",
        receipt.invariant_evidence(),
    );
    let effect_trigger_identity = effect_trigger_digest.map(|digest| {
        worth_query_evidence_identity(WorthQueryEvidenceScope::AuthoritativeIntentReceipt)
            .field_shape(
                WorthQueryEvidenceTag::new("role"),
                "authoritative-receipt-effect-trigger",
            )
            .field_value(WorthQueryEvidenceTag::new("digest"), digest)
            .seal()
    });
    let affected_live_view_identities = receipt_value_identities(
        "authoritative-receipt-affected-live-view",
        &receipt.terminal_affected_live_view_ids_projection(),
    );
    let affected_derived_view_identities = receipt_value_identities(
        "authoritative-receipt-affected-derived-view",
        &receipt.terminal_affected_derived_view_ids_projection(),
    );

    worth_query_evidence_identity(WorthQueryEvidenceScope::AuthoritativeIntentReceipt)
        .field_shape(
            WorthQueryEvidenceTag::new("intent_name"),
            receipt.intent_name(),
        )
        .field_shape(
            WorthQueryEvidenceTag::new("execution_kind"),
            receipt.execution_kind().as_str(),
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
            WorthQueryEvidenceTag::new("strategy_descriptor_digest"),
            receipt.strategy_descriptor_digest(),
        )
        .field_value(
            WorthQueryEvidenceTag::new("canonical_input_digest"),
            receipt.canonical_input_digest(),
        )
        .field_value(
            WorthQueryEvidenceTag::new("outcome_digest"),
            receipt.outcome_digest(),
        )
        .field_evidence_identity_sequence(
            WorthQueryEvidenceTag::new("invariant_evidence"),
            invariant_evidence_identities.iter(),
        )
        .field_shape(
            WorthQueryEvidenceTag::new("source_lane"),
            receipt.source_lane().as_str(),
        )
        .field_shape(
            WorthQueryEvidenceTag::new("target_lane"),
            receipt.target_lane().as_str(),
        )
        .optional_evidence_identity(
            WorthQueryEvidenceTag::new("effect_trigger_digest"),
            effect_trigger_identity
                .as_ref()
                .map(|value| value.as_ref().expect("fixture digest must fit")),
        )
        .field_evidence_identity(
            WorthQueryEvidenceTag::new("commit_evidence_identity"),
            receipt.commit_evidence_identity(),
        )
        .field_evidence_identity(
            WorthQueryEvidenceTag::new("snapshot_evidence_identity"),
            receipt.snapshot_evidence_identity(),
        )
        .field_evidence_identity_sequence(
            WorthQueryEvidenceTag::new("affected_live_view_id"),
            affected_live_view_identities.iter(),
        )
        .field_evidence_identity_sequence(
            WorthQueryEvidenceTag::new("affected_derived_view_id"),
            affected_derived_view_identities.iter(),
        )
        .field_usize(
            WorthQueryEvidenceTag::new("considered_computed_view_count"),
            receipt.considered_computed_view_count(),
        )
        .field_usize(
            WorthQueryEvidenceTag::new("considered_effect_count"),
            receipt.considered_effect_count(),
        )
        .field_usize(
            WorthQueryEvidenceTag::new("delivered_effect_count"),
            receipt.delivered_effect_count(),
        )
        .field_usize(
            WorthQueryEvidenceTag::new("pending_write_intent_count"),
            receipt.pending_write_intent_count(),
        )
        .field_usize(
            WorthQueryEvidenceTag::new("suppressed_effect_count"),
            receipt.suppressed_effect_count(),
        )
        .field_usize(
            WorthQueryEvidenceTag::new("meaningful_effect_suppression_count"),
            receipt.meaningful_effect_suppression_count(),
        )
        .field_usize(
            WorthQueryEvidenceTag::new("effect_expression_failure_count"),
            receipt.effect_expression_failure_count(),
        )
        .field_bool(
            WorthQueryEvidenceTag::new("refresh_fallback"),
            receipt.refresh_fallback(),
        )
        .field_shape(
            WorthQueryEvidenceTag::new("admission_family"),
            receipt.admission_family(),
        )
        .field_shape(
            WorthQueryEvidenceTag::new("covered_entrypoint"),
            receipt.covered_entrypoint_label(),
        )
        .field_shape(
            WorthQueryEvidenceTag::new("execution_seam"),
            receipt.execution_seam_label(),
        )
        .field_value(
            WorthQueryEvidenceTag::new("admission_decision_digest"),
            receipt.admission_decision_digest(),
        )
        .field_value(
            WorthQueryEvidenceTag::new("execution_handoff_digest"),
            receipt.execution_handoff_digest(),
        )
        .field_value(
            WorthQueryEvidenceTag::new("execution_binding_digest"),
            receipt.execution_binding_digest(),
        )
        .field_evidence_identity(
            WorthQueryEvidenceTag::new("execution_provenance_chain_identity"),
            receipt
                .execution_provenance()
                .execution_provenance_chain_identity(),
        )
        .field_value(
            WorthQueryEvidenceTag::new("decision_trace_digest"),
            receipt.decision_trace_envelope().trace_digest(),
        )
        .seal()
        .expect("fixture digest must fit")
}

pub(in crate::runtime::tests) fn compose_intent_execution_provenance_chain_identity(
    receipt: &WorthQueryIntentReceipt,
) -> WorthQueryEvidenceIdentity {
    let provenance = receipt.execution_provenance();
    WorthQueryEvidenceIdentity::compose(WorthQueryEvidenceScope::IntentExecutionProvenanceChain)
        .field_shape(
            WorthQueryEvidenceTag::new("family"),
            provenance.family().as_str(),
        )
        .field_shape(
            WorthQueryEvidenceTag::new("entrypoint"),
            provenance.entrypoint().as_str(),
        )
        .field_shape(
            WorthQueryEvidenceTag::new("seam"),
            provenance.execution_seam().as_str(),
        )
        .field_value(
            WorthQueryEvidenceTag::new("admission_decision_digest"),
            provenance.admission_decision_digest(),
        )
        .field_value(
            WorthQueryEvidenceTag::new("execution_handoff_digest"),
            provenance.execution_handoff_digest(),
        )
        .field_value(
            WorthQueryEvidenceTag::new("execution_binding_digest"),
            provenance.execution_binding_digest(),
        )
        .field_value(
            WorthQueryEvidenceTag::new("execution_outcome_digest"),
            receipt.outcome_digest(),
        )
        .field_evidence_identity(
            WorthQueryEvidenceTag::new("snapshot_token"),
            receipt.snapshot_evidence_identity(),
        )
        .seal()
        .expect("fixture digest must fit")
}

fn receipt_value_identities(
    role: &'static str,
    values: &[String],
) -> Vec<WorthQueryEvidenceIdentity> {
    values
        .iter()
        .map(|value| {
            worth_query_evidence_identity(WorthQueryEvidenceScope::AuthoritativeIntentReceipt)
                .field_shape(WorthQueryEvidenceTag::new("role"), role)
                .field_value(WorthQueryEvidenceTag::new("value"), value)
                .seal()
        })
        .collect::<Result<_, _>>()
        .expect("fixture digests must fit")
}
