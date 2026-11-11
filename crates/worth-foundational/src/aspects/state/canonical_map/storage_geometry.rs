//! Concrete checked bounds for the locked im15.1.0/sized-chunks0.6.5 tree.
//!
//! The threaded fakepool uses Arc. Both Chunk cursors, aligned inline cells,
//! optional child references and the Arc header are included. Two words per
//! child conservatively cover the resolved one-word wrapper. A dependency or
//! representation change invalidates the allocation evidence for these bounds.
use std::alloc::Layout;

const NODE_KEYS: usize = 64;
const MINIMUM_NON_ROOT_KEYS: usize = 31;
const COPIED_NODES_PER_LEVEL: usize = 8;
const KEY_COPIES_PER_LEVEL: usize = 16;

fn node_bytes<K, V>() -> Option<usize> {
    let keys = Layout::new::<[usize; 2]>()
        .extend(Layout::array::<(K, V)>(NODE_KEYS).ok()?)
        .ok()?
        .0
        .pad_to_align();
    let children = Layout::new::<[usize; 2]>()
        .extend(Layout::array::<[usize; 2]>(NODE_KEYS + 1).ok()?)
        .ok()?
        .0
        .pad_to_align();
    let node = keys.extend(children).ok()?.0.pad_to_align();
    Some(
        Layout::new::<[usize; 2]>()
            .extend(node)
            .ok()?
            .0
            .pad_to_align()
            .size(),
    )
}

pub(super) fn retained_nodes<K, V>(entries: usize) -> Option<usize> {
    // A concrete input must itself have addressable cells. Empty im maps still
    // allocate one root; every additional node has at least31 keys.
    Layout::array::<(K, V)>(entries).ok()?;
    let nodes = if entries <= 2 * MINIMUM_NON_ROOT_KEYS {
        // A root with two minimum-size children already needs63 entries.
        1
    } else {
        (entries / MINIMUM_NON_ROOT_KEYS).checked_add(1)?
    };
    node_bytes::<K, V>()?.checked_mul(nodes)
}

pub(super) fn edit_paths<K, V>(
    maximum_entries: usize,
    operations: usize,
    chunk_key_bytes: usize,
) -> Option<usize> {
    if operations == 0 {
        return Some(0);
    }
    let height = usize::BITS as usize - maximum_entries.checked_add(1)?.leading_zeros() as usize;
    let levels = height.checked_add(1)?.checked_mul(operations)?;
    // Insertion: path COW, two-way split and root replacement. Deletion: path
    // COW, sibling COW, pull-up/merge/steal and root replacement. Eight nodes
    // per level conservatively cover those branches. Sixteen key copies cover
    // chunk clones, recursive inserted values and separator copies. Arc values
    // retain their existing payload, so they do not deep-clone nested heaps.
    let nodes = levels.checked_mul(COPIED_NODES_PER_LEVEL)?;
    let keys = levels.checked_mul(KEY_COPIES_PER_LEVEL)?;
    node_bytes::<K, V>()?
        .checked_mul(nodes)?
        .checked_add(chunk_key_bytes.checked_mul(keys)?)
}
