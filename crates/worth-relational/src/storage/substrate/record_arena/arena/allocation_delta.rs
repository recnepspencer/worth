//! Publication delta accounting traverses changed column/index paths, not occupied rows.
use super::payload_allocation::{diagnostic_enrichment_bytes, visit_new_history_prefix};
use super::{RecordArena, RecordKind};
use crate::storage::substrate::{
    visit_inline_value, visit_new_inline_value, StorageAllocationVisitor,
};

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
                allocation.add_owned_bytes(diagnostic_enrichment_bytes(entries));
                allocation.visit(visitor);
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
        self.history_prefixes.visit_new_allocations(
            &previous.history_prefixes,
            visitor,
            &mut visit_new_history_prefix,
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
                if allocation.visit(visitor) {
                    if let Some(previous) = previous {
                        history.visit_new_allocations(
                            previous,
                            visitor,
                            &mut |metadata, _, allocation, visitor| {
                                let _ = metadata;
                                allocation.visit(visitor);
                            },
                        );
                    } else {
                        history.visit_allocations(false, visitor, &mut |_, allocation, visitor| {
                            allocation.visit(visitor);
                        });
                    }
                }
            },
        );
        self.extra
            .visit_new_allocations(&previous.extra, visitor, &mut |_, _, _, _| {});
        self.visit_new_extra_allocations(previous, visitor);
        self.aspect_versions.visit_new_allocations(
            &previous.aspect_versions,
            visitor,
            &mut |versions, previous, allocation, visitor| {
                if let Some(previous) = previous {
                    versions.visit_new_allocations(previous, visitor, &mut visit_new_inline_value);
                } else {
                    versions.visit_allocations(allocation.unique, visitor, &mut visit_inline_value);
                }
            },
        );
        self.field_revisions.visit_new_allocations(
            &previous.field_revisions,
            visitor,
            &mut |revisions, previous, allocation, visitor| {
                if let Some(revisions) = revisions {
                    if let Some(previous) = previous.and_then(Option::as_ref) {
                        revisions.visit_new_allocations(
                            previous,
                            visitor,
                            &mut visit_new_inline_value,
                        );
                    } else {
                        revisions.visit_allocations(
                            allocation.unique,
                            visitor,
                            &mut visit_inline_value,
                        );
                    }
                }
            },
        );
    }
}
