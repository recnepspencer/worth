use super::super::WorthQueryFeedbackPhaseGraphInspection;
use super::super::{WorthQueryAuthorityLane, WorthQueryEffectAction, WorthQueryEffectPolicy};
use super::declaration::{
    WorthQueryEffectCondition, WorthQueryEffectExpressionFailurePosture,
    WorthQueryEffectSuppressionPolicy, WorthQueryEffectTriggerSourceKind,
};
use super::delivery::{WorthQueryEffectCounters, WorthQueryEffectDeliveryFamily};
use super::follow_on::{
    WorthQueryEffectWriteAdjacentTrigger, WorthQueryEffectWriteAdjacentTriggerClass,
};
use super::inspection_identity::effect_inspection_digests;
use super::phase::WorthQueryEffectPhaseEvidence;
use super::registry::WorthQueryEffectRuntime;
use crate::runtime::WorthQueryAspectTouch;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryEffectInspectionEvidence {
    name: String,
    trigger_source: String,
    trigger_source_kind: WorthQueryEffectTriggerSourceKind,
    write_adjacent_trigger: WorthQueryEffectWriteAdjacentTrigger,
    trigger_aspects: Vec<WorthQueryAspectTouch>,
    condition_descriptor: String,
    condition_inputs: Vec<WorthQueryAspectTouch>,
    condition_outputs: Vec<WorthQueryAspectTouch>,
    condition_failure_posture: Option<WorthQueryEffectExpressionFailurePosture>,
    action: WorthQueryEffectAction,
    target_lane: WorthQueryAuthorityLane,
    target: String,
    effect_policy: WorthQueryEffectPolicy,
    suppression_policy: WorthQueryEffectSuppressionPolicy,
    counters: WorthQueryEffectCounters,
    pending_delivery_count: usize,
    pending_delivered_count: usize,
    pending_suppressed_count: usize,
    pending_expression_failure_count: usize,
    pending_write_intent_count: usize,
    latest_delivery_family: Option<WorthQueryEffectDeliveryFamily>,
    latest_phase_evidence: Option<WorthQueryEffectPhaseEvidence>,
    feedback_graph: Option<WorthQueryFeedbackPhaseGraphInspection>,
    trigger_digest: String,
    condition_digest: String,
    declaration_digest: String,
    pending_delivery_digest: String,
    latest_phase_digest: Option<String>,
    inspection_digest: String,
}

impl WorthQueryEffectInspectionEvidence {
    pub(in crate::runtime) fn from_runtime(
        effect: &WorthQueryEffectRuntime,
    ) -> Result<Self, crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial> {
        let (condition_descriptor, condition_inputs, condition_outputs) =
            match effect.declaration.condition() {
                WorthQueryEffectCondition::Always => ("always".to_string(), Vec::new(), Vec::new()),
                WorthQueryEffectCondition::Expression(expression) => (
                    expression.descriptor().to_string(),
                    expression.input_aspect_touches().to_vec(),
                    expression.output_aspect_touches().to_vec(),
                ),
            };
        let condition_failure_posture = match effect.declaration.condition() {
            WorthQueryEffectCondition::Always => None,
            WorthQueryEffectCondition::Expression(expression) => Some(expression.failure_posture()),
        };
        let pending_delivered_count = effect
            .deliveries
            .iter()
            .filter(|delivery| delivery.family() == &WorthQueryEffectDeliveryFamily::Delivered)
            .count();
        let pending_suppressed_count = effect
            .deliveries
            .iter()
            .filter(|delivery| delivery.family() == &WorthQueryEffectDeliveryFamily::Suppressed)
            .count();
        let pending_expression_failure_count = effect
            .deliveries
            .iter()
            .filter(|delivery| {
                delivery.family() == &WorthQueryEffectDeliveryFamily::ExpressionFailed
            })
            .count();
        let pending_write_intent_count = effect
            .deliveries
            .iter()
            .filter(|delivery| {
                delivery.family() == &WorthQueryEffectDeliveryFamily::PendingWriteIntent
            })
            .count();
        let pending_delivery_count =
            pending_delivered_count + pending_suppressed_count + pending_expression_failure_count;
        let latest_delivery_family = effect
            .latest_delivery()
            .map(|delivery| delivery.family().clone());
        let latest_phase_evidence = effect
            .latest_delivery()
            .map(|delivery| delivery.phase_evidence().clone());
        let feedback_graph = WorthQueryFeedbackPhaseGraphInspection::from_effect_runtime(effect)?;
        let digest_set = effect_inspection_digests(
            effect,
            &condition_descriptor,
            &condition_inputs,
            &condition_outputs,
            condition_failure_posture,
            pending_delivery_count,
            pending_delivered_count,
            pending_suppressed_count,
            pending_expression_failure_count,
            pending_write_intent_count,
            latest_delivery_family.as_ref(),
            latest_phase_evidence.as_ref(),
            feedback_graph.as_ref(),
        )?;

        Ok(Self {
            name: effect.declaration.name().to_string(),
            trigger_source: effect.declaration.trigger().source_name().to_string(),
            trigger_source_kind: effect.declaration.trigger().source_kind(),
            write_adjacent_trigger: effect.declaration.write_adjacent_trigger().clone(),
            trigger_aspects: effect.declaration.trigger().aspect_touches().to_vec(),
            condition_descriptor,
            condition_inputs,
            condition_outputs,
            condition_failure_posture,
            action: effect.declaration.action(),
            target_lane: effect.declaration.target_lane(),
            target: effect.declaration.target().to_string(),
            effect_policy: effect.declaration.effect_policy(),
            suppression_policy: effect.declaration.suppression_policy(),
            counters: effect.counters.clone(),
            pending_delivery_count,
            pending_delivered_count,
            pending_suppressed_count,
            pending_expression_failure_count,
            pending_write_intent_count,
            latest_delivery_family,
            latest_phase_evidence,
            feedback_graph,
            trigger_digest: digest_set.trigger_digest,
            condition_digest: digest_set.condition_digest,
            declaration_digest: digest_set.declaration_digest,
            pending_delivery_digest: digest_set.pending_delivery_digest,
            latest_phase_digest: digest_set.latest_phase_digest,
            inspection_digest: digest_set.inspection_digest,
        })
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn trigger_source(&self) -> &str {
        &self.trigger_source
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

    pub fn trigger_aspect_touches(&self) -> &[WorthQueryAspectTouch] {
        &self.trigger_aspects
    }

    pub fn condition_descriptor(&self) -> &str {
        &self.condition_descriptor
    }

    pub fn condition_input_touches(&self) -> &[WorthQueryAspectTouch] {
        &self.condition_inputs
    }

    pub fn condition_output_touches(&self) -> &[WorthQueryAspectTouch] {
        &self.condition_outputs
    }

    pub fn condition_failure_posture(&self) -> Option<WorthQueryEffectExpressionFailurePosture> {
        self.condition_failure_posture
    }

    pub fn action(&self) -> WorthQueryEffectAction {
        self.action
    }

    pub fn target_lane(&self) -> WorthQueryAuthorityLane {
        self.target_lane
    }

    pub fn target(&self) -> &str {
        &self.target
    }

    pub fn effect_policy(&self) -> WorthQueryEffectPolicy {
        self.effect_policy
    }

    pub fn suppression_policy(&self) -> WorthQueryEffectSuppressionPolicy {
        self.suppression_policy
    }

    pub fn counters(&self) -> &WorthQueryEffectCounters {
        &self.counters
    }

    pub fn pending_delivery_count(&self) -> usize {
        self.pending_delivery_count
    }

    pub fn pending_delivered_count(&self) -> usize {
        self.pending_delivered_count
    }

    pub fn pending_suppressed_count(&self) -> usize {
        self.pending_suppressed_count
    }

    pub fn pending_expression_failure_count(&self) -> usize {
        self.pending_expression_failure_count
    }

    pub fn pending_write_intent_count(&self) -> usize {
        self.pending_write_intent_count
    }

    pub fn latest_delivery_family(&self) -> Option<&WorthQueryEffectDeliveryFamily> {
        self.latest_delivery_family.as_ref()
    }

    pub fn latest_phase_evidence(&self) -> Option<&WorthQueryEffectPhaseEvidence> {
        self.latest_phase_evidence.as_ref()
    }

    pub fn feedback_graph(&self) -> Option<&WorthQueryFeedbackPhaseGraphInspection> {
        self.feedback_graph.as_ref()
    }

    pub fn trigger_digest(&self) -> &str {
        &self.trigger_digest
    }

    pub fn condition_digest(&self) -> &str {
        &self.condition_digest
    }

    pub fn declaration_digest(&self) -> &str {
        &self.declaration_digest
    }

    pub fn pending_delivery_digest(&self) -> &str {
        &self.pending_delivery_digest
    }

    pub fn latest_phase_digest(&self) -> Option<&str> {
        self.latest_phase_digest.as_deref()
    }

    pub fn inspection_digest(&self) -> &str {
        &self.inspection_digest
    }
}
