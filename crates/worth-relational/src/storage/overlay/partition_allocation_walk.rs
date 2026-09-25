use super::PartitionState;
use crate::storage::substrate::StorageAllocationVisitor;

impl PartitionState {
    pub(crate) fn new_authoritative_allocation_bytes_since(&self, previous: &Self) -> u64 {
        let mut new = NewAllocations::default();
        self.entity_arena
            .visit_new_authoritative_allocations(&previous.entity_arena, &mut new);
        self.relation_arena
            .visit_new_authoritative_allocations(&previous.relation_arena, &mut new);
        self.adjacency
            .visit_new_authoritative_allocations(&previous.adjacency, &mut new);
        self.reverse_adjacency
            .visit_new_authoritative_allocations(&previous.reverse_adjacency, &mut new);
        new.bytes
    }
    pub(crate) fn reclaimable_unique_authoritative_bytes(&self) -> u64 {
        let mut reclaimable = ReclaimableAllocations::default();
        self.visit_authoritative_allocations(true, &mut reclaimable);
        reclaimable.bytes
    }
    pub(crate) fn visit_authoritative_allocations(
        &self,
        unique: bool,
        visitor: &mut dyn StorageAllocationVisitor,
    ) {
        self.entity_arena
            .visit_authoritative_allocations(unique, visitor);
        self.relation_arena
            .visit_authoritative_allocations(unique, visitor);
        self.adjacency
            .visit_authoritative_allocations(unique, visitor);
        self.reverse_adjacency
            .visit_authoritative_allocations(unique, visitor);
    }

    pub(crate) fn visit_diagnostic_allocations(&self, visitor: &mut dyn StorageAllocationVisitor) {
        self.entity_arena
            .visit_diagnostic_allocations(false, visitor);
        self.relation_arena
            .visit_diagnostic_allocations(false, visitor);
    }

    pub(crate) fn visit_retention_allocations(&self, visitor: &mut dyn StorageAllocationVisitor) {
        self.entity_arena
            .visit_retention_allocations(false, visitor);
        self.relation_arena
            .visit_retention_allocations(false, visitor);
    }

    pub(crate) fn visit_cache_allocations(&self, visitor: &mut dyn StorageAllocationVisitor) {
        self.adjacency.visit_cache_allocations(visitor);
        self.reverse_adjacency.visit_cache_allocations(visitor);
    }
}

#[derive(Default)]
struct NewAllocations {
    bytes: u64,
}
impl StorageAllocationVisitor for NewAllocations {
    fn visit(
        &mut self,
        allocation: crate::storage::substrate::StorageAllocationObservation,
    ) -> bool {
        self.bytes = self.bytes.saturating_add(allocation.bytes);
        true
    }
}

#[derive(Default)]
struct ReclaimableAllocations {
    bytes: u64,
}

impl StorageAllocationVisitor for ReclaimableAllocations {
    fn visit(
        &mut self,
        allocation: crate::storage::substrate::StorageAllocationObservation,
    ) -> bool {
        if !allocation.unique {
            return false;
        }
        self.bytes = self.bytes.saturating_add(allocation.bytes);
        true
    }
}
