use super::PartitionState;
use crate::storage::substrate::{
    ExtraDeltaCollector, SealedAllocationWatermarks, StorageAllocationExtent,
    StorageAllocationOverflow, StorageAllocationVisitor,
};
use std::collections::{BTreeMap, BTreeSet};

impl PartitionState {
    /// Checked exact physical bytes of the authoritative allocations this
    /// partition introduces beyond `previous`, deduplicated across entity and
    /// relation arenas, unchanged slots, and retained metadata history. An
    /// unrepresentable total refuses before publication instead of saturating.
    #[cfg(test)]
    pub(crate) fn new_authoritative_allocation_bytes_since(
        &self,
        previous: &Self,
    ) -> Result<u64, super::PartitionContentDigestError> {
        self.new_authoritative_allocation_bytes_since_with_watermarks(
            previous,
            SealedAllocationWatermarks::default(),
        )
    }

    pub(crate) fn visit_new_authoritative_allocations_since_with_watermarks(
        &self,
        previous: &Self,
        previous_watermarks: SealedAllocationWatermarks,
        visitor: &mut dyn StorageAllocationVisitor,
    ) {
        self.entity_arena
            .visit_new_non_extra_authoritative_allocations(&previous.entity_arena, visitor);
        self.relation_arena
            .visit_new_non_extra_authoritative_allocations(&previous.relation_arena, visitor);
        let mut extras = ExtraDeltaCollector::with_watermarks(previous_watermarks);
        self.entity_arena.collect_changed_extra_allocations(
            &previous.entity_arena,
            previous_watermarks.extra,
            &mut extras,
        );
        self.relation_arena.collect_changed_extra_allocations(
            &previous.relation_arena,
            previous_watermarks.extra,
            &mut extras,
        );
        if extras.has_unresolved() {
            let mut seen = BTreeSet::new();
            previous
                .entity_arena
                .resolve_extra_allocations(&mut seen, &mut extras);
            previous
                .relation_arena
                .resolve_extra_allocations(&mut seen, &mut extras);
        }
        extras.emit(visitor);
        self.adjacency
            .visit_new_authoritative_allocations(&previous.adjacency, visitor);
        self.reverse_adjacency
            .visit_new_authoritative_allocations(&previous.reverse_adjacency, visitor);
    }

    pub(crate) fn new_authoritative_allocation_bytes_since_with_watermarks(
        &self,
        previous: &Self,
        previous_watermarks: SealedAllocationWatermarks,
    ) -> Result<u64, super::PartitionContentDigestError> {
        let mut new = NewAllocations {
            bytes: Some(0),
            seen: BTreeSet::new(),
        };
        self.visit_new_authoritative_allocations_since_with_watermarks(
            previous,
            previous_watermarks,
            &mut new,
        );
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

    /// Raw edge traversal across both arenas and adjacency tables for exact
    /// refcount-closure reclamation (every internal owning edge is visited so
    /// multi-edge exclusively owned allocations reach `dying_refs == strong_count`).
    pub(crate) fn visit_authoritative_allocation_edges(
        &self,
        visitor: &mut dyn StorageAllocationVisitor,
    ) {
        self.entity_arena
            .visit_authoritative_allocation_edges(false, visitor);
        self.relation_arena
            .visit_authoritative_allocation_edges(false, visitor);
        self.adjacency
            .visit_authoritative_allocations(false, visitor);
        self.reverse_adjacency
            .visit_authoritative_allocations(false, visitor);
    }

    #[cfg(test)]
    pub(crate) fn reclaimable_unique_authoritative_bytes(
        &self,
    ) -> Result<u64, StorageAllocationOverflow> {
        let mut reclaimable = ReclaimableAllocations::new();
        self.visit_authoritative_allocation_edges(&mut reclaimable);
        reclaimable.finish()
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

/// Exact internal-refcount graph closure for a retiring root.
///
/// An allocation is charged and descended into if and only if the number of
/// dying owning edges observed from already-freed parents in this retiring root
/// reaches that allocation's live `strong_count`. Multiple internal edges (such
/// as `extra` + `metadata_history`, or two slots/arenas sharing a map) release
/// together when their last external owner is gone, while any outside holder
/// keeps `dying_refs < strong_count` and excludes the allocation.
pub(crate) struct ReclaimableAllocations {
    bytes: Option<u64>,
    dying_refs: BTreeMap<u64, usize>,
}

impl ReclaimableAllocations {
    pub(crate) fn new() -> Self {
        Self {
            bytes: Some(0),
            dying_refs: BTreeMap::new(),
        }
    }

    pub(crate) fn finish(self) -> Result<u64, StorageAllocationOverflow> {
        self.bytes.ok_or(StorageAllocationOverflow)
    }
}

impl StorageAllocationVisitor for ReclaimableAllocations {
    fn visit(
        &mut self,
        allocation: crate::storage::substrate::StorageAllocationObservation,
    ) -> bool {
        if self.bytes.is_none() {
            return false;
        }
        let refs = self.dying_refs.entry(allocation.id).or_insert(0);
        *refs += 1;
        if *refs != allocation.strong_count {
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
            strong_count: 1,
            birth_seq: None,
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
