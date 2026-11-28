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

/// Iterative Euler traversal of an already qualified, parent-first forest.
/// The caller retains responsibility for validating the graph's edges.
pub fn ancestry_intervals<N: Clone + Ord, Edge: Copy>(
    roots: &BTreeSet<N>,
    children: &std::collections::BTreeMap<N, Vec<(N, Edge)>>,
    mut enter: impl FnMut(&N, Option<(&N, Edge)>),
) -> std::collections::BTreeMap<N, (u32, u32)> {
    enum Visit<N, Edge> {
        Enter(N, Option<(N, Edge)>),
        Exit(N),
    }
    let mut intervals = std::collections::BTreeMap::<N, (u32, u32)>::new();
    let mut timer = 0;
    let mut pending: Vec<_> = roots
        .iter()
        .rev()
        .map(|root| Visit::Enter(root.clone(), None))
        .collect();
    while let Some(visit) = pending.pop() {
        match visit {
            Visit::Exit(node) => intervals.get_mut(&node).expect("node was entered").1 = timer,
            Visit::Enter(node, parent) => {
                if intervals.contains_key(&node) {
                    continue;
                }
                timer += 1;
                intervals.insert(node.clone(), (timer, timer));
                enter(&node, parent.as_ref().map(|(parent, edge)| (parent, *edge)));
                pending.push(Visit::Exit(node.clone()));
                if let Some(children) = children.get(&node) {
                    pending.extend(children.iter().rev().map(|(child, edge)| {
                        Visit::Enter(child.clone(), Some((node.clone(), *edge)))
                    }));
                }
            }
        }
    }
    intervals
}

/// Follow a caller-defined chain with a shared seen set. Meter each actual
/// arrival, including a repeated node, before examining or extending the chain.
pub fn walk_acyclic_chain<N: Clone + Ord, T, E>(
    start: N,
    visited: &mut BTreeSet<N>,
    mut charge_visit: impl FnMut() -> Result<(), E>,
    mut step: impl FnMut(N) -> Result<std::ops::ControlFlow<T, N>, E>,
) -> Result<Option<T>, E> {
    use std::ops::ControlFlow;
    let mut current = start;
    loop {
        charge_visit()?;
        if !visited.insert(current.clone()) {
            return Ok(None);
        }
        match step(current)? {
            ControlFlow::Break(result) => return Ok(Some(result)),
            ControlFlow::Continue(next) => current = next,
        }
    }
}
