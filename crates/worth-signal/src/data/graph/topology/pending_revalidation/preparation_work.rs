//! Work for concrete waiter projection maps, NodeId sets and traversal storage.
use super::PendingRevalidationPreparationDenial;
use crate::data::retained_storage::{
    ordered_lookup_steps, RetainedStoragePreparation as Work,
    RetainedStoragePreparationDenial as Denial,
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
    // Fixed two-word NodeId keys: at most all keys plus navigation/root setup.
    reserve(work, entries.checked_add(1).and_then(|n| n.checked_mul(4)))
}

pub(crate) fn map_insert(
    work: &mut Work,
    entries: usize,
) -> Result<(), PendingRevalidationPreparationDenial> {
    // std BTree insertion moves initialized fixed-size key/value slots and
    // child pointers, never clones a projection's pending-producer vector.
    // At most one old node per entry plus a split and new root; 64 slots/node
    // covers the installed 11 keys, 11 values and 12 children plus navigation.
    reserve(work, entries.checked_add(2).and_then(|n| n.checked_mul(64)))
}

pub(crate) fn bucket_edit(
    work: &mut Work,
    entries: usize,
) -> Result<(), PendingRevalidationPreparationDenial> {
    // im15.1 NodeId sets: bound path copying, sibling borrow/merge and search.
    // Allow 16 times the 64-slot lookup bound per level, or all initialized
    // nodes for tiny sets. This bounds work, not retained allocation custody.
    reserve(
        work,
        entries
            .checked_add(1)
            .and_then(|n| n.checked_mul(128))
            .map(|n| n.min(16 * ordered_lookup_steps(entries))),
    )
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
