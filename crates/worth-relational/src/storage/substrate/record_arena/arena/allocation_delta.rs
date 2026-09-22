//! Publication delta accounting traverses changed column/index paths, not occupied rows.
use super::payload_allocation::{
    aspect_version_bytes, diagnostic_enrichment_bytes, visit_metadata,
};
use super::{RecordArena, RecordKind};
use crate::storage::substrate::{visit_new_inline_value, StorageAllocationVisitor};

impl<K: RecordKind> RecordArena<K> {
    pub(crate) fn visit_new_diagnostic_allocations(
        &self,
        previous: &Self,
        visitor: &mut dyn StorageAllocationVisitor,
    ) {
        self.diagnostics_enrichment.visit_new_allocations(
            &previous.diagnostics_enrichment,
            visitor,
            &mut |entries, _, mut allocation, visitor| {
                allocation.bytes += diagnostic_enrichment_bytes(entries);
                visitor.visit(allocation);
            },
        );
    }

    pub(crate) fn visit_new_authoritative_allocations(
        &self,
        previous: &Self,
        visitor: &mut dyn StorageAllocationVisitor,
    ) {
        self.slots.visit_new_allocations(&previous.slots, visitor);
        self.partition_ids.visit_new_allocations(
            &previous.partition_ids,
            visitor,
            &mut visit_new_inline_value,
        );
        self.generations.visit_new_allocations(
            &previous.generations,
            visitor,
            &mut visit_new_inline_value,
        );
        self.lifecycle.visit_new_allocations(
            &previous.lifecycle,
            visitor,
            &mut visit_new_inline_value,
        );
        self.kind_ids.visit_new_allocations(
            &previous.kind_ids,
            visitor,
            &mut visit_new_inline_value,
        );
        self.created_at.visit_new_allocations(
            &previous.created_at,
            visitor,
            &mut visit_new_inline_value,
        );
        self.retired_at.visit_new_allocations(
            &previous.retired_at,
            visitor,
            &mut visit_new_inline_value,
        );
        self.live_bitset
            .visit_new_allocations(&previous.live_bitset, visitor);
        self.reclaimable_bitset
            .visit_new_allocations(&previous.reclaimable_bitset, visitor);
        self.metadata_history.visit_new_allocations(
            &previous.metadata_history,
            visitor,
            &mut |history, previous, allocation, visitor| {
                if visitor.visit(allocation) {
                    if let Some(previous) = previous {
                        history.visit_new_allocations(
                            previous,
                            visitor,
                            &mut |metadata, _, allocation, visitor| {
                                visit_metadata::<K>(metadata, allocation, visitor)
                            },
                        );
                    } else {
                        history.visit_allocations(false, visitor, &mut visit_metadata::<K>);
                    }
                }
            },
        );
        self.extra.visit_new_allocations(
            &previous.extra,
            visitor,
            &mut |extra, _, mut allocation, visitor| {
                allocation.bytes += K::extra_owned_allocation_bytes(extra);
                visitor.visit(allocation);
            },
        );
        self.aspect_versions.visit_new_allocations(
            &previous.aspect_versions,
            visitor,
            &mut |versions, _, mut allocation, visitor| {
                allocation.bytes += aspect_version_bytes(versions);
                visitor.visit(allocation);
            },
        );
    }
}
