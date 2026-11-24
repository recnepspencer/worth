use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;
use worth_relational_branch::internal_port::history::data::canonical_commit_envelope::CanonicalCommitEnvelopeBodyOwnerPort as _;

use crate::history::data::{BranchId, CanonicalCommitEnvelope, CommitId};
use crate::merge::data::{
    BranchCommitDelta, BranchDeltaSummary, BranchTouchedRecordDelta, MergePlanningError,
};
use crate::transactions::data::primitives::RecordRef;

pub(super) fn assemble_branch_delta(
    branch_id: BranchId,
    commits: &[CommitId],
    envelopes: &mut dyn Iterator<Item = Result<Arc<CanonicalCommitEnvelope>, MergePlanningError>>,
) -> Result<BranchCommitDelta, MergePlanningError> {
    let mut touched_records = BTreeMap::new();
    for envelope in envelopes {
        let envelope = envelope?;
        for target in envelope.touched_record_refs() {
            touched_records
                .entry(target)
                .or_insert_with(Vec::new)
                .push(envelope.clone());
        }
    }

    let touched_records = touched_records
        .into_iter()
        .map(|(target, envelopes)| BranchTouchedRecordDelta {
            target,
            envelopes: Arc::from(envelopes),
        })
        .collect::<Vec<_>>();

    Ok(BranchCommitDelta {
        branch_id,
        commits: Arc::from(commits.to_vec()),
        touched_records: Arc::from(touched_records),
    })
}

pub(super) fn branch_local_unique_commits(
    commits: &[CommitId],
    merge_base_ancestors: &[CommitId],
    is_branch_commit: &dyn Fn(CommitId) -> bool,
) -> Vec<CommitId> {
    let ancestor_set: BTreeSet<_> = merge_base_ancestors.iter().copied().collect();
    commits
        .iter()
        .copied()
        .filter(|commit| !ancestor_set.contains(commit))
        .filter(|commit| is_branch_commit(*commit))
        .collect()
}

pub(super) fn branch_delta_summary(
    head: &crate::history::data::RelationalCommitReceipt,
    delta: &BranchCommitDelta,
) -> BranchDeltaSummary {
    let touched_entity_count = delta
        .touched_records
        .iter()
        .filter(|record| matches!(record.target, RecordRef::Entity(_)))
        .count();
    let touched_relation_count = delta
        .touched_records
        .iter()
        .filter(|record| matches!(record.target, RecordRef::Relation(_)))
        .count();
    BranchDeltaSummary {
        branch_id: delta.branch_id.clone(),
        head_commit_id: head.commit_id,
        head_version_id: head.version_id,
        unique_commit_count: delta.commits.len(),
        touched_record_count: delta.touched_records.len(),
        touched_entity_count,
        touched_relation_count,
    }
}
