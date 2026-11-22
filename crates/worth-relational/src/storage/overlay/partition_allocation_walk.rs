use super::PartitionState;
use crate::storage::substrate::{
    ExtraDeltaCollector, StorageAllocationExtent, StorageAllocationOverflow,
    StorageAllocationVisitor,
};
use std::collections::BTreeSet;

impl PartitionState {
    /// Checked exact physical bytes of the authoritative allocations this
    /// partition introduces beyond `previous`, deduplicated across entity and
    /// relation arenas, unchanged slots, and retained metadata history. An
    /// unrepresentable total refuses before publication instead of saturating.
    pub(crate) fn new_authoritative_allocation_bytes_since(
        &self,
        previous: &Self,
    ) -> Result<u64, super::PartitionContentDigestError> {
        let mut new = NewAllocations {
            bytes: Some(0),
            seen: BTreeSet::new(),
        };
        self.entity_arena
            .visit_new_non_extra_authoritative_allocations(&previous.entity_arena, &mut new);
        self.relation_arena
            .visit_new_non_extra_authoritative_allocations(&previous.relation_arena, &mut new);
        let mut extras = ExtraDeltaCollector::default();
        self.entity_arena
            .collect_changed_extra_allocations(&previous.entity_arena, &mut extras);
        self.relation_arena
            .collect_changed_extra_allocations(&previous.relation_arena, &mut extras);
        if extras.has_unresolved() {
            let mut seen = BTreeSet::new();
            previous
                .entity_arena
                .resolve_extra_allocations(&mut seen, &mut extras);
            previous
                .relation_arena
                .resolve_extra_allocations(&mut seen, &mut extras);
        }
        extras.emit(&mut new);
        self.adjacency
            .visit_new_authoritative_allocations(&previous.adjacency, &mut new);
        self.reverse_adjacency
            .visit_new_authoritative_allocations(&previous.reverse_adjacency, &mut new);
        new.bytes
            .ok_or(super::PartitionContentDigestError::AllocationAccountingOverflow)
    }

    pub(super) fn unique_authoritative_allocation_bytes(
        &self,
    ) -> Result<u64, StorageAllocationOverflow> {
        let mut observed = NewAllocations {
            bytes: Some(0),
            seen: BTreeSet::new(),
        };
        self.visit_authoritative_allocations(false, &mut observed);
        observed.bytes.ok_or(StorageAllocationOverflow)
    }

    pub(crate) fn reclaimable_unique_authoritative_bytes(
        &self,
    ) -> Result<u64, StorageAllocationOverflow> {
        let mut reclaimable = ReclaimableAllocations { bytes: Some(0) };
        self.visit_authoritative_allocations(true, &mut reclaimable);
        reclaimable
            .bytes
            .ok_or(crate::storage::substrate::StorageAllocationOverflow)
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

/// `None` once an observation or the running total is unrepresentable.
struct NewAllocations {
    bytes: Option<u64>,
    seen: BTreeSet<u64>,
}
impl StorageAllocationVisitor for NewAllocations {
    fn visit(
        &mut self,
        allocation: crate::storage::substrate::StorageAllocationObservation,
    ) -> bool {
        if !self.seen.insert(allocation.id) {
            return false;
        }
        self.bytes = match allocation.extent {
            StorageAllocationExtent::Checked => self
                .bytes
                .and_then(|bytes| bytes.checked_add(allocation.bytes)),
            StorageAllocationExtent::Overflow => None,
        };
        self.bytes.is_some()
    }
}

struct ReclaimableAllocations {
    bytes: Option<u64>,
}

impl StorageAllocationVisitor for ReclaimableAllocations {
    fn visit(
        &mut self,
        allocation: crate::storage::substrate::StorageAllocationObservation,
    ) -> bool {
        if !allocation.unique {
            return false;
        }
        self.bytes = match allocation.extent {
            StorageAllocationExtent::Checked => self
                .bytes
                .and_then(|bytes| bytes.checked_add(allocation.bytes)),
            StorageAllocationExtent::Overflow => None,
        };
        self.bytes.is_some()
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
        let mut new = NewAllocations {
            bytes: Some(0),
            seen: std::collections::BTreeSet::new(),
        };
        assert!(new.visit(carrier));
        assert_eq!(new.bytes, Some(64));
        let mut total = NewAllocations {
            bytes: Some(u64::MAX),
            seen: std::collections::BTreeSet::new(),
        };
        assert!(!total.visit(carrier));
        assert_eq!(total.bytes, None);
        carrier.id = 2;
        carrier.add_checked_nested_bound(None);
        assert!(!new.visit(carrier));
        assert_eq!(new.bytes, None);
    }
}
