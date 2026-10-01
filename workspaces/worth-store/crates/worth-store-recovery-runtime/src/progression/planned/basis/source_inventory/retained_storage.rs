//! Owned selected-inventory backing, including conservative B-tree node storage.

use super::*;

impl RecoveryObservedSuccessorCandidate {
    pub(crate) fn owned_heap_bytes(&self) -> Option<u64> {
        let arrays = u64::try_from(std::mem::size_of_val(&*self.placements))
            .ok()?
            .checked_add(u64::try_from(std::mem::size_of_val(&*self.segment_entries)).ok()?)?
            .checked_add(u64::try_from(std::mem::size_of_val(&*self.free_entries)).ok()?)?
            .checked_add(u64::try_from(std::mem::size_of_val(&*self.referenced_artifacts)).ok()?)?
            .checked_add(u64::try_from(std::mem::size_of_val(&*self.artifacts)).ok()?)?;
        self.artifacts.iter().try_fold(arrays, |bytes, artifact| {
            bytes.checked_add(u64::try_from(artifact.bytes.len()).ok()?)
        })
    }
}

impl RecoverySelectedSourceInventory {
    pub(crate) fn owned_heap_bytes(&self) -> Option<u64> {
        let mut bytes = u64::try_from(std::mem::size_of_val(&*self.free_entries))
            .ok()?
            .checked_add(u64::try_from(std::mem::size_of_val(&*self.source_artifacts)).ok()?)?
            .checked_add(map_node_charge::<(u64, u64), RecoverySelectedSegmentPage>(
                self.segment_pages.len(),
            )?)?
            .checked_add(map_node_charge::<
                (u64, u64),
                worth_store_physical_format::PhysicalSegmentMembershipBlock,
            >(self.segment_topology.len())?)?
            .checked_add(map_node_charge::<
                (u64, u64),
                worth_store_physical_format::PhysicalFreeSpaceMembershipBlock,
            >(self.free_topology.len())?)?;
        for block in self.segment_topology.values() {
            bytes = bytes.checked_add(block.owned_heap_bytes()?)?;
        }
        for block in self.free_topology.values() {
            bytes = bytes.checked_add(block.owned_heap_bytes()?)?;
        }
        Some(bytes)
    }
}

fn map_node_charge<Key, Value>(entries: usize) -> Option<u64> {
    // Stable Rust does not expose BTreeMap node capacity. A nonempty node has
    // at least one key. Charge every entry for a whole maximum node: eleven
    // keys/values, twelve edges, and parent/length/alignment storage. This is
    // an upper structural bound, deliberately not an exact allocator measure.
    let node = std::mem::size_of::<Key>()
        .checked_add(std::mem::size_of::<Value>())?
        .checked_mul(12)?
        .checked_add(16 * std::mem::size_of::<usize>())?;
    u64::try_from(entries)
        .ok()?
        .checked_mul(u64::try_from(node).ok()?)
}
