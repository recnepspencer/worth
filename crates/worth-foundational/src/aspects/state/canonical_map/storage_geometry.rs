//! Concrete checked bounds for the owned persistent AVL aspect entry tree.
//!
//! Each entry owns one `Arc<EntryNode>`. Empty maps allocate no heap backing.
//! Path-copying AVL edits clone `ContractValidatedAspectValue` handles without
//! copying keys or nested payloads; a double rotation allocates at most three
//! nodes per reached level.
use super::entry_tree::AspectEntryTree;
use std::alloc::Layout;

const COPIED_NODES_PER_LEVEL: usize = 3;

pub(super) fn retained_nodes(entries: usize) -> Option<usize> {
    // A concrete input must itself have addressable entries.
    Layout::array::<usize>(entries).ok()?;
    AspectEntryTree::checked_node_allocation_bytes()?.checked_mul(entries)
}

pub(super) fn edit_paths(maximum_entries: usize, operations: usize) -> Option<usize> {
    if operations == 0 {
        return Some(0);
    }
    let maximum = maximum_entries.checked_add(1)?;
    // AVL height is strictly less than twice the binary height.
    let levels = (usize::BITS as usize - maximum.leading_zeros() as usize)
        .checked_mul(2)?
        .checked_add(1)?;
    AspectEntryTree::checked_node_allocation_bytes()?
        .checked_mul(COPIED_NODES_PER_LEVEL)?
        .checked_mul(levels)?
        .checked_mul(operations)
}
