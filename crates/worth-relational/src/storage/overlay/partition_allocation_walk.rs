use super::PartitionState;
use crate::storage::substrate::{StorageAllocationExtent, StorageAllocationVisitor};

impl PartitionState {
    /// Checked bytes of the owners this partition introduces beyond `previous`.
    /// A payload carrier contributes its declared upper bound for nested aspect
    /// backing; an unrepresentable bound or total refuses before publication.
    pub(crate) fn new_authoritative_allocation_bytes_since(
        &self,
        previous: &Self,
    ) -> Result<u64, super::PartitionContentDigestError> {
        let mut new = NewAllocations { bytes: Some(0) };
        self.entity_arena
            .visit_new_authoritative_allocations(&previous.entity_arena, &mut new);
        self.relation_arena
            .visit_new_authoritative_allocations(&previous.relation_arena, &mut new);
        self.adjacency
            .visit_new_authoritative_allocations(&previous.adjacency, &mut new);
        self.reverse_adjacency
            .visit_new_authoritative_allocations(&previous.reverse_adjacency, &mut new);
        new.bytes
            .ok_or(super::PartitionContentDigestError::AllocationAccountingOverflow)
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

/// `None` once a declared bound or the running total is unrepresentable.
struct NewAllocations {
    bytes: Option<u64>,
}
impl StorageAllocationVisitor for NewAllocations {
    fn visit(
        &mut self,
        allocation: crate::storage::substrate::StorageAllocationObservation,
    ) -> bool {
        self.bytes = match allocation.extent {
            StorageAllocationExtent::Checked => self
                .bytes
                .and_then(|bytes| bytes.checked_add(allocation.bytes)),
            StorageAllocationExtent::Overflow => None,
        };
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

#[cfg(test)]
mod tests {
    use super::NewAllocations;
    use crate::storage::substrate::{
        StorageAllocationExtent, StorageAllocationObservation, StorageAllocationVisitor,
    };

    #[test]
    fn unrepresentable_new_owner_bound_or_total_is_refused_not_saturated() {
        let mut carrier = StorageAllocationObservation {
            id: 1,
            bytes: 64,
            unique: false,
            extent: StorageAllocationExtent::Checked,
        };
        let mut new = NewAllocations { bytes: Some(0) };
        assert!(new.visit(carrier));
        assert_eq!(new.bytes, Some(64));
        let mut total = NewAllocations {
            bytes: Some(u64::MAX),
        };
        assert!(total.visit(carrier));
        assert_eq!(total.bytes, None);
        carrier.add_checked_nested_bound(None);
        assert!(new.visit(carrier));
        assert_eq!(new.bytes, None);
    }
}
