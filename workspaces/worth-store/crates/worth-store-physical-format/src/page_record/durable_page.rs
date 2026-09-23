use std::collections::BTreeSet;
use std::ops::Range;

use crate::record_framing::{
    decode_durable_frame, initialize_durable_frame, reseal_durable_frame,
    DURABLE_FRAME_HEADER_BYTES,
};
use crate::{
    DurableFrameDenial, DurableFrameKind, PageGenerationCell, PersistedRecordIdentity,
    PhysicalGeneration, PhysicalGenerationAuthority, PhysicalPageId,
    PhysicalRecordFormatDeclaration, PhysicalRecordSlot, PhysicalSegmentId, SlotGenerationCell,
};

pub const DURABLE_INLINE_PAGE_PREFIX_BYTES: usize = 24;
pub const DURABLE_INLINE_SLOT_BYTES: usize = 40;

const PAGE_PREFIX_BYTES: usize = DURABLE_INLINE_PAGE_PREFIX_BYTES;
const SLOT_BYTES: usize = DURABLE_INLINE_SLOT_BYTES;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InlineRecordAppend<'a> {
    record: PersistedRecordIdentity,
    slot: SlotGenerationCell,
    bytes: &'a [u8],
}

impl<'a> InlineRecordAppend<'a> {
    pub const fn new(
        record: PersistedRecordIdentity,
        slot: SlotGenerationCell,
        bytes: &'a [u8],
    ) -> Self {
        Self {
            record,
            slot,
            bytes,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AppendedInlineRecord {
    pub slot: PhysicalRecordSlot,
    pub slot_generation: u64,
    pub payload_bytes: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InlinePageGeometry {
    page: PageGenerationCell,
    slot_count: u16,
    free_bytes: u32,
}

impl InlinePageGeometry {
    pub const fn page_cell(self) -> PageGenerationCell {
        self.page
    }
    pub const fn segment(self) -> PhysicalSegmentId {
        self.page.segment_id()
    }
    pub const fn page(self) -> PhysicalPageId {
        self.page.page_id()
    }
    pub const fn generation(self) -> u64 {
        self.page.generation().get()
    }
    pub const fn slot_count(self) -> u16 {
        self.slot_count
    }
    pub const fn free_bytes(self) -> u32 {
        self.free_bytes
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InlineRecordRange {
    range: Range<usize>,
}

impl InlineRecordRange {
    pub fn range(&self) -> Range<usize> {
        self.range.clone()
    }
}

pub fn inspect_inline_page(
    format: PhysicalRecordFormatDeclaration,
    bytes: &[u8],
) -> Result<InlinePageGeometry, InlinePageDenial> {
    let (found_format, frame) = decode_durable_frame(bytes, DurableFrameKind::InlinePage)
        .map_err(InlinePageDenial::Frame)?;
    if found_format != format
        || bytes.len() != format.page_bytes() as usize
        || frame.payload.len() < PAGE_PREFIX_BYTES
        || frame.payload[18..PAGE_PREFIX_BYTES] != [0; 6]
    {
        return Err(InlinePageDenial::InvalidGeometry);
    }
    let segment =
        PhysicalSegmentId::from_raw(u64::from_le_bytes(frame.payload[..8].try_into().unwrap()))
            .map_err(|_| InlinePageDenial::InvalidPageIdentity)?;
    let page =
        PhysicalPageId::from_raw(u64::from_le_bytes(frame.payload[8..16].try_into().unwrap()))
            .map_err(|_| InlinePageDenial::InvalidPageIdentity)?;
    let generation = PhysicalGeneration::from_raw(frame.identity)
        .map_err(|_| InlinePageDenial::InvalidPageIdentity)?;
    let count = u16::from_le_bytes(frame.payload[16..18].try_into().unwrap());
    let directory_end = PAGE_PREFIX_BYTES + usize::from(count) * SLOT_BYTES;
    validate_slot_directory(frame.payload, count, directory_end)?;
    let data_start = data_start(frame.payload, count);
    Ok(InlinePageGeometry {
        page: PhysicalGenerationAuthority::for_canonical_physical_format()
            .page_cell(segment, page)
            .with_page_generation(generation),
        slot_count: count,
        free_bytes: u32::try_from(data_start - directory_end)
            .map_err(|_| InlinePageDenial::InvalidGeometry)?,
    })
}

/// Appends records to a candidate page without moving any existing payload.
pub fn append_inline_records_owned(
    format: PhysicalRecordFormatDeclaration,
    candidate_page: PageGenerationCell,
    existing: Option<Vec<u8>>,
    records: &[InlineRecordAppend<'_>],
) -> Result<(Vec<u8>, Vec<AppendedInlineRecord>), InlinePageDenial> {
    if records.is_empty() {
        return Err(InlinePageDenial::InvalidGeometry);
    }
    let mut page = match existing {
        Some(page) => {
            let old = inspect_inline_page(format, &page)?;
            if old.segment() != candidate_page.segment_id()
                || old.page() != candidate_page.page_id()
                || old.generation() >= candidate_page.generation().get()
            {
                return Err(InlinePageDenial::InvalidPageIdentity);
            }
            page
        }
        None => new_inline_page(format, candidate_page),
    };
    let payload = &mut page[DURABLE_FRAME_HEADER_BYTES..];
    let old_count = u16::from_le_bytes(payload[16..18].try_into().unwrap());
    let old_directory_end = PAGE_PREFIX_BYTES + usize::from(old_count) * SLOT_BYTES;
    validate_slot_directory(payload, old_count, old_directory_end)?;
    let mut data_start = data_start(payload, old_count);
    let new_count = usize::from(old_count)
        .checked_add(records.len())
        .and_then(|count| u16::try_from(count).ok())
        .ok_or(InlinePageDenial::InvalidGeometry)?;
    let directory_end = PAGE_PREFIX_BYTES + usize::from(new_count) * SLOT_BYTES;
    let added_bytes = records.iter().try_fold(0_usize, |total, record| {
        total
            .checked_add(record.bytes.len())
            .ok_or(InlinePageDenial::InvalidGeometry)
    })?;
    if directory_end
        .checked_add(added_bytes)
        .ok_or(InlinePageDenial::InvalidGeometry)?
        > data_start
    {
        return Err(InlinePageDenial::PageFull);
    }
    let mut identities = existing_identities(payload, old_count);
    let mut appended = Vec::with_capacity(records.len());
    for (index, record) in records.iter().enumerate() {
        let slot_number = usize::from(old_count) + index + 1;
        let slot = PhysicalRecordSlot::from_raw(slot_number as u16)
            .map_err(|_| InlinePageDenial::InvalidSlot)?;
        if record.slot.segment_id() != candidate_page.segment_id()
            || record.slot.page_id() != candidate_page.page_id()
            || record.slot.slot() != slot
            || !identities.insert(record.record)
        {
            return Err(InlinePageDenial::InvalidSlot);
        }
        data_start -= record.bytes.len();
        payload[data_start..data_start + record.bytes.len()].copy_from_slice(record.bytes);
        let base = PAGE_PREFIX_BYTES + (slot_number - 1) * SLOT_BYTES;
        payload[base..base + 16].copy_from_slice(&record.record.allocation_epoch());
        payload[base + 16..base + 24].copy_from_slice(&record.record.ordinal().to_le_bytes());
        payload[base + 24..base + 28].copy_from_slice(&(data_start as u32).to_le_bytes());
        payload[base + 28..base + 32].copy_from_slice(&(record.bytes.len() as u32).to_le_bytes());
        payload[base + 32..base + 40]
            .copy_from_slice(&record.slot.generation().get().to_le_bytes());
        appended.push(AppendedInlineRecord {
            slot,
            slot_generation: record.slot.generation().get(),
            payload_bytes: record.bytes.len() as u32,
        });
    }
    payload[16..18].copy_from_slice(&new_count.to_le_bytes());
    reseal_durable_frame(
        &mut page,
        DurableFrameKind::InlinePage,
        format,
        candidate_page.generation().get(),
    );
    Ok((page, appended))
}

/// Copies an admitted inline page and reseals it at the successor page generation.
///
/// Record slots and payload bytes stay in place. The previous page LSN is preserved
/// until the WAL bind stamps the rewrite's physical coverage.
pub fn restamp_inline_page_generation(
    format: PhysicalRecordFormatDeclaration,
    page: &[u8],
    generation: u64,
) -> Result<Vec<u8>, InlinePageDenial> {
    if page.len() != format.page_bytes() as usize {
        return Err(InlinePageDenial::InvalidGeometry);
    }
    let current = inspect_inline_page(format, page)?;
    if generation <= current.page_cell().generation().get() {
        return Err(InlinePageDenial::InvalidPageIdentity);
    }
    let mut bytes = page.to_vec();
    reseal_durable_frame(&mut bytes, DurableFrameKind::InlinePage, format, generation);
    Ok(bytes)
}

pub fn encode_inline_page(
    format: PhysicalRecordFormatDeclaration,
    page: PageGenerationCell,
    records: &[InlineRecordAppend<'_>],
) -> Result<Vec<u8>, InlinePageDenial> {
    if records.is_empty() {
        return Ok(new_inline_page(format, page));
    }
    append_inline_records_owned(format, page, None, records).map(|(page, _)| page)
}

pub fn decode_inline_record(
    bytes: &[u8],
    expected_record: PersistedRecordIdentity,
    expected_page: PageGenerationCell,
    expected_slot: SlotGenerationCell,
) -> Result<(InlineRecordRange, PhysicalRecordFormatDeclaration), InlinePageDenial> {
    let geometry = inspect_inline_page_from_frame(bytes)?;
    if geometry.page_cell() != expected_page
        || expected_slot.segment_id() != expected_page.segment_id()
        || expected_slot.page_id() != expected_page.page_id()
    {
        return Err(InlinePageDenial::InvalidPageIdentity);
    }
    let (format, frame) = decode_durable_frame(bytes, DurableFrameKind::InlinePage)
        .map_err(InlinePageDenial::Frame)?;
    let slot_index = usize::from(expected_slot.slot().get() - 1);
    if slot_index >= usize::from(geometry.slot_count()) {
        return Err(InlinePageDenial::InvalidSlot);
    }
    let base = PAGE_PREFIX_BYTES + slot_index * SLOT_BYTES;
    let record = PersistedRecordIdentity::new(
        frame.payload[base..base + 16].try_into().unwrap(),
        u64::from_le_bytes(frame.payload[base + 16..base + 24].try_into().unwrap()),
    )
    .ok_or(InlinePageDenial::InvalidRecordIdentity)?;
    if record != expected_record {
        return Err(InlinePageDenial::RecordIdentityMismatch);
    }
    let generation = u64::from_le_bytes(frame.payload[base + 32..base + 40].try_into().unwrap());
    if generation != expected_slot.generation().get() {
        return Err(InlinePageDenial::SlotGenerationMismatch);
    }
    let offset =
        u32::from_le_bytes(frame.payload[base + 24..base + 28].try_into().unwrap()) as usize;
    let length =
        u32::from_le_bytes(frame.payload[base + 28..base + 32].try_into().unwrap()) as usize;
    Ok((
        InlineRecordRange {
            range: DURABLE_FRAME_HEADER_BYTES + offset
                ..DURABLE_FRAME_HEADER_BYTES + offset + length,
        },
        format,
    ))
}

fn inspect_inline_page_from_frame(bytes: &[u8]) -> Result<InlinePageGeometry, InlinePageDenial> {
    let (format, _) = decode_durable_frame(bytes, DurableFrameKind::InlinePage)
        .map_err(InlinePageDenial::Frame)?;
    inspect_inline_page(format, bytes)
}

fn new_inline_page(format: PhysicalRecordFormatDeclaration, page: PageGenerationCell) -> Vec<u8> {
    let mut frame = initialize_durable_frame(
        DurableFrameKind::InlinePage,
        format,
        page.generation().get(),
        format.page_bytes() as usize - DURABLE_FRAME_HEADER_BYTES,
    );
    let payload = &mut frame[DURABLE_FRAME_HEADER_BYTES..];
    payload[..8].copy_from_slice(&page.segment_id().get().to_le_bytes());
    payload[8..16].copy_from_slice(&page.page_id().get().to_le_bytes());
    reseal_durable_frame(
        &mut frame,
        DurableFrameKind::InlinePage,
        format,
        page.generation().get(),
    );
    frame
}

fn data_start(payload: &[u8], count: u16) -> usize {
    if count == 0 {
        payload.len()
    } else {
        let last = PAGE_PREFIX_BYTES + (usize::from(count) - 1) * SLOT_BYTES;
        u32::from_le_bytes(payload[last + 24..last + 28].try_into().unwrap()) as usize
    }
}

fn existing_identities(payload: &[u8], count: u16) -> BTreeSet<PersistedRecordIdentity> {
    (0..usize::from(count))
        .filter_map(|slot| {
            let base = PAGE_PREFIX_BYTES + slot * SLOT_BYTES;
            PersistedRecordIdentity::new(
                payload[base..base + 16].try_into().unwrap(),
                u64::from_le_bytes(payload[base + 16..base + 24].try_into().unwrap()),
            )
        })
        .collect()
}

fn validate_slot_directory(
    payload: &[u8],
    count: u16,
    directory_end: usize,
) -> Result<(), InlinePageDenial> {
    if directory_end > payload.len() {
        return Err(InlinePageDenial::InvalidGeometry);
    }
    let mut identities = BTreeSet::new();
    let mut previous_start = payload.len();
    for slot in 0..usize::from(count) {
        let base = PAGE_PREFIX_BYTES + slot * SLOT_BYTES;
        let record = PersistedRecordIdentity::new(
            payload[base..base + 16].try_into().unwrap(),
            u64::from_le_bytes(payload[base + 16..base + 24].try_into().unwrap()),
        )
        .ok_or(InlinePageDenial::InvalidRecordIdentity)?;
        let offset = u32::from_le_bytes(payload[base + 24..base + 28].try_into().unwrap()) as usize;
        let length = u32::from_le_bytes(payload[base + 28..base + 32].try_into().unwrap()) as usize;
        let generation = u64::from_le_bytes(payload[base + 32..base + 40].try_into().unwrap());
        let end = offset
            .checked_add(length)
            .ok_or(InlinePageDenial::InvalidGeometry)?;
        if generation == 0
            || !identities.insert(record)
            || offset < directory_end
            || end > previous_start
            || end > payload.len()
        {
            return Err(InlinePageDenial::InvalidGeometry);
        }
        if payload[end..previous_start].iter().any(|byte| *byte != 0) {
            return Err(InlinePageDenial::ReservedFieldNonZero);
        }
        previous_start = offset;
    }
    if payload[directory_end..previous_start]
        .iter()
        .any(|byte| *byte != 0)
    {
        return Err(InlinePageDenial::ReservedFieldNonZero);
    }
    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InlinePageDenial {
    Frame(DurableFrameDenial),
    InvalidGeometry,
    InvalidPageIdentity,
    PageFull,
    InvalidSlot,
    InvalidRecordIdentity,
    RecordIdentityMismatch,
    SlotGenerationMismatch,
    ReservedFieldNonZero,
}
