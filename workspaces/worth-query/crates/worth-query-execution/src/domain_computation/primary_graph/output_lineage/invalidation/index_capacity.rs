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

/// Conservative retained capacity: every nonempty node owns at least one
/// entry. The allocation that owns the nodes holds the bound, so whatever
/// shares them shares it.
pub(super) fn retained_map_bytes<K, V>(entries: usize) -> Option<u64> {
    ordered_node_bytes::<K, V>()?.checked_mul(u64::try_from(entries.checked_add(1)?).ok()?)
}

/// One selected edit copies a search path. Minimum branching two bounds its
/// height by bit length; splitting can add two nodes per level and a new root.
/// Old and new roots coexist during preparation and visibility cutover.
pub(super) fn ordered_edit_bytes<K, V>(entries: usize) -> Option<u64> {
    let prospective = entries.checked_add(1)?;
    let levels = usize::BITS as usize - prospective.leading_zeros() as usize + 1;
    ordered_node_bytes::<K, V>()?
        .checked_mul(u64::try_from(levels.checked_mul(2)?.checked_add(1)?).ok()?)
}

/// Binary search within at most 64 initialized entries plus navigation at
/// each level. Key comparison payload work is admitted by the key owner.
pub(super) fn ordered_navigation_work(entries: usize) -> Option<u64> {
    // Pinned im 15.1 splits into 32-key children; removal can leave 31.
    // A nonroot node therefore has at least 31 keys and 32 children. With
    // a one-key root, a height h needs at least 2 * 32^(h - 1) - 1 keys.
    // Counting bit length as height priced a one-node index as many levels.
    let mut levels = 1usize;
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

#[cfg(test)]
mod navigation_tests;
