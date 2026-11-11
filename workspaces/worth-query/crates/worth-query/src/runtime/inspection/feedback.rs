use crate::evidence_identity::WorthQueryEvidenceIdentity;
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;

use super::super::{
    WorthQueryAuthorityLane, WorthQueryEffectDeliveryFamily, WorthQueryEffectIdempotence,
    WorthQueryEffectIntentReceipt, WorthQueryEffectLoopPrevention, WorthQueryEffectPolicy,
    WorthQueryEffectRuntime, WorthQueryEffectTriggerSourceKind,
    WorthQueryEffectWriteAdjacentTriggerClass, WorthQueryIntentExecutionKind,
};
use super::feedback_identity::{
    feedback_phase_graph_identity, feedback_phase_graph_inspection_identity,
    FeedbackPhaseGraphIdentityParts,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryFeedbackPhaseNode {
    TruthRead,
    Derive,
    EffectDelivery,
    PendingWriteIntent,
    Suppressed,
    ExpressionFailure,
    Commit,
    BridgeRoute,
    Resubscribe,
}

impl WorthQueryFeedbackPhaseNode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::TruthRead => "truth-read",
            Self::Derive => "derive",
            Self::EffectDelivery => "effect-delivery",
            Self::PendingWriteIntent => "pending-write-intent",
            Self::Suppressed => "suppressed",
            Self::ExpressionFailure => "expression-failure",
            Self::Commit => "commit",
            Self::BridgeRoute => "bridge-route",
            Self::Resubscribe => "resubscribe",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryFeedbackTermination {
    Delivered,
    PendingIntentDeferred,
    Suppressed,
    ExpressionFailed,
    CommittedResubscribe,
    CoalescedNoMutation,
}

impl WorthQueryFeedbackTermination {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Delivered => "delivered",
            Self::PendingIntentDeferred => "pending-intent-deferred",
            Self::Suppressed => "suppressed",
            Self::ExpressionFailed => "expression-failed",
            Self::CommittedResubscribe => "committed-resubscribe",
            Self::CoalescedNoMutation => "coalesced-no-mutation",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryFeedbackPhaseGraphInspection {
    effect_name: String,
    trigger_source_kind: WorthQueryEffectTriggerSourceKind,
    write_adjacent_trigger_class: WorthQueryEffectWriteAdjacentTriggerClass,
    write_adjacent_trigger_origin_identity: WorthQueryEvidenceIdentity,
    trigger_commit_evidence_identity: WorthQueryEvidenceIdentity,
    source_lane: WorthQueryAuthorityLane,
    terminal_lane: WorthQueryAuthorityLane,
    effect_policy: Option<WorthQueryEffectPolicy>,
    loop_prevention: WorthQueryEffectLoopPrevention,
    idempotence: WorthQueryEffectIdempotence,
    termination: WorthQueryFeedbackTermination,
    phase_nodes: Vec<WorthQueryFeedbackPhaseNode>,
    resubscribed_live_view_count: usize,
    resubscribed_derived_view_count: usize,
    pending_write_intent_count: usize,
    graph_digest: WorthQueryEvidenceIdentity,
    inspection_digest: WorthQueryEvidenceIdentity,
}

impl WorthQueryFeedbackPhaseGraphInspection {
    pub(in crate::runtime) fn from_effect_runtime(
        runtime: &WorthQueryEffectRuntime,
    ) -> Result<Option<Self>, IdentityDerivationDenial> {
        let Some(latest) = runtime.latest_delivery() else {
            return Ok(None);
        };
        let phase_nodes = latest
            .phase_evidence()
            .phases()
            .iter()
            .map(|phase| match phase.as_str() {
                "truth-read" => WorthQueryFeedbackPhaseNode::TruthRead,
                "derive" => WorthQueryFeedbackPhaseNode::Derive,
                "effect-delivery" => WorthQueryFeedbackPhaseNode::EffectDelivery,
                "pending-write-intent" => WorthQueryFeedbackPhaseNode::PendingWriteIntent,
                "suppressed" => WorthQueryFeedbackPhaseNode::Suppressed,
                "expression-failure" => WorthQueryFeedbackPhaseNode::ExpressionFailure,
                _ => unreachable!("effect phase vocabulary is closed"),
            })
            .collect::<Vec<_>>();
        let termination = match latest.family() {
            WorthQueryEffectDeliveryFamily::Delivered => WorthQueryFeedbackTermination::Delivered,
            WorthQueryEffectDeliveryFamily::PendingWriteIntent => {
                WorthQueryFeedbackTermination::PendingIntentDeferred
            }
            WorthQueryEffectDeliveryFamily::Suppressed => WorthQueryFeedbackTermination::Suppressed,
            WorthQueryEffectDeliveryFamily::ExpressionFailed => {
                WorthQueryFeedbackTermination::ExpressionFailed
            }
        };
        Ok(Some(Self::new(
            runtime.name(),
            latest.trigger_source_kind(),
            latest.write_adjacent_trigger().class(),
            latest.write_adjacent_trigger().origin_evidence_identity(),
            latest.trigger_commit_evidence_identity(),
            WorthQueryAuthorityLane::AuthoritativeTruth,
            latest.authority_lane(),
            Some(runtime.effect_policy()),
            latest.phase_evidence().loop_prevention(),
            latest.phase_evidence().idempotence(),
            termination,
            phase_nodes,
            0,
            0,
            runtime.pending_write_intent_count(),
        )?))
    }

    pub(in crate::runtime) fn from_effect_intent_receipt(
        receipt: &WorthQueryEffectIntentReceipt,
    ) -> Result<Self, IdentityDerivationDenial> {
        let mut phase_nodes = receipt
            .phase_evidence()
            .phases()
            .iter()
            .map(|phase| match phase.as_str() {
                "truth-read" => WorthQueryFeedbackPhaseNode::TruthRead,
                "derive" => WorthQueryFeedbackPhaseNode::Derive,
                "effect-delivery" => WorthQueryFeedbackPhaseNode::EffectDelivery,
                "pending-write-intent" => WorthQueryFeedbackPhaseNode::PendingWriteIntent,
                "suppressed" => WorthQueryFeedbackPhaseNode::Suppressed,
                "expression-failure" => WorthQueryFeedbackPhaseNode::ExpressionFailure,
                _ => unreachable!("effect phase vocabulary is closed"),
            })
            .collect::<Vec<_>>();
        phase_nodes.push(WorthQueryFeedbackPhaseNode::Commit);

        let (termination, resubscribed_live_view_count, resubscribed_derived_view_count) =
            if receipt.intent_receipt().execution_kind() == WorthQueryIntentExecutionKind::Mutating
            {
                phase_nodes.push(WorthQueryFeedbackPhaseNode::BridgeRoute);
                phase_nodes.push(WorthQueryFeedbackPhaseNode::Resubscribe);
                (
                    WorthQueryFeedbackTermination::CommittedResubscribe,
                    receipt.intent_receipt().affected_live_view_targets().len(),
                    receipt
                        .intent_receipt()
                        .affected_derived_view_targets()
                        .len(),
                )
            } else {
                (WorthQueryFeedbackTermination::CoalescedNoMutation, 0, 0)
            };

        Self::new(
            receipt.effect_name(),
            receipt.trigger_source_kind(),
            receipt.write_adjacent_trigger_class(),
            receipt.write_adjacent_trigger().origin_evidence_identity(),
            receipt.trigger_commit_evidence_identity(),
            WorthQueryAuthorityLane::AuthoritativeTruth,
            receipt.target_lane(),
            Some(receipt.effect_policy()),
            receipt.phase_evidence().loop_prevention(),
            receipt.phase_evidence().idempotence(),
            termination,
            phase_nodes,
            resubscribed_live_view_count,
            resubscribed_derived_view_count,
            receipt.intent_receipt().pending_write_intent_count(),
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn new(
        effect_name: &str,
        trigger_source_kind: WorthQueryEffectTriggerSourceKind,
        write_adjacent_trigger_class: WorthQueryEffectWriteAdjacentTriggerClass,
        write_adjacent_trigger_origin_identity: &WorthQueryEvidenceIdentity,
        trigger_commit_evidence_identity: &WorthQueryEvidenceIdentity,
        source_lane: WorthQueryAuthorityLane,
        terminal_lane: WorthQueryAuthorityLane,
        effect_policy: Option<WorthQueryEffectPolicy>,
        loop_prevention: WorthQueryEffectLoopPrevention,
        idempotence: WorthQueryEffectIdempotence,
        termination: WorthQueryFeedbackTermination,
        phase_nodes: Vec<WorthQueryFeedbackPhaseNode>,
        resubscribed_live_view_count: usize,
        resubscribed_derived_view_count: usize,
        pending_write_intent_count: usize,
    ) -> Result<Self, IdentityDerivationDenial> {
        let graph_digest = feedback_phase_graph_identity(FeedbackPhaseGraphIdentityParts {
            effect_name,
            trigger_source_kind,
            write_adjacent_trigger_class,
            write_adjacent_trigger_origin_identity,
            trigger_commit_identity: trigger_commit_evidence_identity,
            source_lane,
            terminal_lane,
            effect_policy,
            loop_prevention,
            idempotence,
            termination,
            phase_nodes: &phase_nodes,
            resubscribed_live_view_count,
            resubscribed_derived_view_count,
            pending_write_intent_count,
        })?;
        let inspection_digest = feedback_phase_graph_inspection_identity(&graph_digest);
        Ok(Self {
            effect_name: effect_name.to_string(),
            trigger_source_kind,
            write_adjacent_trigger_class,
            write_adjacent_trigger_origin_identity: write_adjacent_trigger_origin_identity.clone(),
            trigger_commit_evidence_identity: trigger_commit_evidence_identity.clone(),
            source_lane,
            terminal_lane,
            effect_policy,
            loop_prevention,
            idempotence,
            termination,
            phase_nodes,
            resubscribed_live_view_count,
            resubscribed_derived_view_count,
            pending_write_intent_count,
            graph_digest,
            inspection_digest,
        })
    }

    pub fn effect_name(&self) -> &str {
        &self.effect_name
    }

    pub fn trigger_source_kind(&self) -> WorthQueryEffectTriggerSourceKind {
        self.trigger_source_kind
    }

    pub fn write_adjacent_trigger_class(&self) -> WorthQueryEffectWriteAdjacentTriggerClass {
        self.write_adjacent_trigger_class
    }

    pub fn write_adjacent_trigger_origin_evidence_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.write_adjacent_trigger_origin_identity
    }

    pub fn trigger_commit_evidence_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.trigger_commit_evidence_identity
    }

    pub fn source_lane(&self) -> WorthQueryAuthorityLane {
        self.source_lane
    }

    pub fn terminal_lane(&self) -> WorthQueryAuthorityLane {
        self.terminal_lane
    }

    pub fn effect_policy(&self) -> Option<WorthQueryEffectPolicy> {
        self.effect_policy
    }

    pub fn loop_prevention(&self) -> WorthQueryEffectLoopPrevention {
        self.loop_prevention
    }

    pub fn idempotence(&self) -> WorthQueryEffectIdempotence {
        self.idempotence
    }

    pub fn termination(&self) -> WorthQueryFeedbackTermination {
        self.termination
    }

    pub fn phase_nodes(&self) -> &[WorthQueryFeedbackPhaseNode] {
        &self.phase_nodes
    }

    pub fn resubscribed_live_view_count(&self) -> usize {
        self.resubscribed_live_view_count
    }

    pub fn resubscribed_derived_view_count(&self) -> usize {
        self.resubscribed_derived_view_count
    }

    pub fn pending_write_intent_count(&self) -> usize {
        self.pending_write_intent_count
    }

    pub fn graph_digest(&self) -> &str {
        self.graph_digest.as_str()
    }

    pub fn graph_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.graph_digest
    }

    pub fn inspection_digest(&self) -> &str {
        self.inspection_digest.as_str()
    }

    pub fn inspection_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.inspection_digest
    }
}
