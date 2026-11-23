//! Owned bytes inside record values; shared column/index nodes and nested
//! persistent aspect nodes/values are traversed at their physical allocation granularity.
use super::RecordKind;
use crate::symbols::data::Symbol;
use std::collections::BTreeMap;

pub(super) fn visit_history_prefix(
    prefix: &Option<super::super::RecordHistoryPrefix>,
    allocation: crate::storage::substrate::StoredValueObservation,
    visitor: &mut dyn crate::storage::substrate::StorageAllocationVisitor,
) {
    if allocation.visit(visitor) {
        visit_prefix_opening(prefix, allocation.unique, visitor);
    }
}

pub(super) fn visit_new_history_prefix(
    prefix: &Option<super::super::RecordHistoryPrefix>,
    previous: Option<&Option<super::super::RecordHistoryPrefix>>,
    allocation: crate::storage::substrate::StoredValueObservation,
    visitor: &mut dyn crate::storage::substrate::StorageAllocationVisitor,
) {
    if allocation.visit(visitor)
        && prefix.as_ref().map(|prefix| prefix.history_opening.id())
            != previous
                .and_then(|prefix| prefix.as_ref())
                .map(|prefix| prefix.history_opening.id())
    {
        visit_prefix_opening(prefix, allocation.unique, visitor);
    }
}

fn visit_prefix_opening(
    prefix: &Option<super::super::RecordHistoryPrefix>,
    unique: bool,
    visitor: &mut dyn crate::storage::substrate::StorageAllocationVisitor,
) {
    if let Some(prefix) = prefix {
        let mut allocation = prefix.history_opening.observation(unique);
        allocation.bytes += prefix.history_opening.owned_allocation_bytes();
        visitor.visit(allocation);
    }
}

pub(super) fn visit_metadata<K: RecordKind>(
    metadata: &K::Meta,
    allocation: crate::storage::substrate::StoredValueObservation,
    visitor: &mut dyn crate::storage::substrate::StorageAllocationVisitor,
) {
    if allocation.visit(visitor) {
        let extra = K::metadata_extra(metadata);
        let payload = extra.observation(allocation.unique);
        if visitor.visit(payload) {
            K::visit_extra_allocations(extra, payload.unique, visitor);
        }
    }
}

pub(super) fn diagnostic_enrichment_bytes(entries: &BTreeMap<Symbol, String>) -> u64 {
    (entries.len() as u64)
        .saturating_mul(std::mem::size_of::<(Symbol, String)>() as u64)
        .saturating_add(
            entries
                .values()
                .map(|value| value.capacity() as u64)
                .sum::<u64>(),
        )
}
