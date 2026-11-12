use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::evidence_identity::{
    worth_query_evidence_identity, WorthQueryEvidenceIdentity, WorthQueryEvidenceScope,
    WorthQueryEvidenceTag,
};

use super::super::{
    WorthQueryBranchIntentReceipt, WorthQueryEffectIntentReceipt, WorthQueryIntentReceipt,
};
use super::feedback::WorthQueryFeedbackPhaseGraphInspection;
use super::intent_delivery_counters::WorthQueryIntentInspectionDeliveryCounters;

pub(super) fn intent_inspection_delivery_counter_identity(
    counters: IntentInspectionDeliveryCounterIdentityParts,
) -> WorthQueryEvidenceIdentity {
    worth_query_evidence_identity(WorthQueryEvidenceScope::IntentInspectionDeliveryCounters)
            .field_usize(
                WorthQueryEvidenceTag::new("affected_live_view_count"),
                counters.affected_live_view_count,
            )
            .field_usize(
                WorthQueryEvidenceTag::new("affected_derived_view_count"),
                counters.affected_derived_view_count,
            )
            .field_usize(
                WorthQueryEvidenceTag::new("considered_computed_view_count"),
                counters.considered_computed_view_count,
            )
            .field_usize(
                WorthQueryEvidenceTag::new("considered_effect_count"),
                counters.considered_effect_count,
            )
            .field_usize(
                WorthQueryEvidenceTag::new("delivered_effect_count"),
                counters.delivered_effect_count,
            )
            .field_usize(
                WorthQueryEvidenceTag::new("pending_write_intent_count"),
                counters.pending_write_intent_count,
            )
            .field_usize(
                WorthQueryEvidenceTag::new("suppressed_effect_count"),
                counters.suppressed_effect_count,
            )
            .field_usize(
                WorthQueryEvidenceTag::new("meaningful_effect_suppression_count"),
                counters.meaningful_effect_suppression_count,
            )
            .field_usize(
                WorthQueryEvidenceTag::new("effect_expression_failure_count"),
                counters.effect_expression_failure_count,
            )
            .field_bool(
                WorthQueryEvidenceTag::new("refresh_fallback"),
                counters.refresh_fallback,
            )
            .seal().expect("fixed digest grammar: literal tags and values, primitive values, or sealed identity tokens")
}

pub(super) fn intent_receipt_inspection_identity(
    receipt: &WorthQueryIntentReceipt,
    produced_mutation_digest: Option<&str>,
    delivery_counters: &WorthQueryIntentInspectionDeliveryCounters,
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    worth_query_evidence_identity(WorthQueryEvidenceScope::IntentReceiptInspection)
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
        .optional_value(
            WorthQueryEvidenceTag::new("produced_mutation_digest"),
            produced_mutation_digest,
        )
        .field_value_sequence(
            WorthQueryEvidenceTag::new("invariant_evidence"),
            receipt.invariant_evidence().iter().map(String::as_str),
        )
        .field_shape(
            WorthQueryEvidenceTag::new("source_lane"),
            receipt.source_lane().as_str(),
        )
        .field_shape(
            WorthQueryEvidenceTag::new("target_lane"),
            receipt.target_lane().as_str(),
        )
        .field_evidence_identity(
            WorthQueryEvidenceTag::new("commit_evidence_identity"),
            receipt.commit_evidence_identity(),
        )
        .field_evidence_identity(
            WorthQueryEvidenceTag::new("snapshot_evidence_identity"),
            receipt.snapshot_evidence_identity(),
        )
        .field_evidence_identity(
            WorthQueryEvidenceTag::new("receipt_identity"),
            receipt.receipt_identity(),
        )
        .field_evidence_identity(
            WorthQueryEvidenceTag::new("delivery_counter_identity"),
            delivery_counters.counter_identity(),
        )
        .seal()
}

pub(super) fn branch_intent_receipt_inspection_basis_identity(
    receipt: &WorthQueryBranchIntentReceipt,
    basis_evidence: &[String],
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    worth_query_evidence_identity(WorthQueryEvidenceScope::BranchIntentReceiptInspectionBasis)
        .field_shape(
            WorthQueryEvidenceTag::new("intent_name"),
            receipt.intent_name(),
        )
        .field_evidence_identity(
            WorthQueryEvidenceTag::new("basis_snapshot_identity"),
            &receipt.basis_snapshot_identity().evidence_identity(),
        )
        .field_value_sequence(
            WorthQueryEvidenceTag::new("basis_evidence"),
            basis_evidence.iter().map(String::as_str),
        )
        .seal()
}

pub(super) fn branch_intent_receipt_inspection_identity(
    receipt: &WorthQueryBranchIntentReceipt,
    basis_identity: &WorthQueryEvidenceIdentity,
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    worth_query_evidence_identity(WorthQueryEvidenceScope::BranchIntentReceiptInspection)
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
            WorthQueryEvidenceTag::new("basis_snapshot_identity"),
            &receipt.basis_snapshot_identity().evidence_identity(),
        )
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

pub(super) fn effect_intent_receipt_phase_identity(
    receipt: &WorthQueryEffectIntentReceipt,
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    worth_query_evidence_identity(WorthQueryEvidenceScope::EffectIntentReceiptPhase)
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
        .seal()
}

pub(super) fn effect_intent_receipt_inspection_identity(
    receipt: &WorthQueryEffectIntentReceipt,
    phase_identity: &WorthQueryEvidenceIdentity,
    feedback_graph: &WorthQueryFeedbackPhaseGraphInspection,
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    worth_query_evidence_identity(WorthQueryEvidenceScope::EffectIntentReceiptInspection)
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
        .field_evidence_identity(WorthQueryEvidenceTag::new("phase_identity"), phase_identity)
        .field_evidence_identity(
            WorthQueryEvidenceTag::new("intent_receipt_identity"),
            receipt.intent_receipt().receipt_identity(),
        )
        .field_evidence_identity(
            WorthQueryEvidenceTag::new("receipt_identity"),
            receipt.receipt_identity(),
        )
        .field_evidence_identity(
            WorthQueryEvidenceTag::new("feedback_graph_identity"),
            feedback_graph.graph_identity(),
        )
        .seal()
}

pub(super) struct IntentInspectionDeliveryCounterIdentityParts {
    pub(super) affected_live_view_count: usize,
    pub(super) affected_derived_view_count: usize,
    pub(super) considered_computed_view_count: usize,
    pub(super) considered_effect_count: usize,
    pub(super) delivered_effect_count: usize,
    pub(super) pending_write_intent_count: usize,
    pub(super) suppressed_effect_count: usize,
    pub(super) meaningful_effect_suppression_count: usize,
    pub(super) effect_expression_failure_count: usize,
    pub(super) refresh_fallback: bool,
}
