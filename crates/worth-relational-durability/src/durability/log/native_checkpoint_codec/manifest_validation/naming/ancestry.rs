//! Linear `O(V + E)` combined live + retired fork-ancestry validation and
//! index for read-free `CHK25` manifest validation.

use std::collections::{BTreeMap, BTreeSet};

use worth_foundational::FoundationalBranchTarget;
use worth_relational_contracts::internal_port::durability::data::recovery_errors::DurabilityError;
use worth_relational_contracts::internal_port::history::data::coordinates::BranchId;
use worth_relational_lineage::internal_port::lineage::naming_pages::tables;

use super::super::super::manifest_capture::corrupt;
use crate::durability::data::RelationalCheckpointManifest;

type ReqBounds = (Option<u64>, Option<u64>, Option<u64>);

fn min_opt(a: Option<u64>, b: Option<u64>) -> Option<u64> {
    match (a, b) {
        (Some(x), Some(y)) => Some(x.min(y)),
        (x, y) => x.or(y),
    }
}

pub(super) struct AncestryIndex<'a> {
    committed: BTreeSet<&'a BranchId>,
    euler: BTreeMap<&'a BranchId, (u32, u32)>,
    nearest_present: BTreeMap<&'a BranchId, &'a BranchId>,
    topmost_retired: BTreeMap<&'a BranchId, &'a BranchId>,
    inherited_min: BTreeMap<&'a BranchId, ReqBounds>,
}

impl<'a> AncestryIndex<'a> {
    /// Validates `manifest.naming.retired_forks` and the combined live + retired
    /// fork graph (`O(V + E)` acyclicity, parent-head bounds, no dangling edges)
    /// BEFORE building `O(1)` Euler-tour provenance and inherited requirement
    /// bounds for every branch.
    pub(super) fn build(
        manifest: &'a RelationalCheckpointManifest,
        tail_commit: Option<u64>,
        group_log_branch_min: &BTreeMap<[u8; 32], u64>,
    ) -> Result<Self, DurabilityError> {
        let mut cell_heads: BTreeMap<&'a BranchId, Option<u64>> = BTreeMap::new();
        let mut committed: BTreeSet<&'a BranchId> = BTreeSet::new();
        let mut branch_forks: BTreeMap<&'a BranchId, (&'a BranchId, u64)> = BTreeMap::new();
        for cell in &manifest.branch_cells {
            let head = match cell.observation.target() {
                FoundationalBranchTarget::Basis(target) => {
                    committed.insert(&cell.branch_id);
                    Some(target.selected_commit_id())
                }
                FoundationalBranchTarget::Empty => None,
            };
            cell_heads.insert(&cell.branch_id, head);
            if let (Some(source), Some(prov)) = (&cell.fork_source_branch_id, &cell.fork_provenance)
                && let FoundationalBranchTarget::Basis(target) = prov.target()
            {
                branch_forks.insert(&cell.branch_id, (source, target.selected_commit_id()));
            }
        }

        let retired: BTreeSet<&'a BranchId> = manifest.retired_branch_names.iter().collect();
        for edge in &manifest.naming.retired_forks {
            let parent_cell_ok = match cell_heads.get(&edge.parent) {
                Some(Some(parent_head)) => edge.fork_commit <= *parent_head,
                Some(None) => false,
                None => true,
            };
            if !retired.contains(&edge.branch)
                || committed.contains(&edge.branch)
                || branch_forks.contains_key(&edge.branch)
                || (!committed.contains(&edge.parent) && !retired.contains(&edge.parent))
                || tail_commit.is_none_or(|tail| edge.fork_commit == 0 || edge.fork_commit > tail)
                || !parent_cell_ok
            {
                return Err(corrupt(
                    "checkpoint naming retired_forks edge violates manifest branch provenance",
                ));
            }
            branch_forks.insert(&edge.branch, (&edge.parent, edge.fork_commit));
        }

        // 3-color iterative DFS (`O(V + E)`): combined live + retired graph must be acyclic.
        let mut color: BTreeMap<&'a BranchId, u8> = BTreeMap::new();
        let mut path: Vec<&'a BranchId> = Vec::new();
        for &start in branch_forks.keys() {
            if color.get(start).copied().unwrap_or(0) != 0 {
                continue;
            }
            path.clear();
            let mut cur = start;
            loop {
                match color.get(cur).copied().unwrap_or(0) {
                    1 => {
                        return Err(corrupt(
                            "checkpoint branch fork provenance graph contains a cycle",
                        ));
                    }
                    2 => break,
                    _ => {
                        color.insert(cur, 1);
                        path.push(cur);
                        match branch_forks.get(cur) {
                            Some(&(parent, _)) => cur = parent,
                            None => break,
                        }
                    }
                }
            }
            for node in &path {
                color.insert(node, 2);
            }
        }

        // Reject any dangling retired_forks edge not on an ancestor path of a
        // committed cell, and reject any retired ancestor reached on that path
        // that omits its outgoing `retired_forks` edge.
        let mut reachable: BTreeSet<&'a BranchId> = BTreeSet::new();
        for &branch in &committed {
            let mut cur = branch;
            while reachable.insert(cur) {
                match branch_forks.get(cur) {
                    Some(&(parent, _)) => cur = parent,
                    None => {
                        if retired.contains(cur) && !committed.contains(cur) {
                            return Err(corrupt(
                                "checkpoint naming retired_forks omits retired ancestor edge",
                            ));
                        }
                        break;
                    }
                }
            }
        }
        for edge in &manifest.naming.retired_forks {
            if !reachable.contains(&edge.branch) {
                return Err(corrupt(
                    "checkpoint naming retired_forks contains unreferenced edge",
                ));
            }
        }

        // Build top-down forest adjacency and roots.
        let mut kids: BTreeMap<&'a BranchId, Vec<(&'a BranchId, u64)>> = BTreeMap::new();
        let mut roots: BTreeSet<&'a BranchId> = BTreeSet::new();
        for (&child, &(parent, fc)) in &branch_forks {
            kids.entry(parent).or_default().push((child, fc));
            if !branch_forks.contains_key(parent) {
                roots.insert(parent);
            }
        }
        for &branch in &committed {
            if !branch_forks.contains_key(branch) {
                roots.insert(branch);
            }
        }

        let own_mins = |u: &BranchId| -> ReqBounds {
            let (mut n, mut i, mut g) = (None, None, None);
            if let Some(root) = manifest.naming.root(u) {
                n = root.naming.and_then(|h| {
                    if h.unavailable {
                        Some(0)
                    } else {
                        h.earliest_event_commit
                    }
                });
                i = root.index.and_then(|h| {
                    if h.unavailable {
                        Some(0)
                    } else {
                        h.earliest_commit
                    }
                });
                g = root.groups.as_ref().and_then(|h| {
                    if h.unavailable {
                        Some(0)
                    } else {
                        h.earliest_commit
                    }
                });
            }
            if let Some(&at) = group_log_branch_min.get(&tables::branch_key_digest(u)) {
                g = Some(g.map_or(at, |prev| prev.min(at)));
            }
            (n, i, g)
        };

        let mut nearest_present: BTreeMap<&'a BranchId, &'a BranchId> = BTreeMap::new();
        let mut topmost_retired: BTreeMap<&'a BranchId, &'a BranchId> = BTreeMap::new();
        let mut retired_root: BTreeMap<&'a BranchId, &'a BranchId> = BTreeMap::new();
        let mut inherited_min: BTreeMap<&'a BranchId, ReqBounds> = BTreeMap::new();
        let mut best_min: BTreeMap<&'a BranchId, ReqBounds> = BTreeMap::new();

        let euler = worth_relational_branch::internal_port::history::access::ancestry_walk::ancestry_intervals(&roots, &kids, |&node, parent_edge| {
            let own = own_mins(node);
            if let Some((&parent, fc)) = parent_edge {
                if committed.contains(parent) {
                    nearest_present.insert(node, parent);
                } else if let Some(&anc) = nearest_present.get(parent) {
                    nearest_present.insert(node, anc);
                }
                if let Some(&ret_root) = retired_root.get(parent) {
                    topmost_retired.insert(node, ret_root);
                    if retired.contains(node) && !committed.contains(node) {
                        retired_root.insert(node, ret_root);
                    }
                }
                let (pn, pi, pg) = best_min.get(parent).copied().unwrap_or((None, None, None));
                let inh = (
                    pn.filter(|&e| e <= fc),
                    pi.filter(|&e| e <= fc),
                    pg.filter(|&e| e <= fc),
                );
                inherited_min.insert(node, inh);
                best_min.insert(
                    node,
                    (
                        min_opt(own.0, inh.0),
                        min_opt(own.1, inh.1),
                        min_opt(own.2, inh.2),
                    ),
                );
            } else {
                if retired.contains(node) && !committed.contains(node) {
                    retired_root.insert(node, node);
                }
                best_min.insert(node, own);
            }
        });

        Ok(Self {
            committed,
            euler,
            nearest_present,
            topmost_retired,
            inherited_min,
        })
    }

    pub(super) fn is_committed(&self, branch: &BranchId) -> bool {
        self.committed.contains(branch)
    }

    pub(super) fn is_recorded_ancestor(&self, branch: &BranchId, candidate: &BranchId) -> bool {
        branch != candidate
            && match (self.euler.get(candidate), self.euler.get(branch)) {
                (Some(&(c_in, c_out)), Some(&(b_in, b_out))) => c_in < b_in && b_out <= c_out,
                _ => false,
            }
    }

    pub(super) fn shares_delta_provenance(&self, branch: &BranchId, base: &BranchId) -> bool {
        if self.is_recorded_ancestor(branch, base) {
            return true;
        }
        match (
            self.nearest_present.get(branch),
            self.nearest_present.get(base),
        ) {
            (Some(&a), Some(&b)) => a == b,
            (None, None) => {
                let top_a = self.topmost_retired.get(branch);
                let top_b = self.topmost_retired.get(base);
                top_a.is_some() && top_a == top_b
            }
            _ => false,
        }
    }

    pub(super) fn inherited_requirements(
        &self,
        branch: &BranchId,
        head: u64,
    ) -> (bool, bool, bool) {
        let Some(&(n, i, g)) = self.inherited_min.get(branch) else {
            return (false, false, false);
        };
        (
            n.is_some_and(|at| at <= head),
            i.is_some_and(|at| at <= head),
            g.is_some_and(|at| at <= head),
        )
    }
}
