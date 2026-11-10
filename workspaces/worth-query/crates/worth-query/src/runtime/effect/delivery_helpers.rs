use crate::evidence_identity::{
    worth_query_evidence_identity, WorthQueryEvidenceIdentity, WorthQueryEvidenceScope,
    WorthQueryEvidenceTag,
};
use crate::memory_workspace::WorthQueryCommitIdentity;
use crate::runtime::WorthQueryAspectTouch;

pub(super) fn effect_trigger_commit_evidence_identity(
    commit_identity: &WorthQueryCommitIdentity,
) -> WorthQueryEvidenceIdentity {
    let commit_evidence_identity = commit_identity.evidence_identity();
    worth_query_evidence_identity(WorthQueryEvidenceScope::EffectTriggerCommitIdentity)
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("trigger_commit_identity"),
                &commit_evidence_identity,
            )
            .seal().expect("fixed digest grammar: literal tags and values, primitive values, or sealed identity tokens")
}

pub(super) fn terminal_touch_digest_projection_sequence(
    touches: &[WorthQueryAspectTouch],
) -> String {
    touches
        .iter()
        .map(WorthQueryAspectTouch::admitted_touch_digest_part)
        .collect::<Vec<_>>()
        .join(",")
}
