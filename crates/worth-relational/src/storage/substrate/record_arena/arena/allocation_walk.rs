//! Walk actual storage owners, preserving identities across retained arena editions.
use super::payload_allocation::{
    diagnostic_enrichment_bytes, visit_history_prefix, visit_metadata,
};
use super::{RecordArena, RecordKind};
use crate::storage::substrate::{visit_inline_value, StorageAllocationVisitor};

impl<K: RecordKind> RecordArena<K> {
    pub(crate) fn visit_authoritative_allocations(
        &self,
        unique: bool,
        visitor: &mut dyn StorageAllocationVisitor,
    ) {
        let mut union = super::extra_allocations::AllocationUnion::new(visitor);
        let visitor: &mut dyn StorageAllocationVisitor = &mut union;

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
            .visit_allocations(unique, visitor, &mut |extra, mut allocation, visitor| {
                allocation.add_checked_nested_bound(K::extra_reachable_bound_bytes(extra));
                allocation.visit(visitor);
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

    pub(crate) fn visit_diagnostic_allocations(
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

    pub(crate) fn visit_retention_allocations(
        &self,
        unique: bool,
        visitor: &mut dyn StorageAllocationVisitor,
    ) {
        self.branch_pins.visit_allocations(unique, visitor);
        self.replay_pins.visit_allocations(unique, visitor);
        self.snapshot_pins.visit_allocations(unique, visitor);
    }
}
