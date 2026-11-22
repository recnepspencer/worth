use std::sync::Arc;

use super::{
    RelationalPersistentRegionLeaf, RelationalPersistentRegionNode, RelationalPersistentRegionSet,
};
use crate::storage::substrate::StorageAllocationOverflow;

impl RelationalPersistentRegionSet {
    pub(in crate::branch) fn reclaimable_unique_authoritative_bytes(
        regions: &Arc<Self>,
    ) -> Result<u64, StorageAllocationOverflow> {
        if Arc::strong_count(regions) != 1 {
            return Ok(0);
        }
        let mut bytes = std::mem::size_of::<Self>() as u64;
        observe_reclaimable_node_bytes(regions.index_root.as_ref(), &mut bytes)?;
        Ok(bytes)
    }
}

fn observe_reclaimable_node_bytes(
    current: Option<&Arc<RelationalPersistentRegionNode>>,
    bytes: &mut u64,
) -> Result<(), StorageAllocationOverflow> {
    let Some(node) = current else { return Ok(()) };
    if Arc::strong_count(node) != 1 {
        return Ok(());
    }
    *bytes = bytes
        .checked_add(std::mem::size_of::<RelationalPersistentRegionNode>() as u64)
        .ok_or(StorageAllocationOverflow)?;
    if let Some(leaf) = node.leaf.as_ref() {
        let leaf_bytes = match leaf {
            RelationalPersistentRegionLeaf::Present(region) => {
                let storage = std::mem::size_of_val(region.as_ref()) as u64;
                if Arc::strong_count(&region.region) == 1 {
                    storage
                        .checked_add(region.region.reclaimable_unique_authoritative_bytes()?)
                        .ok_or(StorageAllocationOverflow)?
                } else {
                    storage
                }
            }
            RelationalPersistentRegionLeaf::Removed(partition) => {
                std::mem::size_of_val(partition.as_ref()) as u64
            }
        };
        *bytes = bytes
            .checked_add(leaf_bytes)
            .ok_or(StorageAllocationOverflow)?;
    }
    observe_reclaimable_node_bytes(node.zero.as_ref(), bytes)?;
    observe_reclaimable_node_bytes(node.one.as_ref(), bytes)
}
