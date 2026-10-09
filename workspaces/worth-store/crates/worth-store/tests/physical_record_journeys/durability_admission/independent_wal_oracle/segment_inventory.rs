use std::path::Path;

use sha2::{Digest, Sha256};

const MAGIC: &[u8; 8] = b"WORTHWAL";
const VERSION: u16 = 1;
const HEADER_BYTES: usize = 116;
const FOOTER_BYTES: usize = 32;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(in super::super) struct IndependentWalInventory {
    pub(super) segments: Vec<(u64, u64)>,
    segment_facts: Vec<IndependentWalSegment>,
    pub(super) frame_count: u64,
    pub(super) byte_count: u64,
    pub(super) peak_segment_bytes: u64,
    lsn_range: Option<(u64, u64)>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in super::super) struct IndependentWalSegment {
    identity: (u64, u64),
    lsn_range: (u64, u64),
    pub(super) byte_count: u64,
}

impl IndependentWalInventory {
    pub(in super::super) fn segment_facts(&self) -> &[IndependentWalSegment] {
        &self.segment_facts
    }

    pub(in super::super) const fn lsn_range(&self) -> Option<(u64, u64)> {
        self.lsn_range
    }
}

impl IndependentWalSegment {
    pub(in super::super) const fn identity(self) -> (u64, u64) {
        self.identity
    }

    pub(in super::super) const fn lsn_range(self) -> (u64, u64) {
        self.lsn_range
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in super::super) enum IndependentWalInventoryDenial {
    Io,
    NonCanonicalArtifact,
    NonContiguousSegment,
    GenerationMismatch,
    EmptySegment,
    InvalidFrame,
    SegmentIdentityMismatch,
    NonContiguousLsn,
    DigestMismatch,
    CounterOverflow,
}

pub(in super::super) fn inspect_wal_inventory(
    store_root: &Path,
) -> Result<IndependentWalInventory, IndependentWalInventoryDenial> {
    let directory = store_root.join("families").join("wal");
    let mut artifacts = std::fs::read_dir(directory)
        .map_err(|_| IndependentWalInventoryDenial::Io)?
        .map(|entry| {
            let entry = entry.map_err(|_| IndependentWalInventoryDenial::Io)?;
            let name = entry
                .file_name()
                .into_string()
                .map_err(|_| IndependentWalInventoryDenial::NonCanonicalArtifact)?;
            let identity = parse_artifact_name(&name)
                .ok_or(IndependentWalInventoryDenial::NonCanonicalArtifact)?;
            Ok((identity, entry.path()))
        })
        .collect::<Result<Vec<_>, _>>()?;
    artifacts.sort_unstable_by_key(|((segment, generation), _)| (*segment, *generation));

    let mut identities = Vec::with_capacity(artifacts.len());
    let mut segment_facts = Vec::with_capacity(artifacts.len());
    let mut frame_count = 0u64;
    let mut byte_count = 0u64;
    let mut peak_segment_bytes = 0u64;
    let mut first_lsn = None;
    let mut last_lsn_end = None;
    let mut prior_segment = None;
    let mut inventory_generation = None;
    for ((segment, generation), path) in artifacts {
        if prior_segment.is_some_and(|prior| prior + 1 != segment) {
            return Err(IndependentWalInventoryDenial::NonContiguousSegment);
        }
        if inventory_generation.is_some_and(|prior| prior != generation) {
            return Err(IndependentWalInventoryDenial::GenerationMismatch);
        }
        let bytes = std::fs::read(path).map_err(|_| IndependentWalInventoryDenial::Io)?;
        if bytes.is_empty() {
            return Err(IndependentWalInventoryDenial::EmptySegment);
        }
        let observed = inspect_segment(&bytes, segment, generation, last_lsn_end)?;
        first_lsn.get_or_insert(observed.first_lsn);
        last_lsn_end = Some(observed.last_lsn_end);
        frame_count = frame_count
            .checked_add(observed.frame_count)
            .ok_or(IndependentWalInventoryDenial::CounterOverflow)?;
        byte_count = byte_count
            .checked_add(bytes.len() as u64)
            .ok_or(IndependentWalInventoryDenial::CounterOverflow)?;
        peak_segment_bytes = peak_segment_bytes.max(bytes.len() as u64);
        prior_segment = Some(segment);
        inventory_generation = Some(generation);
        identities.push((segment, generation));
        segment_facts.push(IndependentWalSegment {
            identity: (segment, generation),
            lsn_range: (observed.first_lsn, observed.last_lsn_end),
            byte_count: bytes.len() as u64,
        });
    }

    Ok(IndependentWalInventory {
        segments: identities,
        segment_facts,
        frame_count,
        byte_count,
        peak_segment_bytes,
        lsn_range: first_lsn.zip(last_lsn_end),
    })
}

struct IndependentSegmentObservation {
    first_lsn: u64,
    last_lsn_end: u64,
    frame_count: u64,
}

fn inspect_segment(
    bytes: &[u8],
    expected_segment: u64,
    expected_generation: u64,
    prior_lsn_end: Option<u64>,
) -> Result<IndependentSegmentObservation, IndependentWalInventoryDenial> {
    let mut offset = 0usize;
    let mut first_lsn = None;
    let mut last_lsn_end = prior_lsn_end;
    let mut frame_count = 0u64;
    while offset < bytes.len() {
        let header_end = offset
            .checked_add(HEADER_BYTES)
            .ok_or(IndependentWalInventoryDenial::InvalidFrame)?;
        let header = bytes
            .get(offset..header_end)
            .ok_or(IndependentWalInventoryDenial::InvalidFrame)?;
        let frame = decode_header(header)?;
        if frame.segment != expected_segment || frame.generation != expected_generation {
            return Err(IndependentWalInventoryDenial::SegmentIdentityMismatch);
        }
        if last_lsn_end.is_some_and(|prior| prior != frame.lsn_start) {
            return Err(IndependentWalInventoryDenial::NonContiguousLsn);
        }
        let payload_bytes = usize::try_from(frame.payload_bytes)
            .map_err(|_| IndependentWalInventoryDenial::InvalidFrame)?;
        let payload_end = header_end
            .checked_add(payload_bytes)
            .ok_or(IndependentWalInventoryDenial::InvalidFrame)?;
        let frame_end = payload_end
            .checked_add(FOOTER_BYTES)
            .ok_or(IndependentWalInventoryDenial::InvalidFrame)?;
        let payload = bytes
            .get(header_end..payload_end)
            .ok_or(IndependentWalInventoryDenial::InvalidFrame)?;
        let footer = bytes
            .get(payload_end..frame_end)
            .ok_or(IndependentWalInventoryDenial::InvalidFrame)?;
        if Sha256::digest(payload)[..] != header[84..116]
            || Sha256::digest(&bytes[offset..payload_end])[..] != *footer
        {
            return Err(IndependentWalInventoryDenial::DigestMismatch);
        }
        first_lsn.get_or_insert(frame.lsn_start);
        last_lsn_end = Some(frame.lsn_end);
        frame_count = frame_count
            .checked_add(1)
            .ok_or(IndependentWalInventoryDenial::CounterOverflow)?;
        offset = frame_end;
    }
    Ok(IndependentSegmentObservation {
        first_lsn: first_lsn.ok_or(IndependentWalInventoryDenial::InvalidFrame)?,
        last_lsn_end: last_lsn_end.ok_or(IndependentWalInventoryDenial::InvalidFrame)?,
        frame_count,
    })
}

struct IndependentFrameHeader {
    segment: u64,
    generation: u64,
    lsn_start: u64,
    lsn_end: u64,
    payload_bytes: u64,
}

fn decode_header(header: &[u8]) -> Result<IndependentFrameHeader, IndependentWalInventoryDenial> {
    if header.get(..8) != Some(MAGIC.as_slice())
        || read_u16(header, 8)? != VERSION
        || usize::from(read_u16(header, 10)?) != HEADER_BYTES
    {
        return Err(IndependentWalInventoryDenial::InvalidFrame);
    }
    let frame = IndependentFrameHeader {
        segment: read_u64(header, 12)?,
        generation: read_u64(header, 20)?,
        lsn_start: read_u64(header, 28)?,
        lsn_end: read_u64(header, 36)?,
        payload_bytes: read_u64(header, 44)?,
    };
    if frame.segment == 0
        || frame.generation == 0
        || frame.lsn_start >= frame.lsn_end
        || frame.payload_bytes == 0
    {
        return Err(IndependentWalInventoryDenial::InvalidFrame);
    }
    Ok(frame)
}

fn parse_artifact_name(name: &str) -> Option<(u64, u64)> {
    let body = name.strip_prefix("segment-")?.strip_suffix(".wal")?;
    let (segment, generation) = body.split_once("-generation-")?;
    let segment = segment.parse::<u64>().ok().filter(|value| *value > 0)?;
    let generation = generation.parse::<u64>().ok().filter(|value| *value > 0)?;
    (format!("segment-{segment}-generation-{generation}.wal") == name)
        .then_some((segment, generation))
}

fn read_u16(bytes: &[u8], offset: usize) -> Result<u16, IndependentWalInventoryDenial> {
    let raw = bytes
        .get(offset..offset + 2)
        .ok_or(IndependentWalInventoryDenial::InvalidFrame)?;
    Ok(u16::from_le_bytes(raw.try_into().expect("fixed u16")))
}

fn read_u64(bytes: &[u8], offset: usize) -> Result<u64, IndependentWalInventoryDenial> {
    let raw = bytes
        .get(offset..offset + 8)
        .ok_or(IndependentWalInventoryDenial::InvalidFrame)?;
    Ok(u64::from_le_bytes(raw.try_into().expect("fixed u64")))
}
