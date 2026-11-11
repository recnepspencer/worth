use super::super::delivery_cause::QuerySubscriptionDeliveryCauseKind;
use super::super::patch_group::QueryPatchGroupKind;
use super::super::slice::QuerySubscriptionSlicePart;
use super::SUBSCRIPTION_IDENTITY_SCOPE;
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::evidence_identity::{WorthQueryEvidenceIdentity, WorthQueryEvidenceTag};
use crate::live::LiveQueryFamily;

pub(in crate::subscription) fn live_relevance_identity(
    live_family: &LiveQueryFamily,
    query_identity: &WorthQueryEvidenceIdentity,
    plan_identity: &WorthQueryEvidenceIdentity,
) -> WorthQueryEvidenceIdentity {
    WorthQueryEvidenceIdentity::compose(SUBSCRIPTION_IDENTITY_SCOPE)
            .field_shape(
                WorthQueryEvidenceTag::new("identity_family"),
                "live_relevance_v1",
            )
            .field_shape(
                WorthQueryEvidenceTag::new("live_family"),
                live_family.as_str(),
            )
            .field_evidence_identity(WorthQueryEvidenceTag::new("query"), query_identity)
            .field_evidence_identity(WorthQueryEvidenceTag::new("plan"), plan_identity)
            .seal().expect("fixed digest grammar: literal tags, primitive values, closed enum labels, or sealed digest tokens")
}

pub(in crate::subscription) fn slice_intent_identity(
    parts: &[QuerySubscriptionSlicePart],
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    WorthQueryEvidenceIdentity::compose(SUBSCRIPTION_IDENTITY_SCOPE)
        .field_shape(
            WorthQueryEvidenceTag::new("identity_family"),
            "query_subscription_slice_intent_v1",
        )
        .field_value_sequence(
            WorthQueryEvidenceTag::new("parts"),
            parts.iter().map(QuerySubscriptionSlicePart::canonical_part),
        )
        .seal()
}

pub(in crate::subscription) fn patch_group_identity(
    kind: QueryPatchGroupKind,
    source_identity: &WorthQueryEvidenceIdentity,
    width: u64,
) -> WorthQueryEvidenceIdentity {
    WorthQueryEvidenceIdentity::compose(SUBSCRIPTION_IDENTITY_SCOPE)
            .field_shape(
                WorthQueryEvidenceTag::new("identity_family"),
                "query_patch_group_v1",
            )
            .field_shape(WorthQueryEvidenceTag::new("kind"), kind.as_str())
            .field_evidence_identity(WorthQueryEvidenceTag::new("source"), source_identity)
            .field_usize(WorthQueryEvidenceTag::new("width"), width as usize)
            .seal().expect("fixed digest grammar: literal tags, primitive values, closed enum labels, or sealed digest tokens")
}

pub(in crate::subscription) fn delivery_cause_evidence_label_identity(
    label: &str,
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    WorthQueryEvidenceIdentity::compose(SUBSCRIPTION_IDENTITY_SCOPE)
        .field_shape(
            WorthQueryEvidenceTag::new("identity_family"),
            "query_subscription_delivery_cause_evidence_label_v1",
        )
        .field_shape(WorthQueryEvidenceTag::new("label"), label)
        .seal()
}

pub(in crate::subscription) fn delivery_cause_identity(
    kind: QuerySubscriptionDeliveryCauseKind,
    evidence_identity: &WorthQueryEvidenceIdentity,
) -> WorthQueryEvidenceIdentity {
    WorthQueryEvidenceIdentity::compose(SUBSCRIPTION_IDENTITY_SCOPE)
            .field_shape(
                WorthQueryEvidenceTag::new("identity_family"),
                "query_subscription_delivery_cause_v1",
            )
            .field_shape(WorthQueryEvidenceTag::new("kind"), kind.as_str())
            .field_evidence_identity(WorthQueryEvidenceTag::new("evidence"), evidence_identity)
            .seal().expect("fixed digest grammar: literal tags, primitive values, closed enum labels, or sealed digest tokens")
}

pub(in crate::subscription) fn live_delivery_intent_projection_identity(
    live_family: &LiveQueryFamily,
) -> WorthQueryEvidenceIdentity {
    WorthQueryEvidenceIdentity::compose(SUBSCRIPTION_IDENTITY_SCOPE)
            .field_shape(
                WorthQueryEvidenceTag::new("identity_family"),
                "live_delivery_intent_v1",
            )
            .field_shape(
                WorthQueryEvidenceTag::new("live_family"),
                live_family.as_str(),
            )
            .seal().expect("fixed digest grammar: literal tags, primitive values, closed enum labels, or sealed digest tokens")
}

pub(in crate::subscription) fn subscription_fanout_plan_identity(
    lane_identity: &WorthQueryEvidenceIdentity,
    affected_consumer_attachment_width: u64,
) -> WorthQueryEvidenceIdentity {
    WorthQueryEvidenceIdentity::compose(SUBSCRIPTION_IDENTITY_SCOPE)
            .field_shape(
                WorthQueryEvidenceTag::new("identity_family"),
                "subscription_fanout_plan_v1",
            )
            .field_evidence_identity(WorthQueryEvidenceTag::new("lane"), lane_identity)
            .field_usize(
                WorthQueryEvidenceTag::new("affected_consumer_attachment_width"),
                affected_consumer_attachment_width as usize,
            )
            .seal().expect("fixed digest grammar: literal tags and values, primitive values, or sealed identity tokens")
}

pub(in crate::subscription) fn subscription_fanout_report_identity(
    plan_identity: &WorthQueryEvidenceIdentity,
    shared_lane_count: u64,
    fanout_width: u64,
) -> WorthQueryEvidenceIdentity {
    WorthQueryEvidenceIdentity::compose(SUBSCRIPTION_IDENTITY_SCOPE)
            .field_shape(
                WorthQueryEvidenceTag::new("identity_family"),
                "subscription_fanout_report_v1",
            )
            .field_evidence_identity(WorthQueryEvidenceTag::new("plan"), plan_identity)
            .field_usize(
                WorthQueryEvidenceTag::new("shared_lane_count"),
                shared_lane_count as usize,
            )
            .field_usize(
                WorthQueryEvidenceTag::new("fanout_width"),
                fanout_width as usize,
            )
            .seal().expect("fixed digest grammar: literal tags and values, primitive values, or sealed identity tokens")
}
