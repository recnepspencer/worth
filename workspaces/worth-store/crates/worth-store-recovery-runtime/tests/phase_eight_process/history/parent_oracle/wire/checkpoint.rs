use sha2::{Digest, Sha256};

use super::super::evidence_digest::{digest_bytes, DigestBuilder};
use super::super::{read_u32, read_u64, CheckpointFacts};

const CHECKPOINT_PREFIX_BYTES: usize = 16;
const CHECKPOINT_HEADER_PAYLOAD_BYTES: usize = 144;
const CHECKPOINT_DIRTY_PAYLOAD_BYTES: usize = 48;
const CHECKPOINT_COMPACTION_PAYLOAD_BYTES: usize = 16;
const CHECKPOINT_FOOTER_PAYLOAD_BYTES: usize = 136;
const CHECKPOINT_BINDING_MAX_PAYLOAD: usize = 4 * 1024;

pub(super) fn observe_checkpoint(bytes: &[u8]) -> Option<CheckpointFacts> {
    let footer_offset = bytes
        .len()
        .checked_sub(record_bytes(CHECKPOINT_FOOTER_PAYLOAD_BYTES))?;
    let header = fixed_record(bytes, 0, 1, CHECKPOINT_HEADER_PAYLOAD_BYTES)?;
    let footer = fixed_record(bytes, footer_offset, 5, CHECKPOINT_FOOTER_PAYLOAD_BYTES)?;
    if header[..16] == [0; 16] || read_u64(header, 16)? == 0 || header[64] != 1 {
        return None;
    }
    let covered = (read_u64(header, 24)?, read_u64(header, 32)?);
    if covered.0 >= covered.1 {
        return None;
    }
    let dirty_count = usize::try_from(read_u64(footer, 24)?).ok()?;
    let binding_count = usize::try_from(read_u64(footer, 88)?).ok()?;
    let dirty_end = record_bytes(CHECKPOINT_HEADER_PAYLOAD_BYTES)
        .checked_add(dirty_count.checked_mul(record_bytes(CHECKPOINT_DIRTY_PAYLOAD_BYTES))?)?;
    if read_u64(footer, 64)? != dirty_end as u64 {
        return None;
    }
    let mut dirty_digest = Sha256::new();
    let mut offset = record_bytes(CHECKPOINT_HEADER_PAYLOAD_BYTES);
    for _ in 0..dirty_count {
        let end = offset.checked_add(record_bytes(CHECKPOINT_DIRTY_PAYLOAD_BYTES))?;
        let record = bytes.get(offset..end)?;
        fixed_record(bytes, offset, 2, CHECKPOINT_DIRTY_PAYLOAD_BYTES)?;
        dirty_digest.update(record);
        offset = end;
    }
    let compaction = fixed_record(bytes, offset, 3, CHECKPOINT_COMPACTION_PAYLOAD_BYTES)?;
    if read_u64(compaction, 0)? != read_u64(footer, 72)?
        || read_u64(compaction, 8)? != read_u64(footer, 80)?
    {
        return None;
    }
    offset += record_bytes(CHECKPOINT_COMPACTION_PAYLOAD_BYTES);
    let mut binding_digest = Sha256::new();
    let mut binding_bytes = 0_u64;
    for _ in 0..binding_count {
        let (record_bytes, payload) = binding_record(bytes, offset)?;
        let record = bytes.get(offset..offset + record_bytes)?;
        binding_digest.update(record);
        binding_bytes = binding_bytes.checked_add(record_bytes as u64)?;
        let _ = payload;
        offset += record_bytes;
    }
    if offset != footer_offset
        || binding_bytes != read_u64(footer, 96)?
        || dirty_digest.finalize()[..] != footer[32..64]
        || binding_digest.finalize()[..] != footer[104..136]
        || footer[..24] != header[..24]
    {
        return None;
    }
    let durable = read_u64(footer, 80)?;
    let mut generation = DigestBuilder::new(b"worth.store.recovery-observer.generation-link.v1");
    generation.record(&header[..16]);
    generation.record(&header[16..24]);
    generation.record(&header[40..48]);
    generation.record(&header[48..56]);
    generation.record(&header[56..64]);
    let mut coverage = Vec::with_capacity(40);
    coverage.extend_from_slice(&covered.0.to_le_bytes());
    coverage.extend_from_slice(&covered.1.to_le_bytes());
    coverage.extend_from_slice(&durable.to_le_bytes());
    coverage.extend_from_slice(&(dirty_count as u64).to_le_bytes());
    coverage.extend_from_slice(&(binding_count as u64).to_le_bytes());
    let mut checkpoint =
        DigestBuilder::new(b"worth.store.recovery-observer.checkpoint-coverage.v1");
    checkpoint.record(&coverage);
    checkpoint.record(&digest_bytes(bytes));
    (durable >= covered.0 && durable <= covered.1).then_some(CheckpointFacts {
        sequence: read_u64(header, 16)?,
        page_count: dirty_count as u64,
        covered,
        redo: covered.0,
        durable,
        generation_links: generation.finish(),
        digest: checkpoint.finish().digest(),
    })
}

fn fixed_record<'bytes>(
    bytes: &'bytes [u8],
    offset: usize,
    kind: u8,
    payload_bytes: usize,
) -> Option<&'bytes [u8]> {
    let end = offset.checked_add(record_bytes(payload_bytes))?;
    let record = bytes.get(offset..end)?;
    if record.get(..8) != Some(b"WCP7REC\0")
        || record.get(8) != Some(&1)
        || record.get(9) != Some(&kind)
        || record.get(10..12) != Some(&[0; 2])
        || read_u32(record, 12)? as usize != payload_bytes
        || crc32c(&record[..CHECKPOINT_PREFIX_BYTES + payload_bytes])
            != read_u32(record, CHECKPOINT_PREFIX_BYTES + payload_bytes)?
    {
        return None;
    }
    Some(&record[CHECKPOINT_PREFIX_BYTES..CHECKPOINT_PREFIX_BYTES + payload_bytes])
}

fn binding_record<'bytes>(bytes: &'bytes [u8], offset: usize) -> Option<(usize, &'bytes [u8])> {
    let prefix = bytes.get(offset..offset + CHECKPOINT_PREFIX_BYTES)?;
    if prefix.get(..8) != Some(b"WCP7REC\0")
        || prefix.get(8) != Some(&1)
        || prefix.get(9) != Some(&4)
        || prefix.get(10..12) != Some(&[0; 2])
    {
        return None;
    }
    let payload_bytes = usize::try_from(read_u32(prefix, 12)?).ok()?;
    if payload_bytes == 0 || payload_bytes > CHECKPOINT_BINDING_MAX_PAYLOAD {
        return None;
    }
    let total = record_bytes(payload_bytes);
    let record = bytes.get(offset..offset + total)?;
    if crc32c(&record[..CHECKPOINT_PREFIX_BYTES + payload_bytes])
        != read_u32(record, CHECKPOINT_PREFIX_BYTES + payload_bytes)?
    {
        return None;
    }
    Some((
        total,
        &record[CHECKPOINT_PREFIX_BYTES..CHECKPOINT_PREFIX_BYTES + payload_bytes],
    ))
}

const fn record_bytes(payload_bytes: usize) -> usize {
    CHECKPOINT_PREFIX_BYTES + payload_bytes + 4
}

fn crc32c(bytes: &[u8]) -> u32 {
    let mut value = !0_u32;
    for byte in bytes {
        value ^= u32::from(*byte);
        for _ in 0..8 {
            let mask = 0_u32.wrapping_sub(value & 1);
            value = (value >> 1) ^ (0x82f6_3b78 & mask);
        }
    }
    !value
}
