use std::sync::Arc;

use super::{
    RelationalPersistentRegionLeaf, RelationalPersistentRegionNode, RelationalPersistentRegionSet,
};

impl RelationalPersistentRegionSet {
    pub(in crate::branch) fn reclaimable_unique_authoritative_bytes(regions: &Arc<Self>) -> u64 {
        if Arc::strong_count(regions) != 1 {
            return 0;
        }
        let mut bytes = std::mem::size_of::<Self>() as u64;
        observe_reclaimable_node_bytes(regions.index_root.as_ref(), &mut bytes);
        bytes
    }
}

fn observe_reclaimable_node_bytes(
    current: Option<&Arc<RelationalPersistentRegionNode>>,
    bytes: &mut u64,
) {
    let Some(node) = current else { return };
    if Arc::strong_count(node) != 1 {
        return;
    }
    *bytes = bytes.saturating_add(std::mem::size_of::<RelationalPersistentRegionNode>() as u64);
    if let Some(leaf) = node.leaf.as_ref() {
        *bytes = bytes.saturating_add(match leaf {
            RelationalPersistentRegionLeaf::Present(region) => {
                let storage = std::mem::size_of_val(region.as_ref()) as u64;
                if Arc::strong_count(&region.region) == 1 {
                    storage.saturating_add(region.region.reclaimable_unique_authoritative_bytes())
                } else {
                    storage
                }
            }
            RelationalPersistentRegionLeaf::Removed(partition) => {
                std::mem::size_of_val(partition.as_ref()) as u64
            }
        });
    }
    observe_reclaimable_node_bytes(node.zero.as_ref(), bytes);
    observe_reclaimable_node_bytes(node.one.as_ref(), bytes);
}
