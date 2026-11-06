use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::evidence_identity::{
    worth_query_evidence_identity, WorthQueryEvidenceIdentity, WorthQueryEvidenceScope,
    WorthQueryEvidenceTag,
};

use super::super::{
    WorthQueryAuthorityLane, WorthQueryEffectIdempotence, WorthQueryEffectLoopPrevention,
    WorthQueryEffectPolicy, WorthQueryEffectTriggerSourceKind,
    WorthQueryEffectWriteAdjacentTriggerClass,
};
use super::feedback::{WorthQueryFeedbackPhaseNode, WorthQueryFeedbackTermination};

pub(super) fn feedback_phase_graph_identity(
    parts: FeedbackPhaseGraphIdentityParts<'_>,
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    Ok(
        worth_query_evidence_identity(WorthQueryEvidenceScope::FeedbackPhaseGraph)
            .field_shape(WorthQueryEvidenceTag::new("effect_name"), parts.effect_name)
            .field_shape(
                WorthQueryEvidenceTag::new("trigger_source_kind"),
                parts.trigger_source_kind.as_str(),
            )
            .field_shape(
                WorthQueryEvidenceTag::new("write_adjacent_trigger_class"),
                parts.write_adjacent_trigger_class.as_str(),
            )
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("write_adjacent_trigger_origin_identity"),
                parts.write_adjacent_trigger_origin_identity,
            )
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("trigger_commit_identity"),
                parts.trigger_commit_identity,
            )
            .field_shape(
                WorthQueryEvidenceTag::new("source_lane"),
                parts.source_lane.as_str(),
            )
            .field_shape(
                WorthQueryEvidenceTag::new("terminal_lane"),
                parts.terminal_lane.as_str(),
            )
            .optional_shape(
                WorthQueryEvidenceTag::new("effect_policy"),
                parts.effect_policy.map(WorthQueryEffectPolicy::as_str),
            )
            .field_shape(
                WorthQueryEvidenceTag::new("loop_prevention"),
                parts.loop_prevention.as_str(),
            )
            .field_shape(
                WorthQueryEvidenceTag::new("idempotence"),
                parts.idempotence.as_str(),
            )
            .field_shape(
                WorthQueryEvidenceTag::new("termination"),
                parts.termination.as_str(),
            )
            .field_value_sequence(
                WorthQueryEvidenceTag::new("phase_node"),
                parts.phase_nodes.iter().map(|node| node.as_str()),
            )
            .field_usize(
                WorthQueryEvidenceTag::new("resubscribed_live_view_count"),
                parts.resubscribed_live_view_count,
            )
            .field_usize(
                WorthQueryEvidenceTag::new("resubscribed_derived_view_count"),
                parts.resubscribed_derived_view_count,
            )
            .field_usize(
                WorthQueryEvidenceTag::new("pending_write_intent_count"),
                parts.pending_write_intent_count,
            )
            .seal()?,
    )
}

pub(super) fn feedback_phase_graph_inspection_identity(
    graph_identity: &WorthQueryEvidenceIdentity,
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    Ok(
        worth_query_evidence_identity(WorthQueryEvidenceScope::FeedbackPhaseGraphInspection)
            .field_evidence_identity(WorthQueryEvidenceTag::new("graph_digest"), graph_identity)
            .seal()?,
    )
}

pub(super) struct FeedbackPhaseGraphIdentityParts<'a> {
    pub(super) effect_name: &'a str,
    pub(super) trigger_source_kind: WorthQueryEffectTriggerSourceKind,
    pub(super) write_adjacent_trigger_class: WorthQueryEffectWriteAdjacentTriggerClass,
    pub(super) write_adjacent_trigger_origin_identity: &'a WorthQueryEvidenceIdentity,
    pub(super) trigger_commit_identity: &'a WorthQueryEvidenceIdentity,
    pub(super) source_lane: WorthQueryAuthorityLane,
    pub(super) terminal_lane: WorthQueryAuthorityLane,
    pub(super) effect_policy: Option<WorthQueryEffectPolicy>,
    pub(super) loop_prevention: WorthQueryEffectLoopPrevention,
    pub(super) idempotence: WorthQueryEffectIdempotence,
    pub(super) termination: WorthQueryFeedbackTermination,
    pub(super) phase_nodes: &'a [WorthQueryFeedbackPhaseNode],
    pub(super) resubscribed_live_view_count: usize,
    pub(super) resubscribed_derived_view_count: usize,
    pub(super) pending_write_intent_count: usize,
}
