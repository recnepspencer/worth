use sha2::{Digest, Sha256};

const CHECKPOINT_HEADER_BYTES: usize = 144;
const CHECKPOINT_FOOTER_BYTES: usize = 136;
const CHECKPOINT_DIRTY_BYTES: usize = 48;
const CHECKPOINT_COMPACTION_BYTES: usize = 16;
const CHECKPOINT_PREFIX_BYTES: usize = 16;
const WAL_HEADER_BYTES: usize = 116;
const WAL_FOOTER_BYTES: usize = 32;

struct Frame {
    offset: usize,
    end_offset: usize,
    start: u64,
    end: u64,
}

struct Segment<'a> {
    path: &'a str,
    bytes: &'a [u8],
    segment: u64,
    generation: u64,
    frames: Vec<Frame>,
}

/// Only a contiguous authenticated WAL suffix beginning at the selected
/// checkpoint frontier can prove the checkpoint fixture's in-flight fate.
/// Checkpoint schema 2 retains the schema-1 record layout while carrying
/// maintenance-capable physical truth. Every record must use the selected
/// header's schema before its WAL frontier is admitted.
pub(super) fn tail<'a>(files: &'a [(String, Vec<u8>)]) -> Option<Vec<(&'a str, &'a [u8])>> {
    let frontier = checkpoint_frontier(files)?;
    let mut segments = files
        .iter()
        .filter_map(|(path, bytes)| {
            let (segment, generation) = wal_path(path)?;
            let frames = scan_segment(bytes, segment, generation);
            frames
                .last()
                .is_some_and(|frame| frame.end > frontier)
                .then_some(Segment {
                    path,
                    bytes,
                    segment,
                    generation,
                    frames,
                })
        })
        .collect::<Vec<_>>();
    segments.sort_unstable_by_key(|segment| segment.segment);
    let first = segments.first()?;
    let first_frame = first.frames.iter().find(|frame| frame.start == frontier)?;
    let mut selected = vec![(
        first.path,
        &first.bytes[first_frame.offset..first.frames.last()?.end_offset],
    )];
    let mut previous_segment = first.segment;
    let mut previous_end = first.frames.last()?.end;
    let generation = first.generation;
    for segment in segments.into_iter().skip(1) {
        if segment.generation != generation
            || segment.segment != previous_segment.checked_add(1)?
            || segment.frames.first()?.start != previous_end
        {
            return None;
        }
        let last = segment.frames.last()?;
        selected.push((segment.path, &segment.bytes[..last.end_offset]));
        previous_segment = segment.segment;
        previous_end = last.end;
    }
    Some(selected)
}

fn checkpoint_frontier(files: &[(String, Vec<u8>)]) -> Option<u64> {
    let bytes = &files
        .iter()
        .find(|(path, _)| path == "families/checkpoint.current")?
        .1;
    let schema = *bytes.get(8)?;
    if schema != 1 && schema != 2 {
        return None;
    }
    let footer_offset = bytes
        .len()
        .checked_sub(record_bytes(CHECKPOINT_FOOTER_BYTES))?;
    let header = fixed_record(bytes, 0, schema, 1, CHECKPOINT_HEADER_BYTES)?;
    let footer = fixed_record(bytes, footer_offset, schema, 5, CHECKPOINT_FOOTER_BYTES)?;
    let covered_start = read_u64(header, 24)?;
    let covered_end = read_u64(header, 32)?;
    let durable = read_u64(footer, 80)?;
    if header[..16] == [0; 16]
        || read_u64(header, 16)? == 0
        || header[64] != 1
        || covered_start >= covered_end
        || durable < covered_start
        || durable > covered_end
        || header[..24] != footer[..24]
    {
        return None;
    }
    let dirty_count = usize::try_from(read_u64(footer, 24)?).ok()?;
    let binding_count = usize::try_from(read_u64(footer, 88)?).ok()?;
    let mut offset = record_bytes(CHECKPOINT_HEADER_BYTES);
    let mut dirty = Sha256::new();
    for _ in 0..dirty_count {
        let record =
            bytes.get(offset..offset.checked_add(record_bytes(CHECKPOINT_DIRTY_BYTES))?)?;
        fixed_record(bytes, offset, schema, 2, CHECKPOINT_DIRTY_BYTES)?;
        dirty.update(record);
        offset += record.len();
    }
    if read_u64(footer, 64)? != offset as u64 || dirty.finalize()[..] != footer[32..64] {
        return None;
    }
    let compaction = fixed_record(bytes, offset, schema, 3, CHECKPOINT_COMPACTION_BYTES)?;
    if read_u64(compaction, 0)? != read_u64(footer, 72)? || read_u64(compaction, 8)? != durable {
        return None;
    }
    offset += record_bytes(CHECKPOINT_COMPACTION_BYTES);
    let mut bindings = Sha256::new();
    let binding_start = offset;
    for _ in 0..binding_count {
        let prefix = bytes.get(offset..offset.checked_add(CHECKPOINT_PREFIX_BYTES)?)?;
        let payload_bytes = usize::try_from(read_u32(prefix, 12)?).ok()?;
        if payload_bytes == 0 || payload_bytes > 4096 {
            return None;
        }
        let record = bytes.get(offset..offset.checked_add(record_bytes(payload_bytes))?)?;
        variable_binding_record(record, schema, payload_bytes)?;
        bindings.update(record);
        offset += record.len();
    }
    (offset == footer_offset
        && read_u64(footer, 96)? == (offset - binding_start) as u64
        && bindings.finalize()[..] == footer[104..136])
        .then_some(covered_end)
}

fn fixed_record<'a>(
    bytes: &'a [u8],
    offset: usize,
    schema: u8,
    kind: u8,
    payload: usize,
) -> Option<&'a [u8]> {
    let record = bytes.get(offset..offset.checked_add(record_bytes(payload))?)?;
    valid_record(record, schema, kind, payload)?;
    record.get(CHECKPOINT_PREFIX_BYTES..CHECKPOINT_PREFIX_BYTES + payload)
}

fn variable_binding_record(record: &[u8], schema: u8, payload: usize) -> Option<()> {
    valid_record(record, schema, 4, payload)
}

fn valid_record(record: &[u8], schema: u8, kind: u8, payload: usize) -> Option<()> {
    (record.get(..8) == Some(b"WCP7REC\0")
        && record.get(8) == Some(&schema)
        && record.get(9) == Some(&kind)
        && record.get(10..12) == Some(&[0; 2])
        && read_u32(record, 12)? == payload as u32
        && read_u32(record, CHECKPOINT_PREFIX_BYTES + payload)?
            == crc32c(record.get(..CHECKPOINT_PREFIX_BYTES + payload)?))
    .then_some(())
}

fn record_bytes(payload: usize) -> usize {
    CHECKPOINT_PREFIX_BYTES + payload + 4
}

fn scan_segment(bytes: &[u8], segment: u64, generation: u64) -> Vec<Frame> {
    let mut frames = Vec::new();
    let mut offset = 0;
    while let Some(frame) = wal_frame(bytes, offset, segment, generation) {
        if frames
            .last()
            .is_some_and(|prior: &Frame| prior.end != frame.start)
        {
            break;
        }
        offset = frame.end_offset;
        frames.push(frame);
    }
    frames
}

fn wal_frame(bytes: &[u8], offset: usize, segment: u64, generation: u64) -> Option<Frame> {
    let header = bytes.get(offset..offset.checked_add(WAL_HEADER_BYTES)?)?;
    let start = read_u64(header, 28)?;
    let end = read_u64(header, 36)?;
    let payload = usize::try_from(read_u64(header, 44)?).ok()?;
    let frame_end = offset.checked_add(
        WAL_HEADER_BYTES
            .checked_add(payload)?
            .checked_add(WAL_FOOTER_BYTES)?,
    )?;
    let frame = bytes.get(offset..frame_end)?;
    let body = frame.get(WAL_HEADER_BYTES..WAL_HEADER_BYTES + payload)?;
    (header.get(..8) == Some(b"WORTHWAL")
        && read_u16(header, 8) == Some(1)
        && read_u16(header, 10) == Some(WAL_HEADER_BYTES as u16)
        && read_u64(header, 12) == Some(segment)
        && read_u64(header, 20) == Some(generation)
        && payload > 0
        && start < end
        && Sha256::digest(body)[..] == header[84..116]
        && Sha256::digest(&frame[..WAL_HEADER_BYTES + payload])[..]
            == frame[WAL_HEADER_BYTES + payload..])
        .then_some(Frame {
            offset,
            end_offset: frame_end,
            start,
            end,
        })
}

fn wal_path(path: &str) -> Option<(u64, u64)> {
    let name = path
        .strip_prefix("families/wal/segment-")?
        .strip_suffix(".wal")?;
    let (segment, generation) = name.split_once("-generation-")?;
    let segment = segment.parse::<u64>().ok()?;
    let generation = generation.parse::<u64>().ok()?;
    (segment > 0
        && generation > 0
        && path == format!("families/wal/segment-{segment}-generation-{generation}.wal"))
    .then_some((segment, generation))
}

fn read_u16(bytes: &[u8], offset: usize) -> Option<u16> {
    Some(u16::from_le_bytes(
        bytes.get(offset..offset.checked_add(2)?)?.try_into().ok()?,
    ))
}

fn read_u32(bytes: &[u8], offset: usize) -> Option<u32> {
    Some(u32::from_le_bytes(
        bytes.get(offset..offset.checked_add(4)?)?.try_into().ok()?,
    ))
}

fn read_u64(bytes: &[u8], offset: usize) -> Option<u64> {
    Some(u64::from_le_bytes(
        bytes.get(offset..offset.checked_add(8)?)?.try_into().ok()?,
    ))
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
