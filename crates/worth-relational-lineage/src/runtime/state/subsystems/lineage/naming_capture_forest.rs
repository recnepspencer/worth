//! Linear `O(V + E)` fork-forest construction, retired-parent sibling grouping,
//! combined live+retired Euler-tour ancestor intervals (`O(1)` provenance
//! checks), and cutoff-proximity ranking for `CHK24` delta sealing.

use std::collections::{BTreeMap, BTreeSet};

use crate::contracts::history::data::coordinates::{BranchId, CommitId};
use crate::lineage::naming_pages::store::ColdNamingStore;
use crate::lineage::naming_pages::RetiredBranchFork;

use super::{FrozenBranch, FrozenLineageNaming};

pub(super) struct ForkForest {
    pub(super) cold_entries: BTreeMap<BranchId, usize>,
    pub(super) all_branches: BTreeSet<BranchId>,
    pub(super) parent_of: BTreeMap<BranchId, BranchId>,
    pub(super) fork_cutoff: BTreeMap<BranchId, CommitId>,
    pub(super) children: BTreeMap<BranchId, Vec<(CommitId, BranchId)>>,
    pub(super) roots: Vec<BranchId>,
    /// Sibling groups sharing a retired ancestor with no live ancestor above
    /// it; kept strictly separate from `children` so a group member's own
    /// descendants can never treat another group member as a sibling base.
    pub(super) retired_sibling_groups: Vec<Vec<BranchId>>,
    /// Retired intermediate fork edges referenced by live branch ancestry.
    pub(super) retired_forks: Vec<RetiredBranchFork>,
    /// Combined live+retired Euler-tour `(tin, tout)` intervals for `O(1)`
    /// strict-ancestor tests.
    euler: BTreeMap<BranchId, (u32, u32)>,
    present_anc: BTreeMap<BranchId, BranchId>,
    top_retired_anc: BTreeMap<BranchId, BranchId>,
    ancestor_delta_base: BTreeMap<BranchId, BranchId>,
    nearest_skip_anchor: BTreeMap<BranchId, BranchId>,
    root_of: BTreeMap<BranchId, BranchId>,
}

impl ForkForest {
    pub(super) fn is_strict_ancestor(&self, ancestor: &BranchId, node: &BranchId) -> bool {
        if ancestor == node {
            return false;
        }
        match (self.euler.get(ancestor), self.euler.get(node)) {
            (Some(&(a_in, a_out)), Some(&(n_in, n_out))) => a_in < n_in && n_out <= a_out,
            _ => false,
        }
    }

    /// Exact read-free delta provenance predicate matching
    /// `manifest_validation/naming/ancestry.rs::AncestryIndex::shares_delta_provenance`
    /// in `O(1)` via the combined live+retired Euler tour and memoized ancestors.
    pub(super) fn shares_delta_provenance(&self, branch: &BranchId, base: &BranchId) -> bool {
        if branch == base
            || !self.all_branches.contains(branch)
            || !self.all_branches.contains(base)
        {
            return false;
        }
        if self.is_strict_ancestor(base, branch) {
            return true;
        }
        match (self.present_anc.get(branch), self.present_anc.get(base)) {
            (Some(a), Some(b)) => a == b,
            (None, None) => {
                let top_a = self.top_retired_anc.get(branch);
                let top_b = self.top_retired_anc.get(base);
                top_a.is_some() && top_a == top_b
            }
            _ => false,
        }
    }

    pub(super) fn collect_shallow_ancestors(
        &self,
        branch: &BranchId,
        start: &BranchId,
        depths: &BTreeMap<BranchId, usize>,
        max_depth: usize,
        out: &mut Vec<BranchId>,
    ) {
        if !self.children.contains_key(branch)
            && let Some(prev) = self
                .ancestor_delta_base
                .get(start)
                .or_else(|| self.parent_of.get(start))
            && depths.get(prev).is_some_and(|&d| d < max_depth)
        {
            out.push(prev.clone());
        }
        if let Some(anchor) = self.nearest_skip_anchor.get(start)
            && depths.get(anchor).is_some_and(|&d| d < max_depth)
        {
            out.push(anchor.clone());
        }
        if let Some(root) = self.root_of.get(start)
            && depths.get(root).is_some_and(|&d| d < max_depth)
        {
            out.push(root.clone());
        }
    }

    pub(super) fn record_sealed_anchors(
        &mut self,
        branch: &BranchId,
        delta_base: Option<&BranchId>,
        depth: usize,
        skip_anchor_max_depth: usize,
    ) {
        let is_anc_delta = delta_base.is_some_and(|base| self.is_strict_ancestor(base, branch));
        if is_anc_delta && let Some(base) = delta_base {
            self.ancestor_delta_base
                .insert(branch.clone(), base.clone());
        }
        let is_skip = depth == 0
            || (depth <= skip_anchor_max_depth
                && is_anc_delta
                && delta_base != self.parent_of.get(branch));
        if is_skip {
            self.nearest_skip_anchor
                .insert(branch.clone(), branch.clone());
        } else if let Some(parent) = self.parent_of.get(branch)
            && let Some(anchor) = self.nearest_skip_anchor.get(parent).cloned()
        {
            self.nearest_skip_anchor.insert(branch.clone(), anchor);
        }
        if depth == 0 {
            self.root_of.insert(branch.clone(), branch.clone());
        } else if let Some(parent) = self.parent_of.get(branch)
            && let Some(r) = self.root_of.get(parent).cloned()
        {
            self.root_of.insert(branch.clone(), r);
        }
    }
}

pub(super) fn build_fork_forest(
    frozen: &FrozenLineageNaming,
    cold: Option<&ColdNamingStore>,
    all_branches: &BTreeSet<BranchId>,
) -> ForkForest {
    let cold_entries = cold
        .map(|store| {
            store
                .state()
                .branches
                .iter()
                .map(|r| {
                    (
                        r.branch.clone(),
                        r.pages.iter().map(|p| p.entries as usize).sum(),
                    )
                })
                .collect()
        })
        .unwrap_or_default();

    // Single cycle-aware visited-guarded pass (`O(V + E)`) over reachable ancestry.
    let mut color: BTreeMap<BranchId, u8> = BTreeMap::new();
    let mut retired_edges: BTreeMap<BranchId, RetiredBranchFork> = BTreeMap::new();
    let mut combined_edges: BTreeMap<BranchId, (BranchId, CommitId)> = BTreeMap::new();
    let mut path: Vec<BranchId> = Vec::new();
    for branch in all_branches {
        if color.get(branch).copied().unwrap_or(0) != 0 {
            continue;
        }
        path.clear();
        let mut cursor = branch.clone();
        loop {
            match color.get(&cursor).copied().unwrap_or(0) {
                1 | 2 => break,
                _ => {
                    color.insert(cursor.clone(), 1);
                    path.push(cursor.clone());
                    let Some((parent, fc)) = frozen.branch_forks.get(&cursor) else {
                        break;
                    };
                    if color.get(parent).copied().unwrap_or(0) == 1 {
                        break;
                    }
                    combined_edges.insert(cursor.clone(), (parent.clone(), *fc));
                    if !all_branches.contains(&cursor) {
                        retired_edges.insert(
                            cursor.clone(),
                            RetiredBranchFork {
                                branch: cursor.clone(),
                                parent: parent.clone(),
                                fork_commit: fc.0,
                            },
                        );
                    }
                    cursor = parent.clone();
                }
            }
        }
        for node in &path {
            color.insert(node.clone(), 2);
        }
    }

    // Build top-down adjacency over the full combined live + retired forest.
    let mut combined_kids: BTreeMap<BranchId, Vec<(BranchId, CommitId)>> = BTreeMap::new();
    let mut combined_roots: BTreeSet<BranchId> = BTreeSet::new();
    for (child, (parent, fc)) in &combined_edges {
        combined_kids
            .entry(parent.clone())
            .or_default()
            .push((child.clone(), *fc));
        if !combined_edges.contains_key(parent) {
            combined_roots.insert(parent.clone());
        }
    }
    for branch in all_branches {
        if !combined_edges.contains_key(branch) {
            combined_roots.insert(branch.clone());
        }
    }

    // Single iterative DFS (`O(V + E)`): Euler tour + nearest live/present/topmost-retired.
    let mut live_anc: BTreeMap<BranchId, (BranchId, CommitId)> = BTreeMap::new();
    let mut present_anc: BTreeMap<BranchId, BranchId> = BTreeMap::new();
    let mut top_retired_anc: BTreeMap<BranchId, BranchId> = BTreeMap::new();
    let mut retired_root: BTreeMap<BranchId, BranchId> = BTreeMap::new();
    let euler =
        worth_relational_branch::internal_port::history::access::ancestry_walk::ancestry_intervals(
            &combined_roots,
            &combined_kids,
            |node, parent_edge| {
                if let Some((parent, fc)) = parent_edge {
                    if all_branches.contains(parent) && frozen.branches.contains_key(parent) {
                        live_anc.insert(node.clone(), (parent.clone(), fc));
                    } else if let Some((anc, min_fc)) = live_anc.get(parent) {
                        live_anc.insert(node.clone(), (anc.clone(), (*min_fc).min(fc)));
                    }
                    if all_branches.contains(parent) {
                        present_anc.insert(node.clone(), parent.clone());
                    } else if let Some(anc) = present_anc.get(parent) {
                        present_anc.insert(node.clone(), anc.clone());
                    }
                    if let Some(ret_root) = retired_root.get(parent).cloned() {
                        top_retired_anc.insert(node.clone(), ret_root.clone());
                        if !all_branches.contains(node) {
                            retired_root.insert(node.clone(), ret_root);
                        }
                    }
                } else if !all_branches.contains(node) {
                    retired_root.insert(node.clone(), node.clone());
                }
            },
        );

    let mut parent_of = BTreeMap::new();
    let mut fork_cutoff = BTreeMap::new();
    let mut children: BTreeMap<BranchId, Vec<(CommitId, BranchId)>> = BTreeMap::new();
    let mut retired_groups: BTreeMap<BranchId, Vec<(CommitId, BranchId)>> = BTreeMap::new();
    let mut roots = Vec::new();
    for branch in all_branches {
        if let Some((live_parent, eff_fc)) = live_anc.get(branch) {
            parent_of.insert(branch.clone(), live_parent.clone());
            fork_cutoff.insert(branch.clone(), *eff_fc);
            children
                .entry(live_parent.clone())
                .or_default()
                .push((*eff_fc, branch.clone()));
        } else if let Some(top_retired) = top_retired_anc.get(branch) {
            let fc = combined_edges
                .get(branch)
                .map_or(CommitId(0), |(_, fc)| *fc);
            fork_cutoff.insert(branch.clone(), fc);
            retired_groups
                .entry(top_retired.clone())
                .or_default()
                .push((fc, branch.clone()));
        } else {
            roots.push(branch.clone());
        }
    }
    let mut retired_sibling_groups = Vec::new();
    for (_, mut siblings) in retired_groups {
        siblings.sort_unstable();
        retired_sibling_groups.push(siblings.into_iter().map(|(_, b)| b).collect::<Vec<_>>());
    }
    for list in children.values_mut() {
        list.sort_unstable();
    }
    ForkForest {
        cold_entries,
        all_branches: all_branches.clone(),
        parent_of,
        fork_cutoff,
        children,
        roots,
        retired_sibling_groups,
        retired_forks: retired_edges.into_values().collect(),
        euler,
        present_anc,
        top_retired_anc,
        ancestor_delta_base: BTreeMap::new(),
        nearest_skip_anchor: BTreeMap::new(),
        root_of: BTreeMap::new(),
    }
}

pub(super) fn candidate_rank(
    parent_of: &BTreeMap<BranchId, BranchId>,
    fork_cutoff: &BTreeMap<BranchId, CommitId>,
    branch: &BranchId,
    fb: &FrozenBranch,
    cand: &BranchId,
    fc: &FrozenBranch,
    depth: usize,
) -> (u64, bool, usize, usize, BranchId) {
    let bh = fb.latest_commit();
    let ch = fc.latest_commit();
    let b_parent = parent_of.get(branch);
    let cutoff_dist = if b_parent == Some(cand) {
        fork_cutoff
            .get(branch)
            .map_or_else(|| bh.abs_diff(ch), |bf| ch.saturating_sub(bf.0))
    } else if b_parent == parent_of.get(cand)
        && let (Some(bf), Some(cf)) = (fork_cutoff.get(branch), fork_cutoff.get(cand))
    {
        bf.0.abs_diff(cf.0)
    } else {
        bh.abs_diff(ch)
    };
    let cand_dirty = b_parent != Some(cand) && fork_cutoff.get(cand).is_some_and(|cf| ch > cf.0);
    (
        cutoff_dist,
        cand_dirty,
        fb.delta_len().abs_diff(fc.delta_len()),
        depth,
        cand.clone(),
    )
}
