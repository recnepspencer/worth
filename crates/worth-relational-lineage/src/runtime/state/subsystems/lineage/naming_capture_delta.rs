//! Fork-delta `CHK24` naming sealer (`O(names + fork deltas)`).

use std::collections::{BTreeMap, BTreeSet};
use worth_relational_storage::internal_port::storage::residency::chunked::member_io::ChunkMemberSink;

use crate::contracts::history::data::coordinates::BranchId;
use crate::contracts::storage::data::storage_unavailable::RelationalStorageUnavailableCause as Cause;
use crate::lineage::naming_pages::builder::{seal_pages, seal_pages_with_ops};
use crate::lineage::naming_pages::store::ColdNamingStore;
use crate::lineage::naming_pages::{BranchNamingRoot, RetiredBranchFork};

use super::carry::{
    build_retired_prior_reps, surviving_prior_base, try_carry_peer_prior_delta,
    try_carry_prior_delta,
};
use super::diff::{alias_headers_compatible, ops_have_domain, shared_cold_for};
use super::forest::{build_fork_forest, candidate_rank, ForkForest};
use super::{FrozenBranch, FrozenLineageNaming};

const MAX_SEAL_DELTA_DEPTH: usize = 8;
const SKIP_ANCHOR_MAX_DEPTH: usize = MAX_SEAL_DELTA_DEPTH - 3;

struct SealContext<'a> {
    frozen: &'a FrozenLineageNaming,
    cold: Option<&'a ColdNamingStore>,
    forest: ForkForest,
    retired_prior_reps: BTreeMap<BranchId, BranchId>,
    sealed: BTreeMap<BranchId, BranchNamingRoot>,
    depths: BTreeMap<BranchId, usize>,
    expanded_subtrees: BTreeSet<BranchId>,
    diff_visits: usize,
}

impl<'a> SealContext<'a> {
    fn new(frozen: &'a FrozenLineageNaming, all: &BTreeSet<BranchId>) -> (Self, Vec<BranchId>) {
        let cold = frozen.cold.as_ref().map(|p| p.store().as_ref());
        let mut forest = build_fork_forest(frozen, cold, all);
        let retired_prior_reps = build_retired_prior_reps(frozen, cold, all);
        let roots = std::mem::take(&mut forest.roots);
        (
            Self {
                frozen,
                cold,
                forest,
                retired_prior_reps,
                sealed: BTreeMap::new(),
                depths: BTreeMap::new(),
                expanded_subtrees: BTreeSet::new(),
                diff_visits: 0,
            },
            roots,
        )
    }

    fn full_cost(&self, fb: &FrozenBranch) -> usize {
        let base_cold = fb
            .unified_base()
            .flatten()
            .and_then(|link| self.cold.and_then(|s| link.root_in(s)))
            .map_or(0, |r| {
                let own = self
                    .forest
                    .cold_entries
                    .get(&r.branch)
                    .copied()
                    .unwrap_or(0);
                if r.delta_base.is_some() {
                    own.saturating_add(1).max(usize::MAX / 4)
                } else {
                    own
                }
            });
        fb.delta_len().saturating_add(base_cold)
    }

    fn record_sealed(&mut self, branch: BranchId, root: BranchNamingRoot, depth: usize) {
        self.forest.record_sealed_anchors(
            &branch,
            root.delta_base.as_ref(),
            depth,
            SKIP_ANCHOR_MAX_DEPTH,
        );
        self.sealed.insert(branch.clone(), root);
        self.depths.insert(branch, depth);
    }

    fn seal_branch(
        &mut self,
        branch: &BranchId,
        candidates: &[Option<&BranchId>],
        sink: &mut dyn ChunkMemberSink,
    ) -> Result<(), Cause> {
        if self.sealed.contains_key(branch) {
            return Ok(());
        }
        let Some(fb) = self.frozen.branches.get(branch) else {
            let empty = BranchNamingRoot {
                branch: branch.clone(),
                naming: None,
                index: None,
                groups: None,
                pages: Vec::new(),
                delta_base: None,
            };
            self.record_sealed(branch.clone(), empty, 0);
            return Ok(());
        };
        if let Some((root, depth)) = try_carry_prior_delta(
            self.frozen,
            self.cold,
            &self.forest,
            &self.sealed,
            &self.depths,
            MAX_SEAL_DELTA_DEPTH,
            branch,
            fb,
            sink,
        )? {
            self.record_sealed(branch.clone(), root, depth);
            return Ok(());
        }
        let unified = fb.unified_base();
        let mut best_cost = self.full_cost(fb);
        let mut best: Option<(
            BranchId,
            usize,
            Option<(&'a ColdNamingStore, &'a BranchNamingRoot)>,
        )> = None;
        if let Some(b_base) = unified {
            let mut raw: Vec<BranchId> = Vec::new();
            for cand in candidates.iter().flatten().copied() {
                if cand == branch {
                    continue;
                }
                if self
                    .depths
                    .get(cand)
                    .is_some_and(|&d| d < MAX_SEAL_DELTA_DEPTH)
                {
                    raw.push(cand.clone());
                } else if self.forest.parent_of.get(branch) == Some(cand) {
                    self.forest.collect_shallow_ancestors(
                        branch,
                        cand,
                        &self.depths,
                        MAX_SEAL_DELTA_DEPTH,
                        &mut raw,
                    );
                }
            }
            let mut ranked = Vec::new();
            let mut seen = BTreeSet::new();
            for cand in raw {
                if !seen.insert(cand.clone()) || !self.forest.shares_delta_provenance(branch, &cand)
                {
                    continue;
                }
                let Some(&d) = self
                    .depths
                    .get(&cand)
                    .filter(|&&d| d < MAX_SEAL_DELTA_DEPTH)
                else {
                    continue;
                };
                let Some(fc) = self.frozen.branches.get(&cand) else {
                    continue;
                };
                if fb.has_active_index()
                    && fc.has_active_index()
                    && fc.index_retention_floor() > fb.index_retention_floor()
                {
                    continue;
                }
                ranked.push(candidate_rank(
                    &self.forest.parent_of,
                    &self.forest.fork_cutoff,
                    branch,
                    fb,
                    &cand,
                    fc,
                    d,
                ));
            }
            ranked.sort_unstable();
            for (_, _, _, cand_depth, cand) in ranked {
                if best_cost == 0 {
                    break;
                }
                let Some(fc) = self.frozen.branches.get(&cand) else {
                    continue;
                };
                let Some(c_base) = fc.unified_base() else {
                    continue;
                };
                let Some(shared_cold) = shared_cold_for(self.cold, b_base, c_base) else {
                    continue;
                };
                if let Some(est) = fb.count_diff_against_bounded(
                    fc,
                    shared_cold.is_some(),
                    best_cost - 1,
                    &mut self.diff_visits,
                ) {
                    best_cost = est;
                    best = Some((cand, cand_depth + 1, shared_cold));
                }
            }
        }
        let mut root = fb.headers(branch.clone());
        let depth = if let Some((base_id, d, shared_cold)) = best
            && let Some(fc) = self.frozen.branches.get(&base_id)
            && let Some(base_sealed) = self.sealed.get(&base_id)
            && let Some(mut ops) = fb.diff_against_bounded(fc, shared_cold, best_cost)?
            && alias_headers_compatible(&root, base_sealed, |dom| ops_have_domain(&ops, dom))
        {
            ops.sort_unstable_by(|a, b| a.0.cmp(&b.0));
            root.pages = seal_pages_with_ops(None, ops, sink)?;
            root.delta_base = Some(base_id);
            d
        } else {
            let (prior, mut changes) = match unified {
                Some(Some(link)) => {
                    let store = self.cold.ok_or(Cause::SourceUnavailable)?;
                    match link.root_in(store) {
                        Some(r) if r.delta_base.is_none() && r.naming.is_none() => {
                            (Some((store, r.pages.as_slice())), fb.encoded_changes())
                        }
                        _ => (None, fb.collect_encoded_in(self.cold)?),
                    }
                }
                Some(None) => (None, fb.encoded_changes()),
                None => (None, fb.collect_encoded_in(self.cold)?),
            };
            changes.sort_unstable_by(|a, b| a.0.cmp(&b.0));
            root.pages = seal_pages(prior, changes, sink)?;
            0
        };
        self.record_sealed(branch.clone(), root, depth);
        Ok(())
    }

    fn seal_prior_chain(
        &mut self,
        branch: &BranchId,
        all_branches: &BTreeSet<BranchId>,
        visiting: &mut BTreeSet<BranchId>,
        sink: &mut dyn ChunkMemberSink,
    ) -> Result<(), Cause> {
        if self.sealed.contains_key(branch) || !visiting.insert(branch.clone()) {
            return Ok(());
        }
        if let Some(store) = self.cold
            && let Some(prev_root) = store.state().root(branch)
        {
            if let Some(prev_base) = prev_root.delta_base.as_ref()
                && !all_branches.contains(prev_base)
                && let Some(rep) = self.retired_prior_reps.get(prev_base).cloned()
                && &rep != branch
            {
                self.seal_prior_chain(&rep, all_branches, visiting, sink)?;
                if let Some((root, depth)) = try_carry_peer_prior_delta(
                    self.frozen,
                    store,
                    &self.forest,
                    &self.sealed,
                    &self.depths,
                    MAX_SEAL_DELTA_DEPTH,
                    branch,
                    &rep,
                    sink,
                )? {
                    self.record_sealed(branch.clone(), root, depth);
                    visiting.remove(branch);
                    return Ok(());
                }
            }
            let is_retired_rep = prev_root
                .delta_base
                .as_ref()
                .is_some_and(|pb| self.retired_prior_reps.get(pb) == Some(branch));
            if let Some(b) = surviving_prior_base(store, branch, all_branches) {
                self.seal_prior_chain(&b, all_branches, visiting, sink)?;
                self.seal_branch(branch, &[Some(&b)], sink)?;
            } else if prev_root.delta_base.is_none() || is_retired_rep {
                self.seal_branch(branch, &[], sink)?;
            }
        }
        visiting.remove(branch);
        Ok(())
    }

    fn seal_subtree(
        &mut self,
        parent: &BranchId,
        sink: &mut dyn ChunkMemberSink,
    ) -> Result<(), Cause> {
        if !self.expanded_subtrees.insert(parent.clone()) {
            return Ok(());
        }
        let Some(list) = self.forest.children.get(parent).cloned() else {
            return Ok(());
        };
        let slice: Vec<BranchId> = list.into_iter().map(|(_, b)| b).collect();
        self.seal_sibling_slice(&slice, Some(parent), None, Some(parent), sink)
    }

    fn seal_sibling_slice(
        &mut self,
        slice: &[BranchId],
        parent: Option<&BranchId>,
        left_anchor: Option<&BranchId>,
        right_anchor: Option<&BranchId>,
        sink: &mut dyn ChunkMemberSink,
    ) -> Result<(), Cause> {
        if slice.is_empty() {
            return Ok(());
        }
        let cap = |anchor: Option<&BranchId>| {
            anchor
                .and_then(|a| self.depths.get(a))
                .map_or(0, |&d| MAX_SEAL_DELTA_DEPTH.saturating_sub(d))
        };
        let (cap_l, cap_r) = (cap(left_anchor), cap(right_anchor));
        if slice.len() <= cap_l + cap_r && cap_l + cap_r > 0 {
            let split = if cap_l == 0 {
                0
            } else if cap_r == 0 {
                slice.len()
            } else {
                slice
                    .len()
                    .saturating_sub(cap_r)
                    .max(slice.len() / 2)
                    .min(cap_l)
            };
            let mut prev = left_anchor;
            for b in &slice[..split] {
                self.seal_branch(b, &[prev, right_anchor, parent], sink)?;
                prev = Some(b);
                self.seal_subtree(b, sink)?;
            }
            let mut next = right_anchor;
            for b in slice[split..].iter().rev() {
                self.seal_branch(b, &[next, left_anchor, parent], sink)?;
                next = Some(b);
                self.seal_subtree(b, sink)?;
            }
            return Ok(());
        }
        let mid = slice.len() / 2;
        let b = &slice[mid];
        self.seal_branch(b, &[left_anchor, right_anchor, parent], sink)?;
        self.seal_subtree(b, sink)?;
        self.seal_sibling_slice(&slice[..mid], parent, left_anchor, Some(b), sink)?;
        self.seal_sibling_slice(&slice[mid + 1..], parent, Some(b), right_anchor, sink)
    }
}

pub(super) fn seal_all_branches(
    frozen: &FrozenLineageNaming,
    all_branches: &BTreeSet<BranchId>,
    sink: &mut dyn ChunkMemberSink,
) -> Result<(Vec<BranchNamingRoot>, Vec<RetiredBranchFork>, usize), Cause> {
    let (mut ctx, roots) = SealContext::new(frozen, all_branches);
    let retired_groups = std::mem::take(&mut ctx.forest.retired_sibling_groups);
    let mut visiting = BTreeSet::new();
    for branch in all_branches {
        ctx.seal_prior_chain(branch, all_branches, &mut visiting, sink)?;
    }
    for root in &roots {
        ctx.seal_branch(root, &[], sink)?;
        ctx.seal_subtree(root, sink)?;
    }
    for group in &retired_groups {
        if let Some((first, rest)) = group.split_first() {
            ctx.seal_branch(first, &[], sink)?;
            ctx.seal_subtree(first, sink)?;
            ctx.seal_sibling_slice(rest, None, Some(first), None, sink)?;
        }
    }
    for branch in all_branches {
        if !ctx.sealed.contains_key(branch) {
            ctx.seal_branch(branch, &[], sink)?;
        }
        if !ctx.expanded_subtrees.contains(branch) {
            ctx.seal_subtree(branch, sink)?;
        }
    }
    Ok((
        ctx.sealed.into_values().collect(),
        ctx.forest.retired_forks,
        ctx.diff_visits,
    ))
}
