//! Work for concrete waiter projection maps, NodeId sets and traversal storage.
use super::PendingRevalidationPreparationDenial;
use crate::data::retained_storage::{
    RetainedStoragePreparation as Work, RetainedStoragePreparationDenial as Denial,
};

pub(crate) fn reserve(
    work: &mut Work,
    count: Option<usize>,
) -> Result<(), PendingRevalidationPreparationDenial> {
    let count = count.ok_or(Denial::WorkExhausted {
        maximum_visits: work.maximum_visits(),
    })?;
    work.reserve_visits(count)?;
    Ok(())
}

pub(crate) fn map_lookup(
    work: &mut Work,
    entries: usize,
) -> Result<(), PendingRevalidationPreparationDenial> {
    // Fixed two-word NodeId keys follow the ordered-map search path.
    reserve(work, btree_search_steps(entries).checked_mul(4))
}

pub(crate) fn map_insert(
    work: &mut Work,
    entries: usize,
) -> Result<(), PendingRevalidationPreparationDenial> {
    // Search follows the ordered-map path. One insertion can additionally
    // split the touched leaf and its ancestors; the fixed allowance covers
    // the initialized slots moved at those levels without charging the whole
    // map for every insertion.
    reserve(
        work,
        btree_search_steps(entries)
            .checked_mul(4)
            .and_then(|search| search.checked_add(64)),
    )
}

fn btree_search_steps(entries: usize) -> usize {
    entries
        .checked_ilog2()
        .map_or(1, |depth| depth as usize + 1)
}

pub(crate) fn bucket_edit(
    work: &mut Work,
    entries: usize,
) -> Result<(), PendingRevalidationPreparationDenial> {
    // im 15.1 nodes hold 64 entries and non-root nodes remain at least half
    // full. Charge every comparison/navigation slot on the longest possible
    // edit path. Using a binary-tree depth here turns a B-tree edit into a
    // fictitious quadratic traversal at ordinary fan-out.
    reserve(work, im_btree_edit_steps(entries))
}

fn im_btree_edit_steps(entries: usize) -> Option<usize> {
    let mut levels = 1_usize;
    let mut remaining = entries;
    while remaining > 64 {
        remaining = remaining.checked_add(31)?.checked_div(32)?;
        levels = levels.checked_add(1)?;
    }
    levels.checked_mul(128)
}

pub(crate) fn sequence_growth(
    work: &mut Work,
    existing: usize,
    added: usize,
) -> Result<(), PendingRevalidationPreparationDenial> {
    // The largest traversal entry is a pair of NodeIds. Account old moves and
    // new visits/copies before Vec growth; no capacity growth factor is assumed.
    reserve(
        work,
        existing.checked_add(added).and_then(|n| {
            n.checked_mul(std::mem::size_of::<(
                crate::data::handle::NodeId,
                crate::data::handle::NodeId,
            )>())
            .filter(|bytes| *bytes <= isize::MAX as usize)?;
            n.checked_add(added)?.checked_add(1)
        }),
    )
}
