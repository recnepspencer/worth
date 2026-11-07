use crate::evidence_identity::WorthQueryEvidenceIdentity;
use crate::memory_workspace::WorthQueryCommitIdentity;
use crate::runtime::WorthQueryAspectTouch;

use super::super::WorthQueryEffectPolicy;
use super::super::{WorthQueryAuthorityLane, WorthQueryEffectAction};
use super::declaration::{
    WorthQueryEffectDeclaration, WorthQueryEffectSuppressionPolicy,
    WorthQueryEffectTriggerSourceKind,
};
use super::delivery_helpers::{
    effect_trigger_commit_evidence_identity, terminal_touch_digest_projection_sequence,
};
use super::follow_on::WorthQueryEffectWriteAdjacentTrigger;
use super::phase::WorthQueryEffectPhaseEvidence;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct WorthQueryEffectCounters {
    pub(in crate::runtime::effect) considered: usize,
    pub(in crate::runtime::effect) delivered: usize,
    pub(in crate::runtime::effect) pending_write_intents: usize,
    pub(in crate::runtime::effect) suppressed: usize,
    pub(in crate::runtime::effect) meaningful_suppressions: usize,
    pub(in crate::runtime::effect) expression_failures: usize,
}
impl WorthQueryEffectCounters {
    pub fn considered(&self) -> usize {
        self.considered
    }
    pub fn delivered(&self) -> usize {
        self.delivered
    }
    pub fn pending_write_intents(&self) -> usize {
        self.pending_write_intents
    }
    pub fn suppressed(&self) -> usize {
        self.suppressed
    }
    pub fn meaningful_suppressions(&self) -> usize {
        self.meaningful_suppressions
    }
    pub fn expression_failures(&self) -> usize {
        self.expression_failures
    }
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorthQueryEffectDeliveryFamily {
    Delivered,
    PendingWriteIntent,
    Suppressed,
    ExpressionFailed,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryEffectPayload {
    condition: Option<String>,
    input_aspects: Vec<WorthQueryAspectTouch>,
    output_aspects: Vec<WorthQueryAspectTouch>,
    changed_aspects: Vec<WorthQueryAspectTouch>,
}

impl WorthQueryEffectPayload {
    pub(in crate::runtime::effect) fn always(changed_aspects: &[WorthQueryAspectTouch]) -> Self {
        Self {
            condition: Some("always".to_string()),
            input_aspects: Vec::new(),
            output_aspects: Vec::new(),
            changed_aspects: changed_aspects.to_vec(),
        }
    }

    pub(in crate::runtime::effect) fn expression(
        condition: impl Into<String>,
        input_aspects: &[WorthQueryAspectTouch],
        output_aspects: &[WorthQueryAspectTouch],
        changed_aspects: &[WorthQueryAspectTouch],
    ) -> Self {
        Self {
            condition: Some(condition.into()),
            input_aspects: input_aspects.to_vec(),
            output_aspects: output_aspects.to_vec(),
            changed_aspects: changed_aspects.to_vec(),
        }
    }

    pub(in crate::runtime::effect) fn empty() -> Self {
        Self {
            condition: None,
            input_aspects: Vec::new(),
            output_aspects: Vec::new(),
            changed_aspects: Vec::new(),
        }
    }

    pub fn condition(&self) -> Option<&str> {
        self.condition.as_deref()
    }

    pub fn input_aspect_touches(&self) -> &[WorthQueryAspectTouch] {
        &self.input_aspects
    }

    pub fn output_aspect_touches(&self) -> &[WorthQueryAspectTouch] {
        &self.output_aspects
    }

    pub fn changed_aspect_touches(&self) -> &[WorthQueryAspectTouch] {
        &self.changed_aspects
    }

    pub(crate) fn terminal_digest_material(&self) -> String {
        let Some(condition) = self.condition.as_deref() else {
            return "condition:<none>".to_string();
        };
        [
            format!("condition:{condition}"),
            format!(
                "input:{}",
                terminal_touch_digest_projection_sequence(&self.input_aspects)
            ),
            format!(
                "output:{}",
                terminal_touch_digest_projection_sequence(&self.output_aspects)
            ),
            format!(
                "changed:{}",
                terminal_touch_digest_projection_sequence(&self.changed_aspects)
            ),
        ]
        .join("|")
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct WorthQueryEffectDelivery {
    effect_name: String,
    commit_identity: WorthQueryCommitIdentity,
    trigger_commit_evidence_identity: WorthQueryEvidenceIdentity,
    trigger_source: String,
    trigger_source_kind: WorthQueryEffectTriggerSourceKind,
    target: String,
    action: WorthQueryEffectAction,
    authority_lane: WorthQueryAuthorityLane,
    effect_policy: WorthQueryEffectPolicy,
    aspect_touches: Vec<WorthQueryAspectTouch>,
    family: WorthQueryEffectDeliveryFamily,
    suppression_policy: WorthQueryEffectSuppressionPolicy,
    phase_evidence: WorthQueryEffectPhaseEvidence,
    write_adjacent_trigger: WorthQueryEffectWriteAdjacentTrigger,
    payload: WorthQueryEffectPayload,
    reason: Option<String>,
}
impl WorthQueryEffectDelivery {
    pub(in crate::runtime::effect) fn delivered(
        declaration: &WorthQueryEffectDeclaration,
        commit_identity: WorthQueryCommitIdentity,
        trigger_source: impl Into<String>,
        trigger_source_kind: WorthQueryEffectTriggerSourceKind,
        aspect_touches: Vec<WorthQueryAspectTouch>,
        payload: WorthQueryEffectPayload,
    ) -> Self {
        let trigger_commit_evidence_identity =
            effect_trigger_commit_evidence_identity(&commit_identity);
        Self {
            effect_name: declaration.name().to_string(),
            commit_identity,
            trigger_commit_evidence_identity,
            trigger_source: trigger_source.into(),
            trigger_source_kind,
            target: declaration.target().to_string(),
            action: declaration.action(),
            authority_lane: declaration.target_lane(),
            effect_policy: declaration.effect_policy(),
            aspect_touches,
            family: WorthQueryEffectDeliveryFamily::Delivered,
            suppression_policy: declaration.suppression_policy(),
            phase_evidence: WorthQueryEffectPhaseEvidence::delivery(),
            write_adjacent_trigger: declaration.write_adjacent_trigger().clone(),
            payload,
            reason: None,
        }
    }
    pub(in crate::runtime::effect) fn pending_write_intent(
        declaration: &WorthQueryEffectDeclaration,
        commit_identity: WorthQueryCommitIdentity,
        trigger_source: impl Into<String>,
        trigger_source_kind: WorthQueryEffectTriggerSourceKind,
        aspect_touches: Vec<WorthQueryAspectTouch>,
        payload: WorthQueryEffectPayload,
    ) -> Self {
        let trigger_commit_evidence_identity =
            effect_trigger_commit_evidence_identity(&commit_identity);
        Self {
            effect_name: declaration.name().to_string(),
            commit_identity,
            trigger_commit_evidence_identity,
            trigger_source: trigger_source.into(),
            trigger_source_kind,
            target: declaration.target().to_string(),
            action: declaration.action(),
            authority_lane: declaration.target_lane(),
            effect_policy: declaration.effect_policy(),
            aspect_touches,
            family: WorthQueryEffectDeliveryFamily::PendingWriteIntent,
            suppression_policy: declaration.suppression_policy(),
            phase_evidence: WorthQueryEffectPhaseEvidence::pending_write_intent(),
            write_adjacent_trigger: declaration.write_adjacent_trigger().clone(),
            payload,
            reason: Some(
                "effect lowered to pending write intent; commit execution awaits intent authority"
                    .to_string(),
            ),
        }
    }
    pub(in crate::runtime::effect) fn suppressed(
        declaration: &WorthQueryEffectDeclaration,
        commit_identity: WorthQueryCommitIdentity,
        trigger_source: impl Into<String>,
        trigger_source_kind: WorthQueryEffectTriggerSourceKind,
        reason: impl Into<String>,
    ) -> Self {
        let trigger_commit_evidence_identity =
            effect_trigger_commit_evidence_identity(&commit_identity);
        Self {
            effect_name: declaration.name().to_string(),
            commit_identity,
            trigger_commit_evidence_identity,
            trigger_source: trigger_source.into(),
            trigger_source_kind,
            target: declaration.target().to_string(),
            action: declaration.action(),
            authority_lane: declaration.target_lane(),
            effect_policy: declaration.effect_policy(),
            aspect_touches: Vec::new(),
            family: WorthQueryEffectDeliveryFamily::Suppressed,
            suppression_policy: declaration.suppression_policy(),
            phase_evidence: WorthQueryEffectPhaseEvidence::suppressed(),
            write_adjacent_trigger: declaration.write_adjacent_trigger().clone(),
            payload: WorthQueryEffectPayload::empty(),
            reason: Some(reason.into()),
        }
    }
    pub(in crate::runtime::effect) fn expression_failed(
        declaration: &WorthQueryEffectDeclaration,
        commit_identity: WorthQueryCommitIdentity,
        trigger_source: impl Into<String>,
        trigger_source_kind: WorthQueryEffectTriggerSourceKind,
        aspect_touches: Vec<WorthQueryAspectTouch>,
        reason: impl Into<String>,
    ) -> Self {
        let trigger_commit_evidence_identity =
            effect_trigger_commit_evidence_identity(&commit_identity);
        Self {
            effect_name: declaration.name().to_string(),
            commit_identity,
            trigger_commit_evidence_identity,
            trigger_source: trigger_source.into(),
            trigger_source_kind,
            target: declaration.target().to_string(),
            action: declaration.action(),
            authority_lane: declaration.target_lane(),
            effect_policy: declaration.effect_policy(),
            aspect_touches,
            family: WorthQueryEffectDeliveryFamily::ExpressionFailed,
            suppression_policy: declaration.suppression_policy(),
            phase_evidence: WorthQueryEffectPhaseEvidence::expression_failure(),
            write_adjacent_trigger: declaration.write_adjacent_trigger().clone(),
            payload: WorthQueryEffectPayload::empty(),
            reason: Some(reason.into()),
        }
    }
    pub fn effect_name(&self) -> &str {
        &self.effect_name
    }
    pub fn commit_identity(&self) -> &WorthQueryCommitIdentity {
        &self.commit_identity
    }
    pub fn trigger_commit_evidence_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.trigger_commit_evidence_identity
    }
    pub fn trigger_source(&self) -> &str {
        &self.trigger_source
    }
    pub fn trigger_source_kind(&self) -> WorthQueryEffectTriggerSourceKind {
        self.trigger_source_kind
    }
    pub fn target(&self) -> &str {
        &self.target
    }
    pub fn action(&self) -> WorthQueryEffectAction {
        self.action
    }
    pub fn authority_lane(&self) -> WorthQueryAuthorityLane {
        self.authority_lane
    }
    pub fn effect_policy(&self) -> WorthQueryEffectPolicy {
        self.effect_policy
    }
    pub fn aspect_touches(&self) -> &[WorthQueryAspectTouch] {
        &self.aspect_touches
    }

    pub fn family(&self) -> &WorthQueryEffectDeliveryFamily {
        &self.family
    }
    pub fn suppression_policy(&self) -> WorthQueryEffectSuppressionPolicy {
        self.suppression_policy
    }
    pub fn phase_evidence(&self) -> &WorthQueryEffectPhaseEvidence {
        &self.phase_evidence
    }
    pub fn write_adjacent_trigger(&self) -> &WorthQueryEffectWriteAdjacentTrigger {
        &self.write_adjacent_trigger
    }
    pub fn payload(&self) -> &WorthQueryEffectPayload {
        &self.payload
    }
    pub fn reason(&self) -> Option<&str> {
        self.reason.as_deref()
    }
}
