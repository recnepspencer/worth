use std::sync::Arc;

use super::{RetainedStorageCharge as Charge, RetainedStoragePreparationDenial as Denial};

/// Lookup/seek steps for the installed std and im B-trees described below.
/// A balanced tree has at most bit_length(entries) nonempty levels (at least
/// two children per internal node). Allow 64 comparisons and 64 navigation
/// steps per level, including an empty root. This is a structural bound only:
/// callers must separately charge the concrete key's comparison cost.
pub(crate) fn ordered_lookup_steps(entries: usize) -> usize {
    128 * (usize::BITS as usize - entries.leading_zeros() as usize).max(1)
}

/// Conservative structural allocation charge for the installed im 15.1 B-tree
/// (no pool feature): nodes/btree.rs has 64 entry slots and 65 optional Arc
/// children. sized-chunks 0.6 uses two usize bounds per chunk. Every nonempty
/// node contains a key, so one node per entry plus the empty root is an upper
/// bound. Payload allocations are charged separately by their owners.
pub(crate) fn ordered_index_charge<K, V>(entries: usize) -> Result<Charge, Denial> {
    let alignment = std::mem::align_of::<(K, V)>().max(std::mem::align_of::<usize>());
    let node = Charge::capacity::<(K, V)>(64)?
        .checked_add(Charge::capacity::<Option<Arc<()>>>(65)?)?
        .checked_add(Charge::capacity::<usize>(6)?)?
        .checked_add(Charge::capacity::<u8>(alignment)?.checked_mul(4)?)?;
    node.checked_mul(entries.checked_add(1).ok_or(Denial::ChargeOverflow)?)
}

/// Conservative Rust 1.94 BTreeMap structural bound. alloc/collections/btree/
/// node.rs uses 11 key/value slots, 12 child pointers, a parent pointer and two
/// u16 fields. Charge every node as internal, with padding for each aligned
/// region, and at most one node per entry plus a possibly retained empty root.
/// Recheck this representation when changing the toolchain. This is retained
/// representation, not allocator bookkeeping or process RSS.
pub(crate) fn btree_structure_charge<K, V>(entries: usize) -> Result<Charge, Denial> {
    let alignment = std::mem::align_of::<K>()
        .max(std::mem::align_of::<V>())
        .max(std::mem::align_of::<usize>());
    Charge::capacity::<K>(11)?
        .checked_add(Charge::capacity::<V>(11)?)?
        .checked_add(Charge::capacity::<usize>(13)?)?
        .checked_add(Charge::capacity::<u16>(2)?)?
        .checked_add(Charge::capacity::<u8>(alignment)?.checked_mul(5)?)?
        .checked_mul(entries.checked_add(1).ok_or(Denial::ChargeOverflow)?)
}
