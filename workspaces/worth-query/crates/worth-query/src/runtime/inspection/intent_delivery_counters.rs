use super::super::WorthQueryIntentReceipt;
use super::intent_identity::{
    intent_inspection_delivery_counter_identity, IntentInspectionDeliveryCounterIdentityParts,
};
use crate::evidence_identity::WorthQueryEvidenceIdentity;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryIntentInspectionDeliveryCounters {
    affected_live_view_count: usize,
    affected_derived_view_count: usize,
    considered_computed_view_count: usize,
    considered_effect_count: usize,
    delivered_effect_count: usize,
    pending_write_intent_count: usize,
    suppressed_effect_count: usize,
    meaningful_effect_suppression_count: usize,
    effect_expression_failure_count: usize,
    refresh_fallback: bool,
    counter_digest: WorthQueryEvidenceIdentity,
}

impl WorthQueryIntentInspectionDeliveryCounters {
    pub(in crate::runtime) fn from_receipt(receipt: &WorthQueryIntentReceipt) -> Self {
        let affected_live_view_count = receipt.affected_live_view_targets().len();
        let affected_derived_view_count = receipt.affected_derived_view_targets().len();
        let considered_computed_view_count = receipt.considered_computed_view_count();
        let considered_effect_count = receipt.considered_effect_count();
        let delivered_effect_count = receipt.delivered_effect_count();
        let pending_write_intent_count = receipt.pending_write_intent_count();
        let suppressed_effect_count = receipt.suppressed_effect_count();
        let meaningful_effect_suppression_count = receipt.meaningful_effect_suppression_count();
        let effect_expression_failure_count = receipt.effect_expression_failure_count();
        let refresh_fallback = receipt.refresh_fallback();
        let counter_digest = intent_inspection_delivery_counter_identity(
            IntentInspectionDeliveryCounterIdentityParts {
                affected_live_view_count,
                affected_derived_view_count,
                considered_computed_view_count,
                considered_effect_count,
                delivered_effect_count,
                pending_write_intent_count,
                suppressed_effect_count,
                meaningful_effect_suppression_count,
                effect_expression_failure_count,
                refresh_fallback,
            },
        );
        Self {
            affected_live_view_count,
            affected_derived_view_count,
            considered_computed_view_count,
            considered_effect_count,
            delivered_effect_count,
            pending_write_intent_count,
            suppressed_effect_count,
            meaningful_effect_suppression_count,
            effect_expression_failure_count,
            refresh_fallback,
            counter_digest,
        }
    }

    pub fn affected_live_view_count(&self) -> usize {
        self.affected_live_view_count
    }

    pub fn affected_derived_view_count(&self) -> usize {
        self.affected_derived_view_count
    }

    pub fn considered_computed_view_count(&self) -> usize {
        self.considered_computed_view_count
    }

    pub fn considered_effect_count(&self) -> usize {
        self.considered_effect_count
    }

    pub fn delivered_effect_count(&self) -> usize {
        self.delivered_effect_count
    }

    pub fn pending_write_intent_count(&self) -> usize {
        self.pending_write_intent_count
    }

    pub fn suppressed_effect_count(&self) -> usize {
        self.suppressed_effect_count
    }

    pub fn meaningful_effect_suppression_count(&self) -> usize {
        self.meaningful_effect_suppression_count
    }

    pub fn effect_expression_failure_count(&self) -> usize {
        self.effect_expression_failure_count
    }

    pub fn refresh_fallback(&self) -> bool {
        self.refresh_fallback
    }

    pub fn counter_digest(&self) -> &str {
        self.counter_digest.as_str()
    }

    pub fn counter_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.counter_digest
    }
}
