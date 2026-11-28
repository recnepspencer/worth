//! Traverse the immutable parent ledger for a selected commit.
use crate::history::data::{CommitId, RelationalCommitReceipt};
use std::collections::BTreeSet;
pub struct CommitAncestryInspection {
    pub reachable_commits: BTreeSet<CommitId>,
    authoring_branches: BTreeSet<crate::history::data::BranchId>,
    pub selected_commit_available: bool,
    node_visits: usize,
    catalog_probes: usize,
    parent_edge_visits: usize,
}

impl CommitAncestryInspection {
    pub fn commit_ids(&self) -> &BTreeSet<CommitId> {
        &self.reachable_commits
    }
    pub fn authoring_branches(&self) -> &BTreeSet<crate::history::data::BranchId> {
        &self.authoring_branches
    }

    pub fn node_visits(&self) -> usize {
        self.node_visits
    }

    pub fn catalog_probes(&self) -> usize {
        self.catalog_probes
    }

    pub fn parent_edge_visits(&self) -> usize {
        self.parent_edge_visits
    }

    pub fn total_work(&self) -> usize {
        self.node_visits
            .saturating_add(self.catalog_probes)
            .saturating_add(self.parent_edge_visits)
    }
}

pub fn inspect_commit_ancestry(
    start: CommitId,
    receipt: impl FnMut(CommitId) -> Option<RelationalCommitReceipt>,
) -> CommitAncestryInspection {
    let Ok(inspection) = inspect_commit_ancestry_with_checkpoint(start, receipt, |_| {
        Ok::<_, std::convert::Infallible>(())
    });
    inspection
}

pub fn inspect_commit_ancestry_with_checkpoint<E>(
    start: CommitId,
    mut receipt: impl FnMut(CommitId) -> Option<RelationalCommitReceipt>,
    mut checkpoint: impl FnMut(u64) -> Result<(), E>,
) -> Result<CommitAncestryInspection, E> {
    let mut visited_commits = BTreeSet::new();
    let mut reachable_commits = BTreeSet::new();
    let mut authoring_branches = BTreeSet::new();
    let mut selected_commit_available = false;
    let mut stack = vec![start];
    let mut node_visits = 0;
    let mut catalog_probes = 0;
    let mut parent_edge_visits = 0;
    while let Some(commit_id) = stack.pop() {
        checkpoint(1)?;
        node_visits += 1;
        if !visited_commits.insert(commit_id) {
            continue;
        }
        checkpoint(1)?;
        catalog_probes += 1;
        if let Some(receipt) = receipt(commit_id) {
            reachable_commits.insert(commit_id);
            selected_commit_available |= commit_id == start;
            authoring_branches.insert(receipt.branch_id.clone());
            checkpoint(receipt.parents.len() as u64)?;
            parent_edge_visits += receipt.parents.len();
            stack.extend(receipt.parents.iter().copied());
        }
    }
    Ok(CommitAncestryInspection {
        reachable_commits,
        authoring_branches,
        selected_commit_available,
        node_visits,
        catalog_probes,
        parent_edge_visits,
    })
}

pub fn common_ancestor(
    left_ancestors: &BTreeSet<CommitId>,
    right_ancestors: &BTreeSet<CommitId>,
) -> Option<CommitId> {
    left_ancestors
        .intersection(right_ancestors)
        .copied()
        .max_by_key(|commit_id| commit_id.0)
}
