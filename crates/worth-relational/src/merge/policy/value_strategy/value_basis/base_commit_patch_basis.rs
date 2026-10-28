use crate::history::data::CanonicalCommitEnvelope;
use crate::transactions::data::RecordRef;
use worth_foundational::facade::{AspectKey, AspectValue};

pub(super) fn scalar_from_base_commit_patch(
    envelope: &CanonicalCommitEnvelope,
    candidate_targets: &[RecordRef],
    aspect_key: &AspectKey,
) -> Option<AspectValue> {
    candidate_targets.iter().find_map(|target| {
        envelope
            .committed_record_changes_for_target(target)
            .find_map(|change| {
                change
                    .record
                    .authoritative_patch
                    .scalar_set_for(aspect_key)
                    .cloned()
            })
    })
}
