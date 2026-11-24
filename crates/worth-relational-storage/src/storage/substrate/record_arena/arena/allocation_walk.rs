//! Walk actual storage owners, preserving identities across retained arena editions.
use super::payload_allocation::{
    diagnostic_enrichment_bytes, visit_history_prefix, visit_metadata,
};
use super::{RecordArena, RecordKind};
use crate::storage::substrate::{visit_inline_value, StorageAllocationVisitor};

impl<K: RecordKind> RecordArena<K> {
    pub fn visit_authoritative_allocations(
        &self,
        unique: bool,
        visitor: &mut dyn StorageAllocationVisitor,
    ) {
        let mut union = super::extra_allocations::AllocationUnion::new(visitor);
        self.visit_authoritative_allocation_edges(unique, &mut union);
    }

    /// Raw edge traversal without an internal dedup set. Used by exact
    /// refcount-closure reclamation so multiple internal edges to one shared
    /// allocation (e.g. `extra` and `metadata_history`, or two slots sharing a
    /// map) are each counted against that allocation's live `strong_count`.
    pub fn visit_authoritative_allocation_edges(
        &self,
        unique: bool,
        visitor: &mut dyn StorageAllocationVisitor,
    ) {
        self.slots.visit_allocations(unique, visitor);
        self.partition_ids
            .visit_allocations(unique, visitor, &mut visit_inline_value);
        self.generations
            .visit_allocations(unique, visitor, &mut visit_inline_value);
        self.lifecycle
            .visit_allocations(unique, visitor, &mut visit_inline_value);
        self.kind_ids
            .visit_allocations(unique, visitor, &mut visit_inline_value);
        self.history_prefixes
            .visit_allocations(unique, visitor, &mut visit_history_prefix);
        self.created_at
            .visit_allocations(unique, visitor, &mut visit_inline_value);
        self.retired_at
            .visit_allocations(unique, visitor, &mut visit_inline_value);
        self.live_bitset.visit_allocations(unique, visitor);
        self.reclaimable_bitset.visit_allocations(unique, visitor);
        self.metadata_history.visit_allocations(
            unique,
            visitor,
            &mut |history, allocation, visitor| {
                if allocation.visit(visitor) {
                    history.visit_allocations(allocation.unique, visitor, &mut visit_metadata::<K>);
                }
            },
        );
        self.extra
            .visit_allocations(unique, visitor, &mut |extra, allocation, visitor| {
                if allocation.visit(visitor) {
                    K::visit_extra_allocations(extra, allocation.unique, visitor);
                }
            });
        self.aspect_versions.visit_allocations(
            unique,
            visitor,
            &mut |versions, allocation, visitor| {
                versions.visit_allocations(allocation.unique, visitor, &mut visit_inline_value);
            },
        );
        self.field_revisions.visit_allocations(
            unique,
            visitor,
            &mut |revisions, allocation, visitor| {
                if let Some(revisions) = revisions {
                    revisions.visit_allocations(
                        allocation.unique,
                        visitor,
                        &mut visit_inline_value,
                    );
                }
            },
        );
    }

    pub fn visit_diagnostic_allocations(
        &self,
        unique: bool,
        visitor: &mut dyn StorageAllocationVisitor,
    ) {
        self.diagnostics_enrichment.visit_allocations(
            unique,
            visitor,
            &mut |entries, mut allocation, visitor| {
                allocation.add_owned_bytes(diagnostic_enrichment_bytes(entries));
                allocation.visit(visitor);
            },
        );
    }

    pub fn visit_retention_allocations(
        &self,
        unique: bool,
        visitor: &mut dyn StorageAllocationVisitor,
    ) {
        self.branch_pins.visit_allocations(unique, visitor);
        self.replay_pins.visit_allocations(unique, visitor);
        self.snapshot_pins.visit_allocations(unique, visitor);
    }
}
