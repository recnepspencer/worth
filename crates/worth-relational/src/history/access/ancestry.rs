pub(crate) use super::ancestry_walk::CommitAncestryInspection;
use std::collections::BTreeSet;

use super::HistoryAccess;
#[cfg(test)]
use crate::history::data::BranchId;
use crate::history::data::CommitId;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CommitAncestryPosture {
    SelectedCommitUnavailable,
    RequestedCommitUnavailable,
    Reachable,
    Unreachable,
}

pub(crate) struct CommitAncestryClassification {
    posture: CommitAncestryPosture,
    traversal_work: usize,
}

impl CommitAncestryClassification {
    pub(crate) fn posture(&self) -> CommitAncestryPosture {
        self.posture
    }

    pub(crate) fn traversal_work(&self) -> usize {
        self.traversal_work
    }
}

impl<'runtime> HistoryAccess<'runtime> {
    /// Returns the ancestor closure for `commit_id`, ordered by ascending
    /// `CommitId`.
    ///
    /// This is not a linear parent chain on DAG-shaped history.
    pub fn ancestor_closure_by_commit_id_order(&self, commit_id: CommitId) -> Vec<CommitId> {
        self.ancestor_set(commit_id).into_iter().collect()
    }

    pub(crate) fn inspect_commit_ancestry(&self, start: CommitId) -> CommitAncestryInspection {
        super::ancestry_walk::inspect_commit_ancestry(start, |commit| {
            self.immutable_commit_receipt(commit)
        })
    }

    pub(crate) fn classify_commit_in_ancestry(
        &self,
        inspection: &CommitAncestryInspection,
        requested_commit: CommitId,
    ) -> CommitAncestryClassification {
        let posture = if !inspection.selected_commit_available {
            CommitAncestryPosture::SelectedCommitUnavailable
        } else if inspection.reachable_commits.contains(&requested_commit) {
            CommitAncestryPosture::Reachable
        } else if self.immutable_commit_receipt(requested_commit).is_some() {
            CommitAncestryPosture::Unreachable
        } else {
            CommitAncestryPosture::RequestedCommitUnavailable
        };
        CommitAncestryClassification {
            posture,
            traversal_work: inspection.total_work(),
        }
    }

    /// Convenience wrapper over the runtime's current common-ancestor
    /// selection rule for two branch heads.
    ///
    /// The underlying selection rule is `max_commit_id_common_ancestor`, which
    /// intersects both ancestor sets and chooses the maximum `CommitId`.
    #[cfg(test)]
    pub(crate) fn latest_common_ancestor_between_branches(
        &self,
        left_branch: &BranchId,
        right_branch: &BranchId,
    ) -> Option<CommitId> {
        let left = self.branch_head(left_branch)?.commit_id;
        let right = self.branch_head(right_branch)?.commit_id;
        self.max_commit_id_common_ancestor(left, right)
    }

    /// Returns the common ancestor selected by the runtime's current history
    /// rule: intersect both ancestor sets, then choose the maximum `CommitId`.
    ///
    /// This is the actual 7A-certified behavior. Callers should not read richer
    /// merge semantics into this helper than the implementation proves.
    #[cfg(test)]
    pub(crate) fn max_commit_id_common_ancestor(
        &self,
        left: CommitId,
        right: CommitId,
    ) -> Option<CommitId> {
        let left_ancestors = self.ancestor_set(left);
        let right_ancestors = self.ancestor_set(right);
        super::ancestry_walk::common_ancestor(&left_ancestors, &right_ancestors)
    }

    pub(super) fn ancestor_set(&self, start: CommitId) -> BTreeSet<CommitId> {
        let closure = self.inspect_commit_ancestry(start);
        self.runtime
            .performance_access()
            .count_merge_history_ancestry_traversal(closure.total_work());
        closure.reachable_commits
    }
}
