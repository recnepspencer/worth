/// Conservative logical allocation charge for im 15.1.0's ordered map.
///
/// Each B-tree node reserves 64 inline pairs and 65 child references. An
/// insertion-only map has at least 32 pairs per non-root node; charge one full
/// root even for an empty map. Count shared nodes in each retained entry so
/// sharing never discounts the owner's admission budget.
pub(super) fn map_charge_bytes<K: Ord + Clone, V: Clone>(map: &im::OrdMap<K, V>) -> usize {
    const NODE_CAPACITY: usize = 64;
    const MINIMUM_OCCUPANCY: usize = NODE_CAPACITY / 2;
    const ALLOCATION_AND_CHUNK_METADATA: usize = 128;
    let node_bytes = NODE_CAPACITY
        .saturating_mul(std::mem::size_of::<(K, V)>())
        .saturating_add((NODE_CAPACITY + 1).saturating_mul(std::mem::size_of::<usize>()))
        .saturating_add(ALLOCATION_AND_CHUNK_METADATA);
    let maximum_nodes = 1usize.saturating_add(map.len() / MINIMUM_OCCUPANCY);
    node_bytes.saturating_mul(maximum_nodes)
}
