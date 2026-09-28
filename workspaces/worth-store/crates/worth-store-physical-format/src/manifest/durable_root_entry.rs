use crate::{
    PersistedRecordIdentity, PhysicalExtentId, PhysicalGeneration, PhysicalGenerationAuthority,
    PhysicalPageId, PhysicalRecordSlot, PhysicalSegmentId,
};

use super::durable_root::RootManifestDenial;
use super::durable_root_placement::{
    CurrentPhysicalRecordPlacement, DurableExtentRecordPlacement, DurableInlineRecordPlacement,
};

pub(super) fn encode_entry(target: &mut [u8], entry: CurrentPhysicalRecordPlacement) {
    let record = entry.record();
    target[..16].copy_from_slice(&record.allocation_epoch());
    target[16..24].copy_from_slice(&record.ordinal().to_le_bytes());
    match entry {
        CurrentPhysicalRecordPlacement::Inline(value) => {
            target[24] = 1;
            target[32..40].copy_from_slice(&value.segment().get().to_le_bytes());
            target[40..48].copy_from_slice(&value.page().get().to_le_bytes());
            target[48..56].copy_from_slice(&value.segment_generation().to_le_bytes());
            target[56..64].copy_from_slice(&value.page_generation().to_le_bytes());
            target[64..72].copy_from_slice(&value.slot_generation().to_le_bytes());
            target[72..80].copy_from_slice(&value.payload_bytes().to_le_bytes());
            target[80..84].copy_from_slice(&value.segment_page_capacity().to_le_bytes());
            target[84..86].copy_from_slice(&value.slot().get().to_le_bytes());
        }
        CurrentPhysicalRecordPlacement::Extent(value) => {
            target[24] = 2;
            target[32..40].copy_from_slice(&value.arena_range().arena().get().to_le_bytes());
            target[40..48].copy_from_slice(&value.extent().get().to_le_bytes());
            target[48..56].copy_from_slice(&value.extent_generation().to_le_bytes());
            target[56..64].copy_from_slice(&value.arena_range().offset().to_le_bytes());
            target[64..72].copy_from_slice(&value.arena_range().length().to_le_bytes());
            target[72..80].copy_from_slice(&value.payload_bytes().to_le_bytes());
        }
    }
}

pub(super) fn decode_entry(
    bytes: &[u8],
) -> Result<CurrentPhysicalRecordPlacement, RootManifestDenial> {
    if bytes[25..32] != [0; 7] || bytes[86..88] != [0; 2] {
        return Err(RootManifestDenial::ReservedFieldNonZero);
    }
    let record = PersistedRecordIdentity::new(
        bytes[..16].try_into().unwrap(),
        u64::from_le_bytes(bytes[16..24].try_into().unwrap()),
    )
    .ok_or(RootManifestDenial::InvalidRecordIdentity)?;
    let authority = PhysicalGenerationAuthority::for_canonical_physical_format();
    match bytes[24] {
        1 => decode_inline_entry(bytes, record, authority),
        2 => decode_extent_entry(bytes, record, authority),
        _ => Err(RootManifestDenial::InvalidPlacement),
    }
}

fn decode_inline_entry(
    bytes: &[u8],
    record: PersistedRecordIdentity,
    authority: PhysicalGenerationAuthority,
) -> Result<CurrentPhysicalRecordPlacement, RootManifestDenial> {
    let segment =
        PhysicalSegmentId::from_raw(u64::from_le_bytes(bytes[32..40].try_into().unwrap()))
            .map_err(|_| RootManifestDenial::InvalidPlacement)?;
    let page = PhysicalPageId::from_raw(u64::from_le_bytes(bytes[40..48].try_into().unwrap()))
        .map_err(|_| RootManifestDenial::InvalidPlacement)?;
    let segment_generation = generation(bytes, 48)?;
    let page_generation = generation(bytes, 56)?;
    let slot_generation = generation(bytes, 64)?;
    let slot = PhysicalRecordSlot::from_raw(u16::from_le_bytes(bytes[84..86].try_into().unwrap()))
        .map_err(|_| RootManifestDenial::InvalidPlacement)?;
    DurableInlineRecordPlacement::new(
        record,
        authority
            .segment_cell(segment)
            .with_segment_generation(segment_generation),
        authority
            .page_cell(segment, page)
            .with_page_generation(page_generation),
        authority
            .slot_cell(segment, page, slot)
            .with_slot_generation(slot_generation),
        u32::from_le_bytes(bytes[80..84].try_into().unwrap()),
        u64::from_le_bytes(bytes[72..80].try_into().unwrap()),
    )
    .map(CurrentPhysicalRecordPlacement::Inline)
    .ok_or(RootManifestDenial::InvalidPlacement)
}

fn decode_extent_entry(
    bytes: &[u8],
    record: PersistedRecordIdentity,
    authority: PhysicalGenerationAuthority,
) -> Result<CurrentPhysicalRecordPlacement, RootManifestDenial> {
    if bytes[80..86] != [0; 6] {
        return Err(RootManifestDenial::ReservedFieldNonZero);
    }
    let extent = PhysicalExtentId::from_raw(u64::from_le_bytes(bytes[40..48].try_into().unwrap()))
        .map_err(|_| RootManifestDenial::InvalidPlacement)?;
    DurableExtentRecordPlacement::new(
        record,
        authority
            .record_extent_cell(extent)
            .with_extent_generation(generation(bytes, 48)?),
        u64::from_le_bytes(bytes[72..80].try_into().unwrap()),
        crate::ExtentArenaRange::new(
            crate::ExtentArenaId::new(u64::from_le_bytes(bytes[32..40].try_into().unwrap()))
                .ok_or(RootManifestDenial::InvalidPlacement)?,
            u64::from_le_bytes(bytes[56..64].try_into().unwrap()),
            u64::from_le_bytes(bytes[64..72].try_into().unwrap()),
        )
        .ok_or(RootManifestDenial::InvalidPlacement)?,
    )
    .map(CurrentPhysicalRecordPlacement::Extent)
    .ok_or(RootManifestDenial::InvalidPlacement)
}

fn generation(bytes: &[u8], offset: usize) -> Result<PhysicalGeneration, RootManifestDenial> {
    PhysicalGeneration::from_raw(u64::from_le_bytes(
        bytes[offset..offset + 8].try_into().unwrap(),
    ))
    .map_err(|_| RootManifestDenial::InvalidPlacement)
}
