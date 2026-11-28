//! Prior-checkpoint delta carry and composed carry across retired C1 intermediates.

use std::collections::{BTreeMap, BTreeSet};

use worth_relational_storage::internal_port::storage::residency::chunked::member_io::ChunkMemberSink;

use crate::contracts::history::data::coordinates::BranchId;
use crate::contracts::storage::data::storage_unavailable::RelationalStorageUnavailableCause as Cause;
use crate::lineage::naming_pages::builder::{seal_pages_with_ops, PageChange};
use crate::lineage::naming_pages::store::ColdNamingStore;
use crate::lineage::naming_pages::tables::table_domain;
use crate::lineage::naming_pages::{page_for, BranchNamingRoot, MAX_DELTA_DEPTH};

use super::diff::alias_headers_compatible;
use super::forest::ForkForest;
use super::{FrozenBranch, FrozenLineageNaming};

/// Maps each retired C1 `delta_base` shared by surviving branches in
/// `all_branches` to its lightest surviving child representative so that the
/// representative rebases over the surviving ancestor ONCE and peer children
/// carry directly against that representative without repeating the shared
/// retired delta.
pub(super) fn build_retired_prior_reps(
    frozen: &FrozenLineageNaming,
    cold: Option<&ColdNamingStore>,
    all_branches: &BTreeSet<BranchId>,
) -> BTreeMap<BranchId, BranchId> {
    let Some(store) = cold else {
        return BTreeMap::new();
    };
    let mut best: BTreeMap<BranchId, ((usize, bool, Option<u64>, usize), BranchId)> =
        BTreeMap::new();
    for branch in all_branches {
        let Some(prev) = store.state().root(branch) else {
            continue;
        };
        let Some(prev_base) = prev.delta_base.as_ref() else {
            continue;
        };
        if all_branches.contains(prev_base) {
            continue;
        }
        let rank = match frozen.branches.get(branch) {
            Some(fb) => {
                let hdr = fb.headers(branch.clone());
                let active_domains = [0u8, 1, 2, 3]
                    .into_iter()
                    .filter(|&d| hdr.has_active_domain(d))
                    .count();
                let floor = hdr.index.and_then(|h| h.retention_floor);
                let own_weight = fb.delta_len().saturating_add(prev.pages.len());
                (
                    4usize.saturating_sub(active_domains),
                    !fb.has_active_index(),
                    floor,
                    own_weight,
                )
            }
            None => (4, true, Some(u64::MAX), usize::MAX / 2),
        };
        match best.get_mut(prev_base) {
            Some((best_rank, best_branch)) => {
                if (rank, branch) < (*best_rank, &*best_branch) {
                    *best_rank = rank;
                    *best_branch = branch.clone();
                }
            }
            None => {
                best.insert(prev_base.clone(), (rank, branch.clone()));
            }
        }
    }
    best.into_iter().map(|(mid, (_, rep))| (mid, rep)).collect()
}

/// Carries a branch directly against an already-sealed peer representative
/// `peer_base` that shared the exact same retired C1 `delta_base`, inspecting
/// ONLY the two peers' own C1 delta pages and post-C1 edits (never re-reading
/// or re-sealing the shared retired intermediate's delta pages).
#[allow(clippy::too_many_arguments)]
pub(super) fn try_carry_peer_prior_delta(
    frozen: &FrozenLineageNaming,
    store: &ColdNamingStore,
    forest: &ForkForest,
    sealed: &BTreeMap<BranchId, BranchNamingRoot>,
    depths: &BTreeMap<BranchId, usize>,
    max_depth: usize,
    branch: &BranchId,
    peer_base: &BranchId,
    sink: &mut dyn ChunkMemberSink,
) -> Result<Option<(BranchNamingRoot, usize)>, Cause> {
    if !forest.shares_delta_provenance(branch, peer_base) {
        return Ok(None);
    }
    let (Some(base_sealed), Some(&base_depth)) = (sealed.get(peer_base), depths.get(peer_base))
    else {
        return Ok(None);
    };
    if base_depth >= max_depth {
        return Ok(None);
    }
    let (Some(fb), Some(fp)) = (frozen.branches.get(branch), frozen.branches.get(peer_base)) else {
        return Ok(None);
    };
    if fb.has_active_index()
        && fp.has_active_index()
        && fp.index_retention_floor() > fb.index_retention_floor()
    {
        return Ok(None);
    }
    let (Some(Some(link_b)), Some(Some(link_p))) = (fb.unified_base(), fp.unified_base()) else {
        return Ok(None);
    };
    let (Some(prev_b), Some(prev_p)) = (
        link_b.root_in(store).filter(|r| &r.branch == branch),
        link_p.root_in(store).filter(|r| &r.branch == peer_base),
    ) else {
        return Ok(None);
    };
    if prev_b.delta_base.is_none() || prev_b.delta_base != prev_p.delta_base {
        return Ok(None);
    }
    let mut root = fb.headers(branch.clone());
    for dom in [0u8, 1, 2, 3] {
        if !root.has_active_domain(dom) {
            continue;
        }
        if !prev_b.has_active_domain(dom)
            || !prev_p.has_active_domain(dom)
            || !base_sealed.has_active_domain(dom)
        {
            return Ok(None);
        }
    }
    let b_changes: BTreeMap<_, _> = fb.encoded_changes().into_iter().collect();
    let p_changes: BTreeMap<_, _> = fp.encoded_changes().into_iter().collect();
    let mut keys: BTreeSet<Vec<u8>> = b_changes.keys().chain(p_changes.keys()).cloned().collect();
    for layer in [prev_b, prev_p] {
        for page_ref in &layer.pages {
            let Some(dom) = page_ref.first.first().copied().and_then(table_domain) else {
                continue;
            };
            if !root.has_active_domain(dom) {
                continue;
            }
            let page = store.page(page_ref)?;
            for (k, _) in &page.page.rows {
                keys.insert(k.clone());
            }
        }
    }
    let mut ops = Vec::new();
    for key in keys {
        let Some(dom) = key.first().copied().and_then(table_domain) else {
            continue;
        };
        if !root.has_active_domain(dom) {
            continue;
        }
        let val_b = match b_changes.get(&key) {
            Some(slot) => slot.clone(),
            None => store.get_root(prev_b, &key)?,
        };
        let val_p = match p_changes.get(&key) {
            Some(slot) => slot.clone(),
            None => store.get_root(prev_p, &key)?,
        };
        if val_b != val_p {
            match val_b {
                Some(bytes) => ops.push((key, PageChange::Put(bytes))),
                None => ops.push((key, PageChange::Tombstone)),
            }
        }
    }
    root.pages = seal_pages_with_ops(None, ops, sink)?;
    if !alias_headers_compatible(&root, base_sealed, |dom| {
        root.pages
            .iter()
            .any(|p| p.first.first().copied().and_then(table_domain) == Some(dom))
    }) {
        return Ok(None);
    }
    root.delta_base = Some(peer_base.clone());
    Ok(Some((root, base_depth + 1)))
}

/// Walks `branch`'s prior checkpoint `delta_base` chain through retired
/// intermediates to find the first surviving base in `all_branches`.
pub(super) fn surviving_prior_base(
    cold: &ColdNamingStore,
    branch: &BranchId,
    all_branches: &BTreeSet<BranchId>,
) -> Option<BranchId> {
    let mut cursor = cold.state().root(branch)?.delta_base.as_ref()?;
    let mut seen = BTreeSet::from([branch]);
    for _ in 0..=MAX_DELTA_DEPTH {
        if !seen.insert(cursor) {
            return None;
        }
        if all_branches.contains(cursor) {
            return Some(cursor.clone());
        }
        cursor = cold.state().root(cursor)?.delta_base.as_ref()?;
    }
    None
}

#[allow(clippy::too_many_arguments)]
pub(super) fn try_carry_prior_delta(
    frozen: &FrozenLineageNaming,
    cold: Option<&ColdNamingStore>,
    forest: &ForkForest,
    sealed: &BTreeMap<BranchId, BranchNamingRoot>,
    depths: &BTreeMap<BranchId, usize>,
    max_depth: usize,
    branch: &BranchId,
    fb: &FrozenBranch,
    sink: &mut dyn ChunkMemberSink,
) -> Result<Option<(BranchNamingRoot, usize)>, Cause> {
    let (Some(store), Some(Some(link_b))) = (cold, fb.unified_base()) else {
        return Ok(None);
    };
    let Some(prev_b) = link_b.root_in(store).filter(|r| &r.branch == branch) else {
        return Ok(None);
    };
    let Some(mut cursor) = prev_b.delta_base.as_ref() else {
        return Ok(None);
    };
    let mut hop_chain: Vec<&BranchNamingRoot> = vec![prev_b];
    let mut seen = BTreeSet::from([&prev_b.branch]);
    let base_branch = loop {
        if hop_chain.len() > MAX_DELTA_DEPTH || !seen.insert(cursor) {
            return Ok(None);
        }
        if sealed.contains_key(cursor) {
            break cursor;
        }
        if forest.all_branches.contains(cursor) {
            return Ok(None);
        }
        let Some(mid) = store.state().root(cursor) else {
            return Ok(None);
        };
        hop_chain.push(mid);
        let Some(next) = mid.delta_base.as_ref() else {
            return Ok(None);
        };
        cursor = next;
    };
    if !forest.shares_delta_provenance(branch, base_branch) {
        return Ok(None);
    }
    let (Some(base_sealed), Some(&base_depth)) = (sealed.get(base_branch), depths.get(base_branch))
    else {
        return Ok(None);
    };
    if base_depth >= max_depth {
        return Ok(None);
    }
    let Some(fp) = frozen.branches.get(base_branch) else {
        return Ok(None);
    };
    if fb.has_active_index()
        && fp.has_active_index()
        && fp.index_retention_floor() > fb.index_retention_floor()
    {
        return Ok(None);
    }
    let Some(Some(link_p)) = fp.unified_base() else {
        return Ok(None);
    };
    let Some(prev_p) = link_p.root_in(store).filter(|r| &r.branch == base_branch) else {
        return Ok(None);
    };
    Ok(carry_prior_delta_root(
        branch,
        fb,
        fp,
        store,
        &hop_chain,
        prev_p,
        base_branch,
        base_sealed,
        sink,
    )?
    .map(|root| (root, base_depth + 1)))
}

/// Reconciles a previously delta-sealed branch against its surviving base root,
/// composing intermediate delta layers when C1 intermediates retired after reopen.
#[allow(clippy::too_many_arguments)]
fn carry_prior_delta_root(
    branch: &BranchId,
    fb: &FrozenBranch,
    fp: &FrozenBranch,
    store: &ColdNamingStore,
    hop_chain: &[&BranchNamingRoot],
    prev_p: &BranchNamingRoot,
    base_branch: &BranchId,
    base_sealed: &BranchNamingRoot,
    sink: &mut dyn ChunkMemberSink,
) -> Result<Option<BranchNamingRoot>, Cause> {
    let prev_b = hop_chain[0];
    if prev_b.naming.is_some() {
        // A legacy page run may still interleave the displaced naming tables.
        // Materialize the surviving tables before carrying this branch again.
        return Ok(None);
    }
    let composed = hop_chain.len() > 1;
    let mut root = fb.headers(branch.clone());
    for dom in [0u8, 1, 2, 3] {
        if !root.has_active_domain(dom) {
            continue;
        }
        let base_active = base_sealed.has_active_domain(dom);
        if !prev_b.has_active_domain(dom)
            || prev_p.has_active_domain(dom) != base_active
            || (base_active && hop_chain[1..].iter().any(|mid| !mid.has_active_domain(dom)))
        {
            return Ok(None);
        }
    }
    let b_changes: BTreeMap<_, _> = fb.encoded_changes().into_iter().collect();
    let p_changes: BTreeMap<_, _> = fp.encoded_changes().into_iter().collect();
    let mut keys: BTreeSet<Vec<u8>> = b_changes.keys().chain(p_changes.keys()).cloned().collect();
    if composed {
        for layer in hop_chain {
            for page_ref in &layer.pages {
                let Some(dom) = page_ref.first.first().copied().and_then(table_domain) else {
                    continue;
                };
                if !root.has_active_domain(dom) {
                    continue;
                }
                let page = store.page(page_ref)?;
                for (k, _) in &page.page.rows {
                    keys.insert(k.clone());
                }
            }
        }
    }
    let mut ops = Vec::new();
    for key in keys {
        let Some(dom) = key.first().copied().and_then(table_domain) else {
            continue;
        };
        if !root.has_active_domain(dom) {
            continue;
        }
        let val_b = match b_changes.get(&key) {
            Some(slot) => slot.clone(),
            None => store.get_root(prev_b, &key)?,
        };
        let val_p = if base_sealed.has_active_domain(dom) {
            match p_changes.get(&key) {
                Some(slot) => slot.clone(),
                None => store.get_root(prev_p, &key)?,
            }
        } else {
            None
        };
        if val_b == val_p {
            if !composed && page_for(&prev_b.pages, &key).is_some() {
                ops.push((key, PageChange::Remove));
            }
        } else if let Some(bytes) = val_b {
            ops.push((key, PageChange::Put(bytes)));
        } else {
            ops.push((key, PageChange::Tombstone));
        }
    }
    let prior = (!composed).then_some((store, prev_b.pages.as_slice()));
    root.pages = seal_pages_with_ops(prior, ops, sink)?;
    if !alias_headers_compatible(&root, base_sealed, |dom| {
        root.pages
            .iter()
            .any(|p| p.first.first().copied().and_then(table_domain) == Some(dom))
    }) {
        return Ok(None);
    }
    root.delta_base = Some(base_branch.clone());
    Ok(Some(root))
}
