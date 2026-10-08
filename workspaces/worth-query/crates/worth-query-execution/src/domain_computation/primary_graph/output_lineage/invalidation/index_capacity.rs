//! Structural forecasts for the pinned im 15.1 ordered index, without the pool
//! feature. These are upper bounds, not observed allocation or copied-node counts.

use std::mem::{align_of, size_of};
use std::sync::Arc;

pub(in crate::domain_computation::primary_graph) fn arc_bytes<T>() -> Option<u64> {
    let alignment = align_of::<T>().max(align_of::<usize>());
    let header = size_of::<usize>().checked_mul(2)?;
    let body_offset = header.checked_add(alignment - 1)? / alignment * alignment;
    let bytes = body_offset
        .checked_add(size_of::<T>())?
        .checked_add(alignment - 1)?
        / alignment
        * alignment;
    u64::try_from(bytes).ok()
}

/// im's B-tree has 64 key/value slots and 65 optional Arc children. The two
/// chunks carry four usize bounds; Arc's header carries two more. Alignment
/// padding is bounded for each region. Arc keys/values keep payload allocation
/// out of node copies; their first construction is admitted separately.
fn ordered_node_bytes<K, V>() -> Option<u64> {
    let alignment = align_of::<(K, V)>().max(align_of::<usize>());
    let bytes = size_of::<(K, V)>()
        .checked_mul(64)?
        .checked_add(size_of::<Option<Arc<()>>>().checked_mul(65)?)?
        .checked_add(size_of::<usize>().checked_mul(6)?)?
        .checked_add(alignment.checked_mul(4)?)?;
    u64::try_from(bytes).ok()
}

/// A completed im 15.1 tree owns one root, even when empty. Splitting leaves
/// 32 keys in each child; deletion can leave 31, and rebalances before a
/// further descent. Every nonroot therefore owns at least 31 keys. Roots
/// sharing physical nodes share the allocation owner's reservation.
pub(super) fn retained_map_bytes<K, V>(entries: usize) -> Option<u64> {
    let nodes = 1usize.checked_add(entries.saturating_sub(1) / 31)?;
    ordered_node_bytes::<K, V>()?.checked_mul(u64::try_from(nodes).ok()?)
}

/// Nested maps and sets each own a root, including empty and singleton trees.
/// Summing their nonroot bounds is no greater than total entries / 31. A
/// forest must never be priced as one tree just because its keys are counted
/// together. `roots` and `entries` may conservatively overestimate the forest.
pub(super) fn retained_forest_bytes<K, V>(entries: usize, roots: usize) -> Option<u64> {
    if roots == 0 && entries != 0 {
        return None;
    }
    let nodes = roots.checked_add(entries / 31)?;
    ordered_node_bytes::<K, V>()?.checked_mul(u64::try_from(nodes).ok()?)
}

/// All newly allocated Arc nodes beside a retained predecessor. Insertion
/// copies one path and can allocate two split children at each level, then
/// one root. Deletion can copy a child and donor and materialize a replacement
/// at each level; merge/collapse returns its Node by value. Both fit 3H + 1.
/// This bounds transient allocation as well as the copied nodes retained by
/// the successor; old tickets remain charged until their final owner drops.
pub(super) fn ordered_edit_bytes<K, V>(entries: usize) -> Option<u64> {
    let prospective = entries.checked_add(1)?;
    let levels = ordered_height(prospective)?;
    ordered_node_bytes::<K, V>()?
        .checked_mul(u64::try_from(levels.checked_mul(3)?.checked_add(1)?).ok()?)
}

fn ordered_height(entries: usize) -> Option<usize> {
    let mut levels = 1usize;
    // A nonroot has 31 keys and 32 children. With a one-key root, height H
    // requires at least 2 * 32^(H - 1) - 1 keys: 63, 2047, 65535, ...
    let mut next_minimum = 63usize;
    while entries >= next_minimum {
        levels = levels.checked_add(1)?;
        let Some(next) = next_minimum
            .checked_add(1)
            .and_then(|minimum| minimum.checked_mul(32))
            .and_then(|minimum| minimum.checked_sub(1))
        else {
            break;
        };
        next_minimum = next;
    }
    Some(levels)
}

/// Binary search within at most 64 initialized entries plus navigation at
/// each level. Key comparison payload work is admitted by the key owner.
pub(super) fn ordered_navigation_work(entries: usize) -> Option<u64> {
    // Pinned im 15.1 splits into 32-key children; removal can leave 31.
    // A nonroot node therefore has at least 31 keys and 32 children. With
    // a one-key root, a height h needs at least 2 * 32^(h - 1) - 1 keys.
    // Counting bit length as height priced a one-node index as many levels.
    let levels = ordered_height(entries)?;
    let slots = entries.min(64);
    let comparisons = usize::BITS as usize - slots.leading_zeros() as usize + 1;
    u64::try_from(
        comparisons
            .checked_add(1)?
            .checked_mul(2)?
            .checked_mul(levels)?,
    )
    .ok()
}

/// Removal may look ahead through a child before descending to rebalance it.
/// Keep the former conservative navigation allowance for those repeated paths;
/// read and insertion bounds must not inherit this deletion-only cost.
pub(super) fn ordered_removal_work(entries: usize) -> Option<u64> {
    let levels = (usize::BITS as usize - entries.leading_zeros() as usize).max(1);
    let slots = entries.min(64);
    let comparisons = usize::BITS as usize - slots.leading_zeros() as usize + 1;
    u64::try_from(
        comparisons
            .checked_add(1)?
            .checked_mul(2)?
            .checked_mul(levels)?,
    )
    .ok()
}

#[cfg(all(test, feature = "allocation-probes"))]
mod allocation_tests;
#[cfg(test)]
mod navigation_tests;

#[cfg(test)]
mod retention_tests;

#[cfg(test)]
mod insertion_tests;
