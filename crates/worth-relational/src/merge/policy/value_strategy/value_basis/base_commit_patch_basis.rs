use worth_relational_branch::internal_port::history::data::canonical_commit_envelope::committed_record_changes::CanonicalCommitEnvelopeBodyOwnerPort as _;
use crate::history::data::CanonicalCommitEnvelope;
use crate::transactions::data::primitives::RecordRef;
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
