//! Walk actual storage owners, preserving identities across retained arena editions.
use super::payload_allocation::{
    aspect_version_bytes, diagnostic_enrichment_bytes, field_revision_bytes, visit_metadata,
};
use super::{RecordArena, RecordKind};
use crate::storage::substrate::{visit_inline_value, StorageAllocationVisitor};

impl<K: RecordKind> RecordArena<K> {
    pub(crate) fn visit_authoritative_allocations(
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
                if visitor.visit(allocation) {
                    history.visit_allocations(allocation.unique, visitor, &mut visit_metadata::<K>);
                }
            },
        );
        self.extra
            .visit_allocations(unique, visitor, &mut |extra, mut allocation, visitor| {
                allocation.bytes += K::extra_owned_allocation_bytes(extra);
                visitor.visit(allocation);
            });
        self.aspect_versions.visit_allocations(
            unique,
            visitor,
            &mut |versions, mut allocation, visitor| {
                allocation.bytes += aspect_version_bytes(versions);
                visitor.visit(allocation);
            },
        );
        self.field_revisions.visit_allocations(
            unique,
            visitor,
            &mut |revisions, mut allocation, visitor| {
                allocation.bytes += field_revision_bytes(revisions);
                visitor.visit(allocation);
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
                allocation.bytes += diagnostic_enrichment_bytes(entries);
                visitor.visit(allocation);
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
