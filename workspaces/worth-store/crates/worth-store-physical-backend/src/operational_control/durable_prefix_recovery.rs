use std::io::{self, Read};

use sha2::{Digest, Sha256};
use worth_store_authority::ControlStoreGeneration;

use super::ControlMediaFault;

const MAGIC: &[u8; 8] = b"WCTRL002";
const HEADER_LEN: usize = 8 + 8 + 4 + 8 + 32;
const CHECKSUM_LEN: usize = 32;
pub(crate) const MAX_TRANSITION_IDENTITY_BYTES: usize = 4 * 1024;
pub(crate) const MAX_CONTROL_PAYLOAD_BYTES: usize = 4 * 1024 * 1024;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DurableControlRecordBytes {
    generation: ControlStoreGeneration,
    transition_identity: String,
    payload: Vec<u8>,
    prefix_digest: [u8; 32],
    frame_checksum: [u8; 32],
}

impl DurableControlRecordBytes {
    pub const fn generation(&self) -> ControlStoreGeneration {
        self.generation
    }

    pub fn transition_identity(&self) -> &str {
        &self.transition_identity
    }

    pub fn payload(&self) -> &[u8] {
        &self.payload
    }

    pub const fn prefix_digest(&self) -> [u8; 32] {
        self.prefix_digest
    }

    pub const fn frame_checksum(&self) -> [u8; 32] {
        self.frame_checksum
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct DurablePrefixSummary {
    last_generation: Option<ControlStoreGeneration>,
    record_count: u64,
    end_offset: u64,
    prefix_digest: [u8; 32],
    last_frame_checksum: Option<[u8; 32]>,
}

impl DurablePrefixSummary {
    pub(crate) const fn last_generation(self) -> Option<ControlStoreGeneration> {
        self.last_generation
    }

    pub(crate) const fn record_count(self) -> u64 {
        self.record_count
    }

    pub(crate) const fn end_offset(self) -> u64 {
        self.end_offset
    }

    pub(crate) const fn prefix_digest(self) -> [u8; 32] {
        self.prefix_digest
    }

    pub(crate) const fn last_frame_checksum(self) -> Option<[u8; 32]> {
        self.last_frame_checksum
    }
}

pub(crate) fn validate_record_lengths(
    transition_identity: &str,
    payload: &[u8],
) -> Result<(), ControlMediaFault> {
    if transition_identity.len() > MAX_TRANSITION_IDENTITY_BYTES
        || payload.len() > MAX_CONTROL_PAYLOAD_BYTES
    {
        return Err(ControlMediaFault::RecordTooLarge {
            transition_bytes: transition_identity.len() as u64,
            payload_bytes: payload.len() as u64,
        });
    }
    Ok(())
}

pub(crate) fn encode_record(
    generation: ControlStoreGeneration,
    previous_prefix_digest: [u8; 32],
    transition_identity: &str,
    payload: &[u8],
) -> Result<Vec<u8>, ControlMediaFault> {
    validate_record_lengths(transition_identity, payload)?;
    let transition = transition_identity.as_bytes();
    let frame_bytes = HEADER_LEN
        .checked_add(transition.len())
        .and_then(|bytes| bytes.checked_add(payload.len()))
        .and_then(|bytes| bytes.checked_add(CHECKSUM_LEN))
        .ok_or(ControlMediaFault::RecordTooLarge {
            transition_bytes: transition.len() as u64,
            payload_bytes: payload.len() as u64,
        })?;
    let mut frame = Vec::new();
    frame
        .try_reserve_exact(frame_bytes)
        .map_err(|_| ControlMediaFault::AllocationFailed)?;
    frame.extend_from_slice(MAGIC);
    frame.extend_from_slice(&generation.get().to_le_bytes());
    frame.extend_from_slice(&(transition.len() as u32).to_le_bytes());
    frame.extend_from_slice(&(payload.len() as u64).to_le_bytes());
    frame.extend_from_slice(&previous_prefix_digest);
    frame.extend_from_slice(transition);
    frame.extend_from_slice(payload);
    let checksum = Sha256::digest(&frame);
    frame.extend_from_slice(&checksum);
    Ok(frame)
}

pub(crate) fn scan_durable_prefix(
    reader: &mut impl Read,
    observe: impl FnMut(DurableControlRecordBytes) -> Result<(), ControlMediaFault>,
) -> Result<DurablePrefixSummary, ControlMediaFault> {
    scan_durable_suffix(reader, 0, None, [0; 32], None, observe)
}

pub(crate) fn scan_durable_suffix(
    reader: &mut impl Read,
    initial_offset: u64,
    initial_generation: Option<ControlStoreGeneration>,
    initial_prefix_digest: [u8; 32],
    initial_last_frame_checksum: Option<[u8; 32]>,
    mut observe: impl FnMut(DurableControlRecordBytes) -> Result<(), ControlMediaFault>,
) -> Result<DurablePrefixSummary, ControlMediaFault> {
    scan_durable_records(
        reader,
        initial_offset,
        initial_generation,
        initial_prefix_digest,
        initial_last_frame_checksum,
        &mut observe,
    )
}

/// One compiled control-prefix scan for every reader and observer.
#[inline(never)]
fn scan_durable_records(
    reader: &mut dyn Read,
    initial_offset: u64,
    initial_generation: Option<ControlStoreGeneration>,
    initial_prefix_digest: [u8; 32],
    initial_last_frame_checksum: Option<[u8; 32]>,
    observe: &mut dyn FnMut(DurableControlRecordBytes) -> Result<(), ControlMediaFault>,
) -> Result<DurablePrefixSummary, ControlMediaFault> {
    let mut offset = initial_offset;
    let mut previous = initial_generation;
    let mut prefix_digest = initial_prefix_digest;
    let mut last_frame_checksum = initial_last_frame_checksum;
    let mut record_count = 0u64;
    loop {
        let Some(header) = read_header(reader, offset)? else {
            return Ok(DurablePrefixSummary {
                last_generation: previous,
                record_count,
                end_offset: offset,
                prefix_digest,
                last_frame_checksum,
            });
        };
        let generation = decode_header(&header, offset, previous, prefix_digest)?;
        let transition_len = decode_u32(&header, 16) as usize;
        let payload_len_raw = decode_u64(&header, 20);
        let payload_len =
            usize::try_from(payload_len_raw).map_err(|_| corrupt(offset, generation))?;
        if transition_len > MAX_TRANSITION_IDENTITY_BYTES || payload_len > MAX_CONTROL_PAYLOAD_BYTES
        {
            return Err(ControlMediaFault::RecordTooLarge {
                transition_bytes: transition_len as u64,
                payload_bytes: payload_len_raw,
            });
        }
        let mut transition = zeroed_bytes(transition_len)?;
        let mut payload = zeroed_bytes(payload_len)?;
        let mut checksum = [0; CHECKSUM_LEN];
        read_frame_part(reader, &mut transition, offset)?;
        read_frame_part(reader, &mut payload, offset)?;
        read_frame_part(reader, &mut checksum, offset)?;
        let mut digest = Sha256::new();
        digest.update(header);
        digest.update(&transition);
        digest.update(&payload);
        if digest.finalize().as_slice() != checksum {
            return Err(corrupt(offset, generation));
        }
        prefix_digest = extend_prefix_digest(prefix_digest, checksum);
        last_frame_checksum = Some(checksum);
        let transition_identity =
            String::from_utf8(transition).map_err(|_| corrupt(offset, generation))?;
        observe(DurableControlRecordBytes {
            generation,
            transition_identity,
            payload,
            prefix_digest,
            frame_checksum: checksum,
        })?;
        previous = Some(generation);
        record_count = record_count
            .checked_add(1)
            .ok_or(ControlMediaFault::GenerationExhausted)?;
        offset = offset
            .checked_add((HEADER_LEN + transition_len + payload_len + CHECKSUM_LEN) as u64)
            .ok_or(ControlMediaFault::GenerationExhausted)?;
    }
}

fn zeroed_bytes(bytes: usize) -> Result<Vec<u8>, ControlMediaFault> {
    let mut allocation = Vec::new();
    allocation
        .try_reserve_exact(bytes)
        .map_err(|_| ControlMediaFault::AllocationFailed)?;
    allocation.resize(bytes, 0);
    Ok(allocation)
}

pub(crate) fn extend_prefix_digest(previous: [u8; 32], frame_checksum: [u8; 32]) -> [u8; 32] {
    let mut digest = Sha256::new();
    digest.update(b"worth-store-control-prefix-v1\0");
    digest.update(previous);
    digest.update(frame_checksum);
    digest.finalize().into()
}

fn read_header(
    reader: &mut dyn Read,
    offset: u64,
) -> Result<Option<[u8; HEADER_LEN]>, ControlMediaFault> {
    let mut header = [0; HEADER_LEN];
    let first = reader.read(&mut header)?;
    if first == 0 {
        return Ok(None);
    }
    if let Err(error) = reader.read_exact(&mut header[first..]) {
        return match error.kind() {
            io::ErrorKind::UnexpectedEof => Err(ControlMediaFault::TornTail { offset }),
            _ => Err(error.into()),
        };
    }
    Ok(Some(header))
}

fn read_frame_part(
    reader: &mut dyn Read,
    bytes: &mut [u8],
    offset: u64,
) -> Result<(), ControlMediaFault> {
    reader
        .read_exact(bytes)
        .map_err(|error| match error.kind() {
            io::ErrorKind::UnexpectedEof => ControlMediaFault::TornTail { offset },
            _ => error.into(),
        })
}

fn decode_header(
    header: &[u8; HEADER_LEN],
    offset: u64,
    previous: Option<ControlStoreGeneration>,
    previous_prefix_digest: [u8; 32],
) -> Result<ControlStoreGeneration, ControlMediaFault> {
    if &header[..MAGIC.len()] != MAGIC {
        return Err(ControlMediaFault::CorruptRecord {
            offset,
            generation: None,
        });
    }
    let raw = decode_u64(header, 8);
    let generation =
        ControlStoreGeneration::from_raw(raw).ok_or_else(|| corrupt_without_generation(offset))?;
    let expected = previous.and_then(ControlStoreGeneration::next);
    if previous.is_some() && expected != Some(generation)
        || previous.is_none() && generation != ControlStoreGeneration::initial()
        || header[28..60] != previous_prefix_digest
    {
        return Err(corrupt(offset, generation));
    }
    Ok(generation)
}

const fn corrupt(offset: u64, generation: ControlStoreGeneration) -> ControlMediaFault {
    ControlMediaFault::CorruptRecord {
        offset,
        generation: Some(generation),
    }
}

const fn corrupt_without_generation(offset: u64) -> ControlMediaFault {
    ControlMediaFault::CorruptRecord {
        offset,
        generation: None,
    }
}

fn decode_u32(header: &[u8; HEADER_LEN], start: usize) -> u32 {
    let mut bytes = [0; 4];
    bytes.copy_from_slice(&header[start..start + 4]);
    u32::from_le_bytes(bytes)
}

fn decode_u64(header: &[u8; HEADER_LEN], start: usize) -> u64 {
    let mut bytes = [0; 8];
    bytes.copy_from_slice(&header[start..start + 8]);
    u64::from_le_bytes(bytes)
}
