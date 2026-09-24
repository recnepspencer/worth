use sha2::{Digest, Sha256};
use worth_store::physical_runtime::BoundedRecoveryFilesystemDiscovery;
use worth_store_physical_format::{
    inspect_inline_page_records, CurrentPhysicalRecordPlacement, DurableInlineRecordPlacement,
    PersistedInlineSegmentAllocation, PersistedPhysicalRecoveryProjection,
    PersistedPhysicalRecoveryRootState, PhysicalGeneration, PhysicalGenerationAuthority,
    PhysicalRecordFormatDeclaration, PhysicalSegmentId, RecordArtifactFile,
};
use worth_store_recovery_physics::{PhysicalRedoProjection, PhysicalRewriteAdmission};

use super::decode_record;
use crate::progression::RecoverySelectedSourceInventory;

#[path = "rewrite_span_pages.rs"]
mod pages;
use pages::{
    admit_span, flatten, read_span, restamp_pages, segment_page_count, span_entries, stage,
};

pub(super) fn prove(
    discovery: &mut BoundedRecoveryFilesystemDiscovery,
    selection: &worth_store_recovery_physics::PhysicalSourceSelection,
    format: PhysicalRecordFormatDeclaration,
    byte_limit: u64,
    admission: PhysicalRewriteAdmission,
) -> Result<(), ()> {
    let rewrite = admission.redo();
    let pages = admit_span(rewrite, format).map_err(|_| ())?;
    let record = decode_record(rewrite.record_identity())?;
    let inline = selection
        .page_facts()
        .placements()
        .iter()
        .copied()
        .find_map(|placement| match placement {
            CurrentPhysicalRecordPlacement::Inline(inline)
                if inline.record() == record
                    && inline.page_generation() == rewrite.destination_placement()
                    && inline.segment_generation() == rewrite.destination_generation() =>
            {
                Some(inline)
            }
            _ => None,
        });
    let Some(inline) = inline else {
        return Err(());
    };
    let source = read_span(
        discovery,
        inline.segment().get(),
        rewrite.source_generation(),
        rewrite.source_offset(),
        rewrite.source_length(),
        byte_limit,
    )?;
    let digest: [u8; 32] = Sha256::digest(&source).into();
    if digest != rewrite.source_digest() {
        return Err(());
    }
    let restamped = match restamp_pages(format, &source, pages, rewrite) {
        Ok(pages) => pages,
        Err(()) => return Err(()),
    };
    let destination = read_span(
        discovery,
        inline.segment().get(),
        rewrite.destination_generation(),
        rewrite.destination_offset(),
        rewrite.destination_length(),
        byte_limit,
    )?;
    if destination != flatten(&restamped) {
        return Err(());
    }
    let tail = restamped.last().ok_or(())?;
    if tail.destination != inline.page_cell() {
        return Err(());
    }
    let records = inspect_inline_page_records(format, &tail.bytes).map_err(|_| ())?;
    if !records.iter().any(|found| found.record() == record) {
        return Err(());
    }
    Ok(())
}

pub(super) fn project(
    discovery: &mut BoundedRecoveryFilesystemDiscovery,
    selection: &worth_store_recovery_physics::PhysicalSourceSelection,
    source: &RecoverySelectedSourceInventory,
    format: PhysicalRecordFormatDeclaration,
    byte_limit: u64,
    admission: PhysicalRewriteAdmission,
) -> Result<PhysicalRedoProjection, ()> {
    let rewrite = admission.redo();
    let pages = admit_span(rewrite, format).map_err(|_| ())?;
    let selected_generation = selection.root().selected().selector().root_generation();
    if rewrite.source_root_generation() != selected_generation
        || rewrite.resulting_root_generation() != selected_generation.saturating_add(1)
    {
        return Err(());
    }
    let record = decode_record(rewrite.record_identity())?;
    let placements = selection.page_facts().placements();
    let inline = placements
        .iter()
        .copied()
        .find_map(|placement| match placement {
            CurrentPhysicalRecordPlacement::Inline(inline)
                if inline.record() == record
                    && inline.page_generation() == rewrite.source_placement()
                    && inline.segment_generation() == rewrite.source_generation() =>
            {
                Some(inline)
            }
            _ => None,
        });
    let Some(inline) = inline else {
        return Err(());
    };
    let selected = source
        .segment_pages
        .get(&(inline.segment().get(), inline.page().get()))
        .copied()
        .ok_or(())?;
    let tail = selected.entry;
    let page_bytes = u64::from(format.page_size().bytes());
    let start = u32::try_from(rewrite.source_offset() / page_bytes).map_err(|_| ())?;
    if tail.page_generation() != rewrite.source_placement()
        || tail.data_generation() != rewrite.source_generation()
        || tail.data_page_count() == 0
        || tail.frame_index() >= tail.data_page_count()
        || tail.frame_index() + 1 != start + pages
    {
        return Err(());
    }
    let entries = span_entries(
        source,
        inline.segment().get(),
        rewrite.source_generation(),
        start,
        pages,
    )?;
    let bytes = read_span(
        discovery,
        inline.segment().get(),
        rewrite.source_generation(),
        rewrite.source_offset(),
        rewrite.source_length(),
        byte_limit,
    )?;
    let digest: [u8; 32] = Sha256::digest(&bytes).into();
    if digest != rewrite.source_digest() {
        return Err(());
    }
    let restamped = match restamp_pages(format, &bytes, pages, rewrite) {
        Ok(pages) => pages,
        Err(()) => return Err(()),
    };
    if restamped.len() != entries.len() {
        return Err(());
    }
    let authority = PhysicalGenerationAuthority::for_canonical_physical_format();
    let segment_id = PhysicalSegmentId::from_raw(inline.segment().get()).map_err(|_| ())?;
    let destination_segment = authority.segment_cell(segment_id).with_segment_generation(
        PhysicalGeneration::from_raw(rewrite.destination_generation()).map_err(|_| ())?,
    );
    let artifact = RecordArtifactFile::Segment {
        segment: inline.segment().get(),
        generation: rewrite.destination_generation(),
    };
    let mut frames = Vec::new();
    let mut updates = Vec::new();
    let mut destinations = Vec::new();
    for (frame, (entry, page)) in (0_u32..).zip(entries.iter().zip(restamped.iter())) {
        if entry.page_cell() != page.source {
            return Err(());
        }
        stage(
            &mut frames,
            &mut updates,
            &mut destinations,
            artifact,
            destination_segment,
            frame,
            pages,
            page.source,
            page.destination,
            &page.bytes,
            page_bytes,
        )?;
    }
    let mut rebound = Vec::new();
    for placement in placements {
        let CurrentPhysicalRecordPlacement::Inline(existing) = placement else {
            continue;
        };
        let Some(destination) = destinations
            .iter()
            .find(|(source_page, _)| *source_page == existing.page_cell())
            .map(|(_, destination)| *destination)
        else {
            continue;
        };
        rebound.push(
            DurableInlineRecordPlacement::new(
                existing.record(),
                destination_segment,
                destination,
                existing.slot_cell(),
                existing.segment_page_capacity(),
                existing.payload_bytes(),
            )
            .ok_or(())?,
        );
    }
    rebound.sort_by_key(|placement| placement.record());
    if rebound.is_empty() {
        return Err(());
    }
    let allocation = PersistedInlineSegmentAllocation::new(
        destination_segment,
        inline.segment_page_capacity(),
        segment_page_count(placements, inline.segment().get())?,
    )
    .ok_or(())?;
    let root_state = PersistedPhysicalRecoveryRootState::new(
        rewrite.candidate_bytes(),
        1,
        selection.root().selected().manifest().node_capacity(),
        vec![allocation],
        Some(record),
        Some(destination_segment),
    )
    .ok_or(())?;
    let projection = PersistedPhysicalRecoveryProjection::new(
        rewrite.source_root_generation(),
        root_state,
        rebound.iter().map(|placement| placement.record()).collect(),
        frames,
        rebound
            .into_iter()
            .map(CurrentPhysicalRecordPlacement::Inline)
            .collect(),
        updates,
        Vec::new(),
    )
    .ok_or(())?;
    Ok(PhysicalRedoProjection::from_rewrite_materialization(
        admission.operation(),
        admission.group(),
        admission.fate(),
        projection,
    ))
}
