use crate::evidence_identity::WorthQueryEvidenceIdentity;
use crate::intent_admission::WorthQueryIntentDecisionTraceEnvelope;
use crate::runtime::WorthQueryIntentConsumerInspection;

use super::super::{
    WorthQueryAuthorityLane, WorthQueryEffectDelivery, WorthQueryEffectPhaseEvidence,
    WorthQueryEffectPolicy, WorthQueryEffectTriggerSourceKind,
    WorthQueryEffectWriteAdjacentTrigger, WorthQueryEffectWriteAdjacentTriggerClass,
};
use super::receipt_identity::effect_intent_receipt_identity;
use super::*;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryEffectIntentReceipt {
    effect_name: String,
    trigger_commit_evidence_identity: WorthQueryEvidenceIdentity,
    trigger_source_kind: WorthQueryEffectTriggerSourceKind,
    write_adjacent_trigger: WorthQueryEffectWriteAdjacentTrigger,
    pending_intent_target: String,
    source_lane: WorthQueryIntentSourceLane,
    target_lane: WorthQueryAuthorityLane,
    effect_policy: WorthQueryEffectPolicy,
    phase_evidence: WorthQueryEffectPhaseEvidence,
    intent_receipt: WorthQueryIntentReceipt,
    receipt_identity: WorthQueryEvidenceIdentity,
}

impl WorthQueryEffectIntentReceipt {
    pub(in crate::runtime) fn new(
        delivery: &WorthQueryEffectDelivery,
        intent_receipt: WorthQueryIntentReceipt,
    ) -> Result<Self, crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial> {
        let receipt_identity = effect_intent_receipt_identity(delivery, &intent_receipt)?;
        Ok(Self {
            effect_name: delivery.effect_name().to_string(),
            trigger_commit_evidence_identity: delivery.trigger_commit_evidence_identity().clone(),
            trigger_source_kind: delivery.trigger_source_kind(),
            write_adjacent_trigger: delivery.write_adjacent_trigger().clone(),
            pending_intent_target: delivery.target().to_string(),
            source_lane: WorthQueryIntentSourceLane::EffectTriggered,
            target_lane: intent_receipt.target_lane(),
            effect_policy: delivery.effect_policy(),
            phase_evidence: delivery.phase_evidence().clone(),
            intent_receipt,
            receipt_identity,
        })
    }

    pub fn effect_name(&self) -> &str {
        &self.effect_name
    }

    pub fn trigger_commit_evidence_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.trigger_commit_evidence_identity
    }

    pub fn trigger_source_kind(&self) -> WorthQueryEffectTriggerSourceKind {
        self.trigger_source_kind
    }

    pub fn write_adjacent_trigger(&self) -> &WorthQueryEffectWriteAdjacentTrigger {
        &self.write_adjacent_trigger
    }

    pub fn write_adjacent_trigger_class(&self) -> WorthQueryEffectWriteAdjacentTriggerClass {
        self.write_adjacent_trigger.class()
    }

    pub fn pending_intent_target(&self) -> &str {
        &self.pending_intent_target
    }

    pub fn source_lane(&self) -> WorthQueryIntentSourceLane {
        self.source_lane
    }

    pub fn target_lane(&self) -> WorthQueryAuthorityLane {
        self.target_lane
    }

    pub fn effect_policy(&self) -> WorthQueryEffectPolicy {
        self.effect_policy
    }

    pub fn phase_evidence(&self) -> &WorthQueryEffectPhaseEvidence {
        &self.phase_evidence
    }

    pub fn intent_receipt(&self) -> &WorthQueryIntentReceipt {
        &self.intent_receipt
    }

    pub fn admission_family(&self) -> &str {
        self.intent_receipt.admission_family()
    }

    pub fn covered_entrypoint_label(&self) -> &str {
        self.intent_receipt.covered_entrypoint_label()
    }

    pub fn execution_seam_label(&self) -> &str {
        self.intent_receipt.execution_seam_label()
    }

    pub fn admission_decision_digest(&self) -> &str {
        self.intent_receipt.admission_decision_digest()
    }

    pub fn execution_handoff_digest(&self) -> &str {
        self.intent_receipt.execution_handoff_digest()
    }

    pub fn execution_binding_digest(&self) -> &str {
        self.intent_receipt.execution_binding_digest()
    }

    pub fn execution_provenance_chain_digest(&self) -> &str {
        self.intent_receipt.execution_provenance_chain_digest()
    }

    pub fn execution_provenance(&self) -> &WorthQueryIntentExecutionProvenance {
        self.intent_receipt.execution_provenance()
    }

    pub fn decision_trace_envelope(&self) -> &WorthQueryIntentDecisionTraceEnvelope {
        self.intent_receipt.decision_trace_envelope()
    }

    pub fn consumer_inspection(&self) -> WorthQueryIntentConsumerInspection<'_> {
        self.intent_receipt.consumer_inspection()
    }

    pub fn receipt_digest(&self) -> &str {
        self.receipt_identity.as_str()
    }

    pub fn receipt_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.receipt_identity
    }
}
