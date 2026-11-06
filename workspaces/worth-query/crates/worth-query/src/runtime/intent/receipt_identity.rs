use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::evidence_identity::{
    worth_query_evidence_identity, WorthQueryEvidenceIdentity, WorthQueryEvidenceScope,
    WorthQueryEvidenceTag,
};
use crate::intent_admission::WorthQueryIntentDecisionTraceEnvelope;

use super::super::{WorthQueryEffectDelivery, WorthQueryEffectPhase};
use super::{
    WorthQueryIntentDeclaration, WorthQueryIntentExecution, WorthQueryIntentExecutionProvenance,
    WorthQueryIntentSourceLane, WorthQueryWriteReceipt,
};

pub(super) fn authoritative_intent_receipt_identity(
    declaration: &WorthQueryIntentDeclaration,
    execution: &WorthQueryIntentExecution,
    write_receipt: &WorthQueryWriteReceipt,
    execution_provenance: &WorthQueryIntentExecutionProvenance,
    decision_trace_envelope: &WorthQueryIntentDecisionTraceEnvelope,
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    let invariant_evidence_identities = receipt_value_identities(
        "authoritative-receipt-invariant-evidence",
        execution.invariant_evidence(),
    );
    let effect_trigger_identity = declaration.effect_trigger().map(|trigger| {
        worth_query_evidence_identity(WorthQueryEvidenceScope::AuthoritativeIntentReceipt)
            .field_shape(
                WorthQueryEvidenceTag::new("role"),
                "authoritative-receipt-effect-trigger",
            )
            .field_value(WorthQueryEvidenceTag::new("digest"), trigger.digest())
            .seal()
    });
    let affected_live_view_identities = receipt_value_identities(
        "authoritative-receipt-affected-live-view",
        &write_receipt.terminal_affected_live_view_ids_projection(),
    );
    let affected_derived_view_identities = receipt_value_identities(
        "authoritative-receipt-affected-derived-view",
        &write_receipt.terminal_affected_derived_view_ids_projection(),
    );

    Ok(
        worth_query_evidence_identity(WorthQueryEvidenceScope::AuthoritativeIntentReceipt)
            .field_shape(
                WorthQueryEvidenceTag::new("intent_name"),
                declaration.name(),
            )
            .field_shape(
                WorthQueryEvidenceTag::new("execution_kind"),
                execution.execution_kind().as_str(),
            )
            .field_value(
                WorthQueryEvidenceTag::new("strategy_identity"),
                execution.strategy_identity(),
            )
            .field_shape(
                WorthQueryEvidenceTag::new("strategy_version"),
                execution.strategy_version(),
            )
            .field_value(
                WorthQueryEvidenceTag::new("strategy_descriptor_digest"),
                execution.strategy_descriptor_digest(),
            )
            .field_value(
                WorthQueryEvidenceTag::new("canonical_input_digest"),
                execution.canonical_input_digest(),
            )
            .field_value(
                WorthQueryEvidenceTag::new("outcome_digest"),
                execution.outcome_digest(),
            )
            .field_evidence_identity_sequence(
                WorthQueryEvidenceTag::new("invariant_evidence"),
                invariant_evidence_identities.iter(),
            )
            .field_shape(
                WorthQueryEvidenceTag::new("source_lane"),
                declaration.source_lane().as_str(),
            )
            .field_shape(
                WorthQueryEvidenceTag::new("target_lane"),
                declaration.target_lane().as_str(),
            )
            .optional_evidence_identity(
                WorthQueryEvidenceTag::new("effect_trigger_digest"),
                effect_trigger_identity.as_ref()?,
            )
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("commit_evidence_identity"),
                write_receipt.commit_evidence_identity(),
            )
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("snapshot_evidence_identity"),
                write_receipt.snapshot_evidence_identity(),
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
                write_receipt.considered_computed_view_count(),
            )
            .field_usize(
                WorthQueryEvidenceTag::new("considered_effect_count"),
                write_receipt.considered_effect_count(),
            )
            .field_usize(
                WorthQueryEvidenceTag::new("delivered_effect_count"),
                write_receipt.delivered_effect_count(),
            )
            .field_usize(
                WorthQueryEvidenceTag::new("pending_write_intent_count"),
                write_receipt.pending_write_intent_count(),
            )
            .field_usize(
                WorthQueryEvidenceTag::new("suppressed_effect_count"),
                write_receipt.suppressed_effect_count(),
            )
            .field_usize(
                WorthQueryEvidenceTag::new("meaningful_effect_suppression_count"),
                write_receipt.meaningful_effect_suppression_count(),
            )
            .field_usize(
                WorthQueryEvidenceTag::new("effect_expression_failure_count"),
                write_receipt.effect_expression_failure_count(),
            )
            .field_bool(
                WorthQueryEvidenceTag::new("refresh_fallback"),
                write_receipt.refresh_fallback(),
            )
            .field_shape(
                WorthQueryEvidenceTag::new("admission_family"),
                execution_provenance.family().as_str(),
            )
            .field_shape(
                WorthQueryEvidenceTag::new("covered_entrypoint"),
                execution_provenance.entrypoint().as_str(),
            )
            .field_shape(
                WorthQueryEvidenceTag::new("execution_seam"),
                execution_provenance.execution_seam().as_str(),
            )
            .field_value(
                WorthQueryEvidenceTag::new("admission_decision_digest"),
                execution_provenance.admission_decision_digest(),
            )
            .field_value(
                WorthQueryEvidenceTag::new("execution_handoff_digest"),
                execution_provenance.execution_handoff_digest(),
            )
            .field_value(
                WorthQueryEvidenceTag::new("execution_binding_digest"),
                execution_provenance.execution_binding_digest(),
            )
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("execution_provenance_chain_identity"),
                execution_provenance.execution_provenance_chain_identity(),
            )
            .field_value(
                WorthQueryEvidenceTag::new("decision_trace_digest"),
                decision_trace_envelope.trace_digest(),
            )
            .seal()?,
    )
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
        .collect()
}

pub(super) fn effect_intent_receipt_identity(
    delivery: &WorthQueryEffectDelivery,
    intent_receipt: &super::WorthQueryIntentReceipt,
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    Ok(
        worth_query_evidence_identity(WorthQueryEvidenceScope::EffectIntentReceipt)
            .field_shape(
                WorthQueryEvidenceTag::new("effect_name"),
                delivery.effect_name(),
            )
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("trigger_commit_evidence_identity"),
                delivery.trigger_commit_evidence_identity(),
            )
            .field_shape(
                WorthQueryEvidenceTag::new("trigger_source_kind"),
                delivery.trigger_source_kind().as_str(),
            )
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("write_adjacent_trigger"),
                delivery.write_adjacent_trigger().identity(),
            )
            .field_value(
                WorthQueryEvidenceTag::new("pending_intent_target"),
                delivery.target(),
            )
            .field_shape(
                WorthQueryEvidenceTag::new("source_lane"),
                WorthQueryIntentSourceLane::EffectTriggered.as_str(),
            )
            .field_shape(
                WorthQueryEvidenceTag::new("target_lane"),
                intent_receipt.target_lane().as_str(),
            )
            .field_shape(
                WorthQueryEvidenceTag::new("effect_policy"),
                delivery.effect_policy().as_str(),
            )
            .field_value_sequence(
                WorthQueryEvidenceTag::new("phase"),
                delivery
                    .phase_evidence()
                    .phases()
                    .iter()
                    .map(|phase: &WorthQueryEffectPhase| phase.as_str()),
            )
            .field_shape(
                WorthQueryEvidenceTag::new("loop_prevention"),
                delivery.phase_evidence().loop_prevention().as_str(),
            )
            .field_shape(
                WorthQueryEvidenceTag::new("idempotence"),
                delivery.phase_evidence().idempotence().as_str(),
            )
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("intent_receipt_identity"),
                intent_receipt.receipt_identity(),
            )
            .seal()?,
    )
}
