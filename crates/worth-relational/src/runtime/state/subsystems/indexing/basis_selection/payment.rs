//! Native B-tree catalog insertions and carried im selection path copies.
use super::super::generation_catalog::insertion_quote::{add, insertion};
use super::{
    Arc, BranchId, DerivedIndexGeneration, Key, PreparedIndexBasisSelection, VersionId,
    VersionSelections,
};
use crate::indexes::data::DerivedIndexId;

pub(super) fn outer(entries: usize, width: u64) -> Option<(u64, u64)> {
    let mut quote = (0_u64, 0_u64);
    for _ in 0..2 {
        add(
            &mut quote,
            insertion::<Key, Arc<PreparedIndexBasisSelection>>(
                entries,
                usize::try_from(width).ok()?.checked_add(1)?,
            )?,
        )?;
        add(
            &mut quote,
            insertion::<VersionId, VersionSelections>(entries, 1)?,
        )?;
    }
    add(
        &mut quote,
        insertion::<Arc<BranchId>, Arc<PreparedIndexBasisSelection>>(
            entries,
            usize::try_from(width).ok()?.checked_add(1)?,
        )?,
    )?;
    quote.1 = quote.1.checked_add(width.checked_mul(4)?)?.checked_add(
        (std::mem::size_of::<PreparedIndexBasisSelection>()
            + std::mem::size_of::<BranchId>()
            + 4 * std::mem::size_of::<usize>()) as u64,
    )?;
    quote.1 = quote.1.checked_add(
        (std::mem::size_of::<super::super::IndexSelectionState>()
            + std::mem::size_of::<PreparedIndexBasisSelection>()
            + 4 * std::mem::size_of::<usize>()) as u64,
    )?;
    quote.0 = quote.0.checked_add(4)?;
    // Two FIFO insertions reuse the same now-owned append path. im's 64-slot
    // RRB geometry bounds its fringe copies by 5h+4 branches and four leaves.
    let mut queue_entries = entries;
    let mut height = 1_u64;
    while queue_entries > 64 {
        queue_entries = queue_entries.div_ceil(32);
        height += 1;
    }
    let queue_bytes = (5 * height + 4)
        .checked_mul((332 * std::mem::size_of::<usize>()) as u64)?
        .checked_add(
            (4 * (64 * std::mem::size_of::<Key>() + 6 * std::mem::size_of::<usize>())) as u64,
        )?;
    quote.0 = quote.0.checked_add((5 * height + 4) * 64)?;
    quote.1 = quote.1.checked_add(queue_bytes)?;
    Some(quote)
}

pub(super) fn inner(mut entries: usize) -> Option<(u64, u64)> {
    // im 15 has 64-slot nodes, half-full non-root levels. Copy, sibling,
    // rebalance/split at each level and two roots cover one removal/insertion.
    let mut height = 1_u64;
    while entries > 64 {
        entries = entries.div_ceil(32);
        height += 1;
    }
    let nodes = height.checked_mul(3)?.checked_add(2)?;
    let bytes = (64 * std::mem::size_of::<(DerivedIndexId, Arc<DerivedIndexGeneration>)>()
        + 71 * std::mem::size_of::<usize>()) as u64;
    // A reached global index updates the full selection and its global-only
    // carried view. Scoped updates reserve the same two-path upper bound.
    Some((
        nodes.checked_mul(128)?,
        nodes.checked_mul(bytes)?.checked_mul(2)?,
    ))
}
