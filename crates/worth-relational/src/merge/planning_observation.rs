//! Immutable original contributions retained by the shipping merge planner.
use crate::history::data::CommitId;
use crate::merge::data::{BranchCommitDelta, MergePlanningArtifactCore};
use crate::transactions::data::RecordRef;

/// A planning artifact with its actual per-record original commit contributions.
/// This observation grants no mutation, transaction admission or publication.
#[derive(Debug)]
pub struct RelationalMergePlanningObservation {
    artifact: MergePlanningArtifactCore,
    target: BranchCommitDelta,
    source: BranchCommitDelta,
}

impl RelationalMergePlanningObservation {
    pub(super) fn new(
        artifact: MergePlanningArtifactCore,
        target: BranchCommitDelta,
        source: BranchCommitDelta,
    ) -> Self {
        Self {
            artifact,
            target,
            source,
        }
    }

    pub fn artifact(&self) -> &MergePlanningArtifactCore {
        &self.artifact
    }

    pub fn into_artifact(self) -> MergePlanningArtifactCore {
        self.artifact
    }

    /// Borrow contributions in the planner's canonical history order.
    /// Lookup is O(touched records); iteration is O(contributing originals).
    pub fn target_contributing_commits(
        &self,
        record: RecordRef,
    ) -> impl DoubleEndedIterator<Item = CommitId> + '_ {
        contributions(&self.target, record)
    }

    /// Borrow source contributions without reconstructing or reading history.
    pub fn source_contributing_commits(
        &self,
        record: RecordRef,
    ) -> impl DoubleEndedIterator<Item = CommitId> + '_ {
        contributions(&self.source, record)
    }
}

fn contributions(
    delta: &BranchCommitDelta,
    record: RecordRef,
) -> impl DoubleEndedIterator<Item = CommitId> + '_ {
    delta
        .touched_records
        .iter()
        .find(|row| row.target == record)
        .into_iter()
        .flat_map(|row| row.commit_ids())
}
