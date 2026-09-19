use std::collections::BTreeSet;

use crate::branch::AdmittedRelationalBranchBasis;
#[cfg(test)]
use crate::history::data::BranchId;
use crate::history::data::CommitId;
#[cfg(test)]
use crate::history::data::{MergeConflictRecord, MergeInspection};

use super::HistoryAccess;

pub(crate) struct CommitAncestryInspection {
    reachable_commits: BTreeSet<CommitId>,
    authoring_branches: BTreeSet<crate::history::data::BranchId>,
    selected_commit_available: bool,
    node_visits: usize,
    catalog_probes: usize,
    parent_edge_visits: usize,
}

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

impl CommitAncestryInspection {
    pub(crate) fn authoring_branches(&self) -> &BTreeSet<crate::history::data::BranchId> {
        &self.authoring_branches
    }

    pub(crate) fn node_visits(&self) -> usize {
        self.node_visits
    }

    pub(crate) fn catalog_probes(&self) -> usize {
        self.catalog_probes
    }

    pub(crate) fn parent_edge_visits(&self) -> usize {
        self.parent_edge_visits
    }

    pub(crate) fn total_work(&self) -> usize {
        self.node_visits
            .saturating_add(self.catalog_probes)
            .saturating_add(self.parent_edge_visits)
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
        let mut visited_commits = BTreeSet::new();
        let mut reachable_commits = BTreeSet::new();
        let mut authoring_branches = BTreeSet::new();
        let mut selected_commit_available = false;
        let mut stack = vec![start];
        let mut node_visits = 0;
        let mut catalog_probes = 0;
        let mut parent_edge_visits = 0;
        while let Some(commit_id) = stack.pop() {
            node_visits += 1;
            if !visited_commits.insert(commit_id) {
                continue;
            }
            catalog_probes += 1;
            if let Some(envelope) = self.runtime.history.canonical_envelope(commit_id) {
                reachable_commits.insert(commit_id);
                selected_commit_available |= commit_id == start;
                authoring_branches.insert(envelope.commit.branch_id.clone());
                parent_edge_visits += envelope.commit.parents.len();
                stack.extend(envelope.commit.parents.iter().copied());
            }
        }
        CommitAncestryInspection {
            reachable_commits,
            authoring_branches,
            selected_commit_available,
            node_visits,
            catalog_probes,
            parent_edge_visits,
        }
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
        } else if self
            .runtime
            .history
            .canonical_envelope(requested_commit)
            .is_some()
        {
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

    /// Selects the common ancestor from two exact owner observations.  Raw
    /// branch names are intentionally not a production currentness input.
    pub(crate) fn latest_common_ancestor_between_bindings(
        &self,
        left_binding: &AdmittedRelationalBranchBasis,
        right_binding: &AdmittedRelationalBranchBasis,
    ) -> Option<CommitId> {
        let left = self.bound_head_commit_id(left_binding)?;
        let right = self.bound_head_commit_id(right_binding)?;
        self.max_commit_id_common_ancestor(left, right)
    }

    fn bound_head_commit_id(&self, binding: &AdmittedRelationalBranchBasis) -> Option<CommitId> {
        binding
            .commit_identity()
            .map(|identity| identity.commit_id())
    }

    #[cfg(test)]
    pub(crate) fn can_merge_branch_into(
        &self,
        source_branch: &BranchId,
        target_branch: &BranchId,
    ) -> bool {
        let Some(source_head) = self.branch_head(source_branch) else {
            return false;
        };
        let Some(target_head) = self.branch_head(target_branch) else {
            return false;
        };
        self.max_commit_id_common_ancestor(target_head.commit_id, source_head.commit_id)
            .is_some()
    }

    #[cfg(test)]
    pub(crate) fn inspect_merge(
        &self,
        source_branch: &BranchId,
        target_branch: &BranchId,
    ) -> MergeInspection {
        // This remains a history substrate helper. New merge-semantic planning
        // should land in `crate::merge`, not grow richer semantics here.
        let source_head = self.branch_head(source_branch);
        let target_head = self.branch_head(target_branch);
        let merge_base =
            source_head
                .as_ref()
                .zip(target_head.as_ref())
                .and_then(|(source, target)| {
                    self.max_commit_id_common_ancestor(source.commit_id, target.commit_id)
                });

        let source_only_commits = source_head
            .as_ref()
            .map(|head| {
                self.branch_unique_commit_closure_by_commit_id_order(head.commit_id, merge_base)
            })
            .unwrap_or_default();
        let target_only_commits = target_head
            .as_ref()
            .map(|head| {
                self.branch_unique_commit_closure_by_commit_id_order(head.commit_id, merge_base)
            })
            .unwrap_or_default();
        let conflicting_records = self.merge_conflicts_between(
            source_only_commits.as_slice(),
            target_only_commits.as_slice(),
        );

        MergeInspection {
            source_branch: source_branch.clone(),
            target_branch: target_branch.clone(),
            source_head,
            target_head,
            merge_base,
            source_only_commits,
            target_only_commits,
            can_merge: merge_base.is_some() && conflicting_records.is_empty(),
            conflicting_records,
        }
    }

    /// Returns the common ancestor selected by the runtime's current history
    /// rule: intersect both ancestor sets, then choose the maximum `CommitId`.
    ///
    /// This is the actual 7A-certified behavior. Callers should not read richer
    /// merge semantics into this helper than the implementation proves.
    pub(crate) fn max_commit_id_common_ancestor(
        &self,
        left: CommitId,
        right: CommitId,
    ) -> Option<CommitId> {
        let left_ancestors = self.ancestor_set(left);
        let right_ancestors = self.ancestor_set(right);
        left_ancestors
            .intersection(&right_ancestors)
            .copied()
            .max_by_key(|commit_id| commit_id.0)
    }

    pub(super) fn ancestor_set(&self, start: CommitId) -> BTreeSet<CommitId> {
        let closure = self.inspect_commit_ancestry(start);
        self.runtime
            .performance_access()
            .count_merge_history_ancestry_traversal(closure.total_work());
        closure.reachable_commits
    }

    /// Returns the branch-local ancestor closure for `head` after removing the
    /// merge-base ancestor closure, ordered by ascending `CommitId`.
    #[cfg(test)]
    fn branch_unique_commit_closure_by_commit_id_order(
        &self,
        head: CommitId,
        merge_base: Option<CommitId>,
    ) -> Vec<CommitId> {
        let mut commits = self.ancestor_set(head).into_iter().collect::<Vec<_>>();
        if let Some(merge_base) = merge_base {
            let base_ancestors = self.ancestor_set(merge_base);
            commits.retain(|commit_id| !base_ancestors.contains(commit_id));
        }
        debug_assert!(commits.windows(2).all(|window| window[0] <= window[1]));
        commits
    }

    #[cfg(test)]
    fn merge_conflicts_between(
        &self,
        left_commits: &[CommitId],
        right_commits: &[CommitId],
    ) -> Vec<MergeConflictRecord> {
        let left_records = self.commit_record_set(left_commits);
        let right_records = self.commit_record_set(right_commits);
        left_records.intersection(&right_records).cloned().collect()
    }

    #[cfg(test)]
    fn commit_record_set(&self, commits: &[CommitId]) -> BTreeSet<MergeConflictRecord> {
        commits
            .iter()
            .filter_map(|commit_id| self.runtime.history.canonical_envelope(*commit_id))
            .flat_map(|envelope| envelope.touched_record_refs().into_iter())
            .map(|record_ref| match record_ref {
                crate::transactions::data::RecordRef::Entity(entity_id) => {
                    MergeConflictRecord::Entity(entity_id)
                }
                crate::transactions::data::RecordRef::Relation(relation_id) => {
                    MergeConflictRecord::Relation(relation_id)
                }
            })
            .collect()
    }
}
