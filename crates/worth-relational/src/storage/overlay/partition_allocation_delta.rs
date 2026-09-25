//! Update an immutable region inventory by changed owner paths in both directions.
use super::{PartitionState, RelationalPartitionAllocationInventory};
use crate::storage::substrate::{StorageAllocationObservation, StorageAllocationVisitor};

impl PartitionState {
    pub(crate) fn allocation_inventory_since(
        &self,
        previous: &Self,
        prior: RelationalPartitionAllocationInventory,
    ) -> RelationalPartitionAllocationInventory {
        let introduced = self.new_allocation_inventory_since(previous);
        let removed = previous.new_allocation_inventory_since(self);
        RelationalPartitionAllocationInventory {
            authoritative_bytes: update(
                prior.authoritative_bytes,
                removed.authoritative_bytes,
                introduced.authoritative_bytes,
            ),
            diagnostic_bytes: update(
                prior.diagnostic_bytes,
                removed.diagnostic_bytes,
                introduced.diagnostic_bytes,
            ),
            optional_cache_bytes: update(
                prior.optional_cache_bytes,
                removed.optional_cache_bytes,
                introduced.optional_cache_bytes,
            ),
            // Both operands are immutable regions with cleared live-owner pins.
            retention_metadata_bytes: 0,
            allocator_bookkeeping_bytes: 0,
        }
    }

    fn new_allocation_inventory_since(
        &self,
        previous: &Self,
    ) -> RelationalPartitionAllocationInventory {
        let mut diagnostic = Bytes::default();
        self.entity_arena
            .visit_new_diagnostic_allocations(&previous.entity_arena, &mut diagnostic);
        self.relation_arena
            .visit_new_diagnostic_allocations(&previous.relation_arena, &mut diagnostic);
        let mut cache = Bytes::default();
        self.adjacency
            .visit_new_cache_allocations(&previous.adjacency, &mut cache);
        self.reverse_adjacency
            .visit_new_cache_allocations(&previous.reverse_adjacency, &mut cache);
        RelationalPartitionAllocationInventory {
            authoritative_bytes: self.new_authoritative_allocation_bytes_since(previous),
            diagnostic_bytes: diagnostic.0,
            optional_cache_bytes: cache.0,
            ..Default::default()
        }
    }
}

fn update(prior: u64, removed: u64, introduced: u64) -> u64 {
    prior
        .checked_sub(removed)
        .and_then(|bytes| bytes.checked_add(introduced))
        .expect("owner allocation delta must fit its prior inventory")
}

#[derive(Default)]
struct Bytes(u64);
impl StorageAllocationVisitor for Bytes {
    fn visit(&mut self, allocation: StorageAllocationObservation) -> bool {
        self.0 = self.0.saturating_add(allocation.bytes);
        true
    }
}
