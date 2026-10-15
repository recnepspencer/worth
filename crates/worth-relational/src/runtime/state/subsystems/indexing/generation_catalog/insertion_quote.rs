//! Per-container std B-tree insertion geometry, including a possible root split.
use std::alloc::Layout;

// Rust alloc::collections::btree::node uses B=6: eleven key/value slots,
// twelve internal edges, and parent/index/length metadata. Non-root nodes
// contain at least five keys. See https://github.com/rust-lang/rust/blob/1.94.0/library/alloc/src/collections/btree/node.rs.
pub(super) fn path_nodes(entries: usize) -> usize {
    let mut nodes = 1;
    let mut minimum = 1u128;
    while let Some(next) = minimum.checked_mul(6).and_then(|n| n.checked_add(5)) {
        if next > entries as u128 {
            break;
        }
        minimum = next;
        nodes += 1;
    }
    nodes
}

pub(super) fn insertion<K, V>(entries: usize, key_visits: usize) -> Option<(u64, u64)> {
    let nodes = path_nodes(entries);
    // Quote separate key/value arrays and their alignment, rather than an
    // array of inflated scope tuples. Header metadata and twelve edges are
    // distinct native allocations fields; pad both leaf and internal layout.
    let (leaf, _) = Layout::new::<(usize, u16, u16)>()
        .extend(Layout::new::<[K; 11]>())
        .ok()?;
    let (leaf, _) = leaf.extend(Layout::new::<[V; 11]>()).ok()?;
    let (internal, _) = leaf
        .pad_to_align()
        .extend(Layout::new::<[usize; 12]>())
        .ok()?;
    let node = internal.pad_to_align().size();
    // One sibling per level and one new root; a leaf insertion can allocate
    // one leaf. The bound covers either case without charging binary depth.
    let bytes = node
        .checked_mul(nodes)?
        .checked_add(leaf.pad_to_align().size())?;
    let comparisons = nodes
        .checked_mul(11)?
        .checked_mul(key_visits)?
        .checked_add(1)?;
    Some((u64::try_from(comparisons).ok()?, u64::try_from(bytes).ok()?))
}

pub(super) fn add(total: &mut (u64, u64), quote: (u64, u64)) -> Option<()> {
    total.0 = total.0.checked_add(quote.0)?;
    total.1 = total.1.checked_add(quote.1)?;
    Some(())
}
