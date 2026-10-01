use std::sync::Arc;

use super::{RetainedStorageCharge as Charge, RetainedStoragePreparationDenial as Denial};

/// Lookup/seek steps for the installed std and im B-trees described below.
/// A balanced tree has at most bit_length(entries) nonempty levels (at least
/// two children per internal node). A search can inspect only initialized
/// slots: at most min(entries, 64) comparisons and their navigation steps per
/// level, plus the missing-child decision, including an empty root.
/// This is a structural bound only:
/// callers must separately charge the concrete key's comparison cost.
pub(crate) fn ordered_lookup_steps(entries: usize) -> usize {
    let levels = (usize::BITS as usize - entries.leading_zeros() as usize).max(1);
    2 * (entries.min(64) + 1) * levels
}

/// Rust 1.94 std BTreeMap searches at most 11 initialized keys per node.
/// Non-root nodes hold at least 5 keys and therefore 6 child edges.
/// Include navigation separately from the concrete key comparison cost.
pub(crate) fn std_btree_lookup_steps(entries: usize) -> usize {
    2 * (entries.min(11) + 1) * minimum_fill_height(entries, 6)
}

/// im 15.1 OrdMap binary-searches its initialized 64-entry chunks. Splitting
/// and merging leave non-root nodes with at least 31 keys (32 child edges).
/// One extra comparison and doubled navigation cover exact/predecessor seeks.
pub(crate) fn im_btree_lookup_steps(entries: usize) -> usize {
    let keys = entries.min(64);
    let comparisons = (usize::BITS as usize - keys.leading_zeros() as usize) + 1;
    2 * (comparisons + 1) * minimum_fill_height(entries, 32)
}

fn minimum_fill_height(entries: usize, branching: usize) -> usize {
    // A nonempty root has at least one key and two children. A tree with h
    // levels therefore needs at least 2 * branching^(h - 1) - 1 keys.
    let mut minimum = 1_usize;
    let mut levels = 1;
    while let Some(next) = minimum
        .checked_add(1)
        .and_then(|n| n.checked_mul(branching))
        .and_then(|n| n.checked_sub(1))
    {
        if next > entries {
            break;
        }
        minimum = next;
        levels += 1;
    }
    levels
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

/// Upper bound for one copy-on-write im OrdMap key edit. An edited search
/// path has at most bit_length(entries + 1) levels (minimum branching two);
/// a split can allocate two nodes per level plus a new root. The value and
/// selected key payload are charged by the map owner.
pub(crate) fn ordered_edit_growth_charge<K, V>(entries: usize) -> Result<Charge, Denial> {
    let length = entries.checked_add(1).ok_or(Denial::ChargeOverflow)?;
    let levels = (usize::BITS as usize - length.leading_zeros() as usize)
        .checked_add(1)
        .ok_or(Denial::ChargeOverflow)?;
    ordered_index_charge::<K, V>(0)?
        .checked_mul(levels.checked_mul(2).ok_or(Denial::ChargeOverflow)?)
}

/// Conservative Rust 1.94 BTreeMap structural bound. alloc/collections/btree/
/// node.rs uses 11 key/value slots, 12 child pointers, a parent pointer and two
/// u16 fields. Every nonroot node retains at least five keys, so charge at
/// most ceil(entries / 5) occupied nodes plus one spare root. The spare root
/// also covers the first insertion from an empty map. Charge each node as
/// internal, including padding for every aligned region.
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
        .checked_mul(
            entries
                .div_ceil(5)
                .checked_add(1)
                .ok_or(Denial::ChargeOverflow)?,
        )
}
