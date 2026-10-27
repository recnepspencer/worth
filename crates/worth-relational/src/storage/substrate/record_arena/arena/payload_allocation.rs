//! Owned bytes inside record values; shared column/index nodes are measured separately.
use super::RecordKind;
use crate::symbols::data::Symbol;
use std::collections::BTreeMap;

pub(super) fn history_prefix_bytes(prefix: &Option<super::super::RecordHistoryPrefix>) -> u64 {
    prefix.as_ref().map_or(0, |prefix| {
        crate::storage::substrate::StorageAllocation::<super::super::RecordHistoryOpening>::layout_bytes()
            + prefix.history_opening.owned_allocation_bytes()
    })
}

pub(super) fn visit_history_prefix(
    prefix: &Option<super::super::RecordHistoryPrefix>,
    allocation: crate::storage::substrate::StorageAllocationObservation,
    visitor: &mut dyn crate::storage::substrate::StorageAllocationVisitor,
) {
    if visitor.visit(allocation) {
        visit_prefix_opening(prefix, allocation.unique, visitor);
    }
}

pub(super) fn visit_new_history_prefix(
    prefix: &Option<super::super::RecordHistoryPrefix>,
    previous: Option<&Option<super::super::RecordHistoryPrefix>>,
    allocation: crate::storage::substrate::StorageAllocationObservation,
    visitor: &mut dyn crate::storage::substrate::StorageAllocationVisitor,
) {
    if visitor.visit(allocation)
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

pub(super) fn metadata_history_bytes<K: RecordKind>(
    history: &crate::storage::substrate::SharedColumn<K::Meta>,
) -> u64 {
    history.allocation_bytes().saturating_add(
        history
            .iter()
            .map(K::metadata_owned_allocation_bytes)
            .sum::<u64>(),
    )
}

pub(super) fn visit_metadata<K: RecordKind>(
    metadata: &K::Meta,
    mut allocation: crate::storage::substrate::StorageAllocationObservation,
    visitor: &mut dyn crate::storage::substrate::StorageAllocationVisitor,
) {
    allocation.bytes = allocation
        .bytes
        .saturating_add(K::metadata_owned_allocation_bytes(metadata));
    visitor.visit(allocation);
}

pub(super) fn aspect_version_bytes(versions: &BTreeMap<Symbol, u64>) -> u64 {
    (versions.len() as u64).saturating_mul(std::mem::size_of::<(Symbol, u64)>() as u64)
}

pub(super) fn field_revision_bytes(
    revisions: &Option<BTreeMap<(Symbol, Symbol), crate::storage::data::RelationalFieldRevision>>,
) -> u64 {
    revisions.as_ref().map_or(0, |revisions| {
        (revisions.len() as u64).saturating_mul(std::mem::size_of::<(
            (Symbol, Symbol),
            crate::storage::data::RelationalFieldRevision,
        )>() as u64)
    })
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
