//! Quote insertion coexistence from the original AVL path and allocation owner.
use super::{node::MapNode, SharedMap};
use crate::storage::substrate::StorageAllocation;

impl<K: Ord + Copy, V: Copy> SharedMap<K, V> {
    /// Every insertion creates a value owner. A shared ancestor's copy makes
    /// its selected children shared too. Rotation pivots are on that same path;
    /// rotating uniquely admitted nodes does not create another allocation.
    pub(crate) fn insertion_allocation_bytes(&self, key: &K) -> Option<u64> {
        let node_bytes = StorageAllocation::<MapNode<K, V>>::layout_bytes();
        let mut bytes = StorageAllocation::<V>::layout_bytes();
        let mut current = self.root.as_ref();
        let mut shared = false;
        while let Some(node) = current {
            shared |= !node.is_unique();
            if shared {
                bytes = bytes.checked_add(node_bytes)?;
            }
            current = match key.cmp(&node.key) {
                std::cmp::Ordering::Less => node.left.as_ref(),
                std::cmp::Ordering::Greater => node.right.as_ref(),
                std::cmp::Ordering::Equal => return Some(bytes),
            };
        }
        bytes.checked_add(node_bytes)
    }

    /// Node and value owners only; separately owned V payload is its owner's bill.
    pub(crate) fn shallow_allocation_bytes(&self) -> Option<u64> {
        let pair = StorageAllocation::<MapNode<K, V>>::layout_bytes()
            .checked_add(StorageAllocation::<V>::layout_bytes())?;
        u64::try_from(self.len()).ok()?.checked_mul(pair)
    }
}

#[cfg(test)]
mod tests;
