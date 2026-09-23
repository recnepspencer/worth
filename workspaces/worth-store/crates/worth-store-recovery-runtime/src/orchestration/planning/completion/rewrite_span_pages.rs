use worth_store::physical_runtime::BoundedRecoveryFilesystemDiscovery;
use worth_store_physical_format::{
    decode_data_frame_page_lsn, encode_data_frame_page_lsn, inspect_inline_page,
    restamp_inline_page_generation, CurrentPhysicalRecordPlacement, DurableFrameKind,
    PageGenerationCell, PersistedPhysicalDataFrameSubject, PersistedPhysicalRecoveryFrame,
    PhysicalGeneration, PhysicalGenerationAuthority, PhysicalPageLsn,
    PhysicalRecordFormatDeclaration, RecordArtifactFile, RecordFrameCoordinate,
    RecordSegmentPageManifestEntry, SegmentGenerationCell,
};

use crate::progression::RecoverySelectedSourceInventory;

const SPAN_LIMIT: u64 = 256 * 1024;

#[cfg(test)]
#[path = "rewrite_span_admission_tests.rs"]
mod tests;

pub(super) struct RestampedPage {
    pub(super) source: PageGenerationCell,
    pub(super) destination: PageGenerationCell,
    pub(super) bytes: Vec<u8>,
}

/// Why a sealed span redo cannot be materialized.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum SpanAdmissionDenial {
    Empty,
    LengthMismatch,
    OverSpanLimit,
    PartialPage,
    /// The span becomes its own compact generation starting at frame 0; any
    /// other destination offset would publish frames where the membership
    /// entries do not point.
    OffsetMismatch,
    Misaligned,
}

pub(super) fn admit_span(
    rewrite: worth_store_physical_format::PhysicalRewriteRedo,
    format: PhysicalRecordFormatDeclaration,
) -> Result<u32, SpanAdmissionDenial> {
    let page_bytes = format.page_size().bytes();
    let length = rewrite.source_length();
    if page_bytes == 0 || length == 0 {
        return Err(SpanAdmissionDenial::Empty);
    }
    if rewrite.destination_length() != length {
        return Err(SpanAdmissionDenial::LengthMismatch);
    }
    if u64::from(length) > SPAN_LIMIT {
        return Err(SpanAdmissionDenial::OverSpanLimit);
    }
    if length % page_bytes != 0 {
        return Err(SpanAdmissionDenial::PartialPage);
    }
    if rewrite.destination_offset() != 0 {
        return Err(SpanAdmissionDenial::OffsetMismatch);
    }
    if rewrite.source_offset() % u64::from(page_bytes) != 0 {
        return Err(SpanAdmissionDenial::Misaligned);
    }
    Ok(length / page_bytes)
}

pub(super) fn restamp_pages(
    format: PhysicalRecordFormatDeclaration,
    source: &[u8],
    pages: u32,
    rewrite: worth_store_physical_format::PhysicalRewriteRedo,
) -> Result<Vec<RestampedPage>, ()> {
    let page_bytes = format.page_size().bytes() as usize;
    if source.len() != page_bytes * pages as usize {
        return Err(());
    }
    let mut restamped = Vec::with_capacity(pages as usize);
    for index in 0..pages {
        let start = index as usize * page_bytes;
        let slice = &source[start..start + page_bytes];
        let geometry = inspect_inline_page(format, slice).map_err(|_| ())?;
        let source_cell = geometry.page_cell();
        let next = source_cell.generation().get().checked_add(1).ok_or(())?;
        if index + 1 == pages
            && (source_cell.generation().get() != rewrite.source_placement()
                || next != rewrite.destination_placement())
        {
            return Err(());
        }
        let generation = PhysicalGeneration::from_raw(next).map_err(|_| ())?;
        let mut bytes =
            restamp_inline_page_generation(format, slice, generation.get()).map_err(|_| ())?;
        encode_data_frame_page_lsn(
            &mut bytes,
            DurableFrameKind::InlinePage,
            PhysicalPageLsn::new(rewrite.page_lsn()),
        )
        .map_err(|_| ())?;
        let destination = PhysicalGenerationAuthority::for_canonical_physical_format()
            .page_cell(source_cell.segment_id(), source_cell.page_id())
            .with_page_generation(generation);
        let stamped = inspect_inline_page(format, &bytes).map_err(|_| ())?;
        if stamped.page_cell() != destination {
            return Err(());
        }
        let page_lsn =
            decode_data_frame_page_lsn(&bytes, DurableFrameKind::InlinePage).map_err(|_| ())?;
        if page_lsn.get() != rewrite.page_lsn() {
            return Err(());
        }
        restamped.push(RestampedPage {
            source: source_cell,
            destination,
            bytes,
        });
    }
    Ok(restamped)
}

pub(super) fn span_entries(
    source: &RecoverySelectedSourceInventory,
    segment: u64,
    data_generation: u64,
    start: u32,
    pages: u32,
) -> Result<Vec<RecordSegmentPageManifestEntry>, ()> {
    let mut found = Vec::new();
    for page in source
        .segment_pages
        .range((segment, 0)..=(segment, u64::MAX))
        .map(|(_, page)| page)
    {
        let entry = page.entry;
        if entry.data_generation() != data_generation {
            continue;
        }
        if entry.frame_index() >= start && entry.frame_index() < start + pages {
            found.push(entry);
        }
    }
    found.sort_by_key(|entry| entry.frame_index());
    if found.len() != pages as usize {
        return Err(());
    }
    if found
        .iter()
        .enumerate()
        .any(|(index, entry)| entry.frame_index() != start + index as u32)
    {
        return Err(());
    }
    Ok(found)
}

pub(super) fn read_span(
    discovery: &mut BoundedRecoveryFilesystemDiscovery,
    segment: u64,
    generation: u64,
    offset: u64,
    length: u32,
    byte_limit: u64,
) -> Result<Vec<u8>, ()> {
    let bytes = discovery
        .read_segment_range(segment, generation, offset, length, byte_limit)
        .map_err(|_| ())?
        .into_bytes()
        .ok_or(())?;
    if bytes.len() != length as usize {
        return Err(());
    }
    Ok(bytes)
}

/// Pages of the segment that hold records in the selected root. A rewrite
/// moves pages between generations but never changes how many the segment
/// uses, and recovery physics checks the allocation against this same set.
pub(super) fn segment_page_count(
    placements: &[CurrentPhysicalRecordPlacement],
    segment: u64,
) -> Result<u32, ()> {
    let pages = placements
        .iter()
        .filter_map(|placement| match placement {
            CurrentPhysicalRecordPlacement::Inline(inline) if inline.segment().get() == segment => {
                Some(inline.page().get())
            }
            _ => None,
        })
        .collect::<std::collections::BTreeSet<_>>();
    u32::try_from(pages.len()).map_err(|_| ())
}

pub(super) fn stage(
    frames: &mut Vec<PersistedPhysicalRecoveryFrame>,
    updates: &mut Vec<RecordSegmentPageManifestEntry>,
    destinations: &mut Vec<(PageGenerationCell, PageGenerationCell)>,
    artifact: RecordArtifactFile,
    segment: SegmentGenerationCell,
    frame_index: u32,
    data_page_count: u32,
    source_cell: PageGenerationCell,
    destination_cell: PageGenerationCell,
    bytes: &[u8],
    page_bytes: u64,
) -> Result<(), ()> {
    let coordinate = RecordFrameCoordinate::new(
        artifact,
        u64::from(frame_index) * page_bytes,
        u32::try_from(page_bytes).map_err(|_| ())?,
    )
    .ok_or(())?;
    frames.push(
        PersistedPhysicalRecoveryFrame::new(
            PersistedPhysicalDataFrameSubject::InlinePage(destination_cell),
            coordinate,
            bytes,
        )
        .ok_or(())?,
    );
    updates.push(
        RecordSegmentPageManifestEntry::new(
            destination_cell,
            segment,
            data_page_count,
            frame_index,
        )
        .ok_or(())?,
    );
    destinations.push((source_cell, destination_cell));
    Ok(())
}

pub(super) fn flatten(pages: &[RestampedPage]) -> Vec<u8> {
    let mut bytes = Vec::new();
    for page in pages {
        bytes.extend_from_slice(&page.bytes);
    }
    bytes
}
