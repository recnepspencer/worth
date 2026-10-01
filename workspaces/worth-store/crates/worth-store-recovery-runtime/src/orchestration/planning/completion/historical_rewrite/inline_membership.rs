//! Authenticate source/result span membership without old segment data, which
//! C.10 may already have retired.

use std::collections::VecDeque;
use worth_store::physical_runtime::BoundedRecoveryFilesystemDiscovery;
use worth_store_physical_format::{
    DurableInlineRecordPlacement, DurablePhysicalRootManifest, PhysicalRecordFormatDeclaration,
    PhysicalRewriteRedo, PhysicalTreeIdentity, RecordSegmentPageManifestEntry,
    SegmentManifestBlockReference,
};

use super::super::super::historical_publication::{discovery_failure, HistoricalFailure};
use crate::integrity_ingress::{
    projection::MembershipProjectionFailure, RecoveryIntegrityIngressTrace,
};
use crate::orchestration::planning::manifest_entry_budget::ManifestEntryBudget;

pub(super) struct SourceInlineSpan {
    placement: DurableInlineRecordPlacement,
    entries: Vec<RecordSegmentPageManifestEntry>,
}

pub(super) fn source(
    discovery: &mut BoundedRecoveryFilesystemDiscovery,
    root: &DurablePhysicalRootManifest,
    budget: &mut ManifestEntryBudget,
    trace: &mut RecoveryIntegrityIngressTrace,
    scratch: &mut u64,
    format: PhysicalRecordFormatDeclaration,
    rewrite: PhysicalRewriteRedo,
    placement: DurableInlineRecordPlacement,
) -> Result<SourceInlineSpan, HistoricalFailure> {
    let pages = super::super::rewrite_span::historical_page_count(rewrite, format)
        .map_err(|_| HistoricalFailure::Invalid)?;
    let start = u32::try_from(rewrite.source_offset() / u64::from(format.page_size().bytes()))
        .map_err(|_| HistoricalFailure::Invalid)?;
    if rewrite.extent_arena().is_some()
        || placement.page_generation() != rewrite.source_placement()
        || placement.segment_generation() != rewrite.source_generation()
    {
        return Err(HistoricalFailure::Invalid);
    }
    let entries = collect(
        discovery,
        root,
        budget,
        trace,
        scratch,
        format,
        placement,
        rewrite.source_generation(),
        start,
        pages,
        None,
    )?;
    Ok(SourceInlineSpan { placement, entries })
}

pub(super) fn verify(
    discovery: &mut BoundedRecoveryFilesystemDiscovery,
    root: &DurablePhysicalRootManifest,
    budget: &mut ManifestEntryBudget,
    trace: &mut RecoveryIntegrityIngressTrace,
    scratch: &mut u64,
    format: PhysicalRecordFormatDeclaration,
    rewrite: PhysicalRewriteRedo,
    placement: DurableInlineRecordPlacement,
    source: &SourceInlineSpan,
) -> Result<(), HistoricalFailure> {
    let pages = super::super::rewrite_span::historical_page_count(rewrite, format)
        .map_err(|_| HistoricalFailure::Invalid)?;
    let entries = collect(
        discovery,
        root,
        budget,
        trace,
        scratch,
        format,
        placement,
        rewrite.destination_generation(),
        0,
        pages,
        Some(pages),
    )?;
    *scratch = scratch.saturating_add(
        (source.entries.capacity() * std::mem::size_of::<RecordSegmentPageManifestEntry>()) as u64,
    );
    verify_inline_coordinates(
        format,
        rewrite,
        source.placement,
        &source.entries,
        placement,
        &entries,
    )
}

pub(super) fn verify_inline_coordinates(
    format: PhysicalRecordFormatDeclaration,
    rewrite: PhysicalRewriteRedo,
    source: DurableInlineRecordPlacement,
    source_entries: &[RecordSegmentPageManifestEntry],
    destination: DurableInlineRecordPlacement,
    result_entries: &[RecordSegmentPageManifestEntry],
) -> Result<(), HistoricalFailure> {
    let pages = super::super::rewrite_span::historical_page_count(rewrite, format)
        .map_err(|_| HistoricalFailure::Invalid)?;
    let start = u32::try_from(rewrite.source_offset() / u64::from(format.page_size().bytes()))
        .map_err(|_| HistoricalFailure::Invalid)?;
    if source.record() != destination.record()
        || source.segment() != destination.segment()
        || source.page() != destination.page()
        || source.slot_cell() != destination.slot_cell()
        || source.segment_page_capacity() != destination.segment_page_capacity()
        || source.payload_bytes() != destination.payload_bytes()
        || source.segment_generation() != rewrite.source_generation()
        || source.page_generation() != rewrite.source_placement()
        || destination.page_generation() != rewrite.destination_placement()
        || destination.segment_generation() != rewrite.destination_generation()
        || rewrite.source_placement().checked_add(1) != Some(rewrite.destination_placement())
        || source_entries.len() != pages as usize
        || result_entries.len() != pages as usize
    {
        return Err(HistoricalFailure::Invalid);
    }
    for (index, (prior, result)) in source_entries.iter().zip(result_entries).enumerate() {
        let index = index as u32;
        if prior.page() != result.page()
            || prior.frame_index() != start + index
            || prior.data_page_count() < start + pages
            || result.frame_index() != index
            || result.data_page_count() != pages
            || prior.data_generation() != rewrite.source_generation()
            || result.data_generation() != rewrite.destination_generation()
            || prior.page_generation().checked_add(1) != Some(result.page_generation())
        {
            return Err(HistoricalFailure::Invalid);
        }
    }
    if source_entries
        .last()
        .is_none_or(|entry| entry.page_cell() != source.page_cell())
        || result_entries
            .last()
            .is_none_or(|entry| entry.page_cell() != destination.page_cell())
    {
        return Err(HistoricalFailure::Invalid);
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn collect(
    discovery: &mut BoundedRecoveryFilesystemDiscovery,
    root: &DurablePhysicalRootManifest,
    budget: &mut ManifestEntryBudget,
    trace: &mut RecoveryIntegrityIngressTrace,
    scratch: &mut u64,
    format: PhysicalRecordFormatDeclaration,
    placement: DurableInlineRecordPlacement,
    generation: u64,
    start: u32,
    pages: u32,
    compact_count: Option<u32>,
) -> Result<Vec<RecordSegmentPageManifestEntry>, HistoricalFailure> {
    let end = start.checked_add(pages).ok_or(HistoricalFailure::Invalid)?;
    let mut pending = root.segment_root().into_iter().collect::<VecDeque<_>>();
    let mut found = vec![None; pages as usize];
    while let Some(reference) = pending.pop_front() {
        let segment = placement.segment().get();
        if reference.first().segment().get() > segment || reference.last().segment().get() < segment
        {
            continue;
        }
        budget
            .consume(1)
            .map_err(|_| HistoricalFailure::ManifestEntries)?;
        let observed = discovery
            .read_segment_membership_block(
                reference.generation(),
                reference.block(),
                u64::from(format.page_size().bytes()),
            )
            .map_err(discovery_failure)?;
        let tree =
            PhysicalTreeIdentity::new(root.tree_identity()).ok_or(HistoricalFailure::Invalid)?;
        let block = crate::integrity_ingress::projection::segment_membership_block(
            &observed,
            discovery.store_identity(),
            format,
            tree,
            reference,
            root.node_capacity(),
            budget.remaining(),
            trace,
        )
        .map_err(|failure| match failure {
            MembershipProjectionFailure::EntryLimit { .. } => HistoricalFailure::ManifestEntries,
            MembershipProjectionFailure::Integrity(_) => HistoricalFailure::Invalid,
        })?;
        if let Some(entries) = block.entries() {
            budget
                .consume(entries.len())
                .map_err(|_| HistoricalFailure::ManifestEntries)?;
            for entry in entries {
                if entry.page_cell().segment_id().get() != segment
                    || entry.data_generation() != generation
                    || !(start..end).contains(&entry.frame_index())
                {
                    continue;
                }
                if entry.data_page_count() < end
                    || compact_count.is_some_and(|count| entry.data_page_count() != count)
                    || found[(entry.frame_index() - start) as usize]
                        .replace(*entry)
                        .is_some()
                {
                    return Err(HistoricalFailure::Invalid);
                }
            }
            *scratch = (*scratch).max(traversal_scratch(
                &pending,
                &found,
                entries.len() * std::mem::size_of::<RecordSegmentPageManifestEntry>(),
            ));
        } else {
            let children = block.children().ok_or(HistoricalFailure::Invalid)?;
            budget
                .consume(children.len())
                .map_err(|_| HistoricalFailure::ManifestEntries)?;
            if children
                .iter()
                .any(|child| child.level().checked_add(1) != Some(reference.level()))
            {
                return Err(HistoricalFailure::Invalid);
            }
            pending.extend(children.iter().copied().filter(|child| {
                child.first().segment().get() <= segment && child.last().segment().get() >= segment
            }));
            *scratch = (*scratch).max(traversal_scratch(
                &pending,
                &found,
                children.len() * std::mem::size_of::<SegmentManifestBlockReference>(),
            ));
        }
    }
    if found.iter().any(Option::is_none)
        || found
            .last()
            .and_then(|entry| *entry)
            .is_none_or(|entry| entry.page_cell() != placement.page_cell())
    {
        return Err(HistoricalFailure::Invalid);
    }
    Ok(found.into_iter().map(Option::unwrap).collect())
}

fn traversal_scratch(
    pending: &VecDeque<SegmentManifestBlockReference>,
    found: &Vec<Option<RecordSegmentPageManifestEntry>>,
    decoded_block_bytes: usize,
) -> u64 {
    // Vec/VecDeque capacities and the decoded block are live together. Also
    // reserve the exact output vector while the Option slots still exist.
    (pending.capacity() * std::mem::size_of::<SegmentManifestBlockReference>()
        + found.capacity() * std::mem::size_of::<Option<RecordSegmentPageManifestEntry>>()
        + found.len() * std::mem::size_of::<RecordSegmentPageManifestEntry>()
        + decoded_block_bytes) as u64
}

#[cfg(test)]
#[path = "inline_membership/tests.rs"]
mod tests;
