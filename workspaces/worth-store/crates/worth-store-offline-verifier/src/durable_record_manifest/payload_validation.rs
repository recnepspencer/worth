use std::collections::{BTreeMap, HashMap};
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

use sha2::{Digest, Sha256};
use worth_store_physical_format::PhysicalRecordFormatDeclaration;

use super::independent_frame::decode_frame;
use super::observation::{
    OfflineRecordIdentity, OfflineRecordPayloadObservation, OfflineRecordPlacement,
    OfflineSegmentPageMembership,
};
use super::{read_u32, read_u64, OfflineDurableManifestDenial};

const FRAME_HEADER_BYTES: usize = 48;
const PAGE_PREFIX_BYTES: usize = 24;
const SLOT_BYTES: usize = 40;
const EXTENT_METADATA_BYTES: usize = 64;

pub(super) struct OfflinePayloadWalk {
    pub(super) frames_read: u64,
    pub(super) payload_bytes: u64,
    pub(super) payload_digest: [u8; 32],
    pub(super) records: Vec<OfflineRecordPayloadObservation>,
}

pub(super) fn validate_payloads(
    root: &Path,
    format: PhysicalRecordFormatDeclaration,
    placements: &[OfflineRecordPlacement],
    pages: &[OfflineSegmentPageMembership],
    extent_geometry: &BTreeMap<(u64, u64), (u64, u64)>,
) -> Result<OfflinePayloadWalk, OfflineDurableManifestDenial> {
    let page_membership = index_page_membership(pages)?;
    let mut digest = Sha256::new();
    let mut frames_read = 0_u64;
    let mut payload_bytes = 0_u64;
    let mut records = Vec::with_capacity(placements.len());
    for placement in placements {
        digest.update(placement.payload_bytes().to_le_bytes());
        let mut record_digest = PayloadDigesters::new(&mut digest);
        let frames = match *placement {
            OfflineRecordPlacement::Inline {
                record,
                segment,
                page,
                page_generation,
                slot_generation,
                payload_bytes,
                slot,
                ..
            } => {
                let membership = page_membership
                    .get(&(segment, page))
                    .ok_or(OfflineDurableManifestDenial::ReachabilityMismatch)?;
                read_inline_payload(
                    root,
                    format,
                    *membership,
                    InlinePayloadExpectation {
                        record,
                        page_generation,
                        slot_generation,
                        payload_bytes,
                        slot,
                    },
                    &mut record_digest,
                )?;
                1
            }
            OfflineRecordPlacement::Extent {
                record,
                extent,
                generation,
                payload_bytes,
                arena,
                arena_offset,
                ..
            } => read_extent_payload(
                root,
                format,
                ExtentPayloadExpectation {
                    record,
                    extent,
                    generation,
                    logical_bytes: payload_bytes,
                    arena,
                    arena_offset,
                    geometry: *extent_geometry
                        .get(&(arena, arena_offset))
                        .ok_or(OfflineDurableManifestDenial::ReachabilityMismatch)?,
                },
                &mut record_digest,
            )?,
        };
        records.push(record_digest.finish(placement.record(), placement.payload_bytes()));
        frames_read = frames_read.saturating_add(frames);
        payload_bytes = payload_bytes.saturating_add(placement.payload_bytes());
    }
    Ok(OfflinePayloadWalk {
        frames_read,
        payload_bytes,
        payload_digest: digest.finalize().into(),
        records,
    })
}

struct PayloadDigesters<'aggregate> {
    aggregate: &'aggregate mut Sha256,
    record: Sha256,
    prefix: Vec<u8>,
}

impl<'aggregate> PayloadDigesters<'aggregate> {
    fn new(aggregate: &'aggregate mut Sha256) -> Self {
        Self {
            aggregate,
            record: Sha256::new(),
            prefix: Vec::with_capacity(8),
        }
    }

    fn update(&mut self, bytes: &[u8]) {
        let remaining = 8_usize.saturating_sub(self.prefix.len());
        self.prefix
            .extend_from_slice(&bytes[..bytes.len().min(remaining)]);
        self.aggregate.update(bytes);
        self.record.update(bytes);
    }

    fn finish(
        self,
        record: OfflineRecordIdentity,
        payload_bytes: u64,
    ) -> OfflineRecordPayloadObservation {
        OfflineRecordPayloadObservation::new(
            record,
            payload_bytes,
            self.prefix.into_boxed_slice(),
            self.record.finalize().into(),
        )
    }
}

fn index_page_membership(
    pages: &[OfflineSegmentPageMembership],
) -> Result<HashMap<(u64, u64), OfflineSegmentPageMembership>, OfflineDurableManifestDenial> {
    let mut indexed = HashMap::with_capacity(pages.len());
    for page in pages {
        if indexed.insert((page.segment, page.page), *page).is_some() {
            return Err(OfflineDurableManifestDenial::ReachabilityMismatch);
        }
    }
    Ok(indexed)
}

struct InlinePayloadExpectation {
    record: OfflineRecordIdentity,
    page_generation: u64,
    slot_generation: u64,
    payload_bytes: u64,
    slot: u16,
}

struct ExtentPayloadExpectation {
    record: OfflineRecordIdentity,
    extent: u64,
    generation: u64,
    logical_bytes: u64,
    arena: u64,
    arena_offset: u64,
    geometry: (u64, u64),
}

fn read_inline_payload(
    root: &Path,
    format: PhysicalRecordFormatDeclaration,
    membership: OfflineSegmentPageMembership,
    expected: InlinePayloadExpectation,
    digest: &mut PayloadDigesters<'_>,
) -> Result<(), OfflineDurableManifestDenial> {
    if membership.page_generation != expected.page_generation || expected.slot == 0 {
        return Err(OfflineDurableManifestDenial::ReachabilityMismatch);
    }
    let path = root.join(format!(
        "families/records/segments/segment-{:016x}-{:016x}.pages",
        membership.segment, membership.data_generation
    ));
    let mut file = std::fs::File::open(path)
        .map_err(|error| OfflineDurableManifestDenial::Io(error.kind()))?;
    let page_bytes = format.page_size().bytes() as usize;
    file.seek(SeekFrom::Start(
        u64::from(membership.frame_index) * page_bytes as u64,
    ))
    .map_err(|error| OfflineDurableManifestDenial::Io(error.kind()))?;
    let mut bytes = vec![0_u8; page_bytes];
    file.read_exact(&mut bytes)
        .map_err(|error| OfflineDurableManifestDenial::Io(error.kind()))?;
    let frame = decode_frame(&bytes, 3, format)?;
    let payload = frame.payload;
    if frame.identity != expected.page_generation
        || payload.len() + FRAME_HEADER_BYTES != page_bytes
        || read_u64(payload, 0) != membership.segment
        || read_u64(payload, 8) != membership.page
        || payload[18..24] != [0; 6]
    {
        return Err(OfflineDurableManifestDenial::MalformedPayloadFrame);
    }
    let count = u16::from_le_bytes(payload[16..18].try_into().unwrap());
    if expected.slot > count {
        return Err(OfflineDurableManifestDenial::MalformedPayloadFrame);
    }
    let directory_end = PAGE_PREFIX_BYTES + usize::from(count) * SLOT_BYTES;
    let base = PAGE_PREFIX_BYTES + usize::from(expected.slot - 1) * SLOT_BYTES;
    if directory_end > payload.len() || base + SLOT_BYTES > payload.len() {
        return Err(OfflineDurableManifestDenial::MalformedPayloadFrame);
    }
    let found = OfflineRecordIdentity::decode(&payload[base..base + 24]);
    let offset = read_u32(payload, base + 24) as usize;
    let length = read_u32(payload, base + 28) as usize;
    let generation = read_u64(payload, base + 32);
    let end = offset
        .checked_add(length)
        .ok_or(OfflineDurableManifestDenial::MalformedPayloadFrame)?;
    if found != Some(expected.record)
        || generation != expected.slot_generation
        || length as u64 != expected.payload_bytes
        || offset < directory_end
        || end > payload.len()
    {
        return Err(OfflineDurableManifestDenial::MalformedPayloadFrame);
    }
    digest.update(&payload[offset..end]);
    Ok(())
}

fn read_extent_payload(
    root: &Path,
    format: PhysicalRecordFormatDeclaration,
    expected: ExtentPayloadExpectation,
    digest: &mut PayloadDigesters<'_>,
) -> Result<u64, OfflineDurableManifestDenial> {
    let path = root.join(format!(
        "families/records/arenas/arena-{:016x}.data",
        expected.arena
    ));
    let mut file = std::fs::File::open(path)
        .map_err(|error| OfflineDurableManifestDenial::Io(error.kind()))?;
    let capacity = format.page_size().bytes() as usize - FRAME_HEADER_BYTES - EXTENT_METADATA_BYTES;
    let mut offset = 0_u64;
    let mut ordinal = 1_u32;
    while offset < expected.logical_bytes {
        let position = expected.arena_offset
            + expected.geometry.0
            + u64::from(ordinal - 1) * expected.geometry.1;
        file.seek(SeekFrom::Start(position))
            .map_err(|error| OfflineDurableManifestDenial::Io(error.kind()))?;
        let length =
            usize::try_from((expected.logical_bytes - offset).min(capacity as u64)).unwrap();
        let mut bytes = vec![0_u8; FRAME_HEADER_BYTES + EXTENT_METADATA_BYTES + length];
        file.read_exact(&mut bytes)
            .map_err(|error| OfflineDurableManifestDenial::Io(error.kind()))?;
        let frame = decode_frame(&bytes, 4, format)?;
        if frame.identity != u64::from(ordinal)
            || frame.payload.len() != EXTENT_METADATA_BYTES + length
            || OfflineRecordIdentity::decode(&frame.payload[..24]) != Some(expected.record)
            || read_u64(frame.payload, 24) != expected.extent
            || read_u64(frame.payload, 32) != expected.generation
            || read_u64(frame.payload, 40) != expected.logical_bytes
            || read_u64(frame.payload, 48) != offset
            || read_u32(frame.payload, 56) as usize != length
            || frame.payload[60..64] != [0; 4]
        {
            return Err(OfflineDurableManifestDenial::MalformedPayloadFrame);
        }
        digest.update(&frame.payload[EXTENT_METADATA_BYTES..]);
        offset += length as u64;
        ordinal = ordinal
            .checked_add(1)
            .ok_or(OfflineDurableManifestDenial::MalformedPayloadFrame)?;
    }
    Ok(u64::from(ordinal - 1))
}

#[cfg(test)]
#[path = "payload_validation_tests.rs"]
mod tests;
