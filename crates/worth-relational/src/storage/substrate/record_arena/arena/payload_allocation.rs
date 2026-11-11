//! Owned bytes inside record values; shared column/index nodes are measured separately.
//! An extra carrier adds its checked declared bound for nested aspect backing.
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
        let mut payload = extra.observation(allocation.unique);
        payload.add_checked_nested_bound(K::extra_reachable_bound_bytes(extra));
        visitor.visit(payload);
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
