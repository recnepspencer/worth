use super::super::canonical_membership_frame::{find_file, frame_at, frame_total};
use super::super::{read_u32, read_u64};
use super::RecordIdentity;

const MANIFEST_KIND: u8 = 6;
const CHUNK_KIND: u8 = 4;
const MANIFEST_FRAME_BYTES: usize = 104;
const CHUNK_METADATA_BYTES: usize = 64;

#[allow(clippy::too_many_arguments)]
pub(super) fn read_extent(
    files: &[(String, Vec<u8>)],
    record: RecordIdentity,
    arena: u64,
    extent: u64,
    generation: u64,
    arena_offset: u64,
    arena_length: u64,
    payload_bytes: u64,
    expected_format: [u8; 10],
) -> Result<(RecordIdentity, Vec<u8>), String> {
    if arena == 0 || extent == 0 || generation == 0 || arena_length == 0 {
        return Err("parent oracle root has an invalid arena extent route".to_owned());
    }
    let path = format!("families/records/arenas/arena-{arena:016x}.data");
    let bytes = find_file(files, &path).ok_or_else(|| format!("parent oracle missing {path}"))?;
    let start = usize::try_from(arena_offset).map_err(|_| "arena offset is too large")?;
    let end = arena_offset
        .checked_add(arena_length)
        .and_then(|value| usize::try_from(value).ok())
        .ok_or("arena range overflow")?;
    let manifest = frame_at(bytes, start).ok_or("parent oracle malformed arena manifest")?;
    let manifest_end = start
        .checked_add(frame_total(bytes, start)?)
        .ok_or("arena manifest end overflow")?;
    if manifest.kind != MANIFEST_KIND
        || manifest.format != expected_format
        || manifest.identity != generation
        || manifest.payload.len() != 56
        || manifest.payload[..16] != record.allocation_epoch
        || read_u64(manifest.payload, 16) != Some(record.ordinal)
        || read_u64(manifest.payload, 24) != Some(extent)
        || read_u64(manifest.payload, 32) != Some(payload_bytes)
        || manifest_end > end
        || manifest_end - start != MANIFEST_FRAME_BYTES
    {
        return Err("parent oracle root membership disagrees with arena manifest".to_owned());
    }
    let maximum_frame =
        u64::from(read_u32(manifest.payload, 40).ok_or("manifest frame limit missing")?);
    let count = u64::from(read_u32(manifest.payload, 44).ok_or("manifest chunk count missing")?);
    let alignment = read_u64(manifest.payload, 48).ok_or("manifest alignment missing")?;
    let page_bytes = u64::from(read_u32(&manifest.format, 2).ok_or("manifest page size missing")?);
    if !alignment.is_power_of_two()
        || maximum_frame != page_bytes
        || maximum_frame <= (48 + CHUNK_METADATA_BYTES) as u64
        || arena_offset % alignment != 0
    {
        return Err("parent oracle arena manifest geometry is invalid".to_owned());
    }
    let manifest_stride =
        align_up(MANIFEST_FRAME_BYTES as u64, alignment).ok_or("manifest stride overflow")?;
    let chunk_stride = align_up(maximum_frame, alignment).ok_or("chunk stride overflow")?;
    let capacity = maximum_frame - (48 + CHUNK_METADATA_BYTES) as u64;
    if payload_bytes == 0
        || count != payload_bytes.div_ceil(capacity)
        || arena_length
            != manifest_stride
                .checked_add(
                    chunk_stride
                        .checked_mul(count)
                        .ok_or("arena range overflow")?,
                )
                .ok_or("arena range overflow")?
    {
        return Err("parent oracle arena range disagrees with manifest geometry".to_owned());
    }
    let mut result = Vec::with_capacity(
        usize::try_from(payload_bytes).map_err(|_| "extent length is too large")?,
    );
    let mut logical_offset = 0_u64;
    for ordinal in 1..=count {
        let slot = arena_offset
            .checked_add(manifest_stride)
            .and_then(|value| value.checked_add(chunk_stride.checked_mul(ordinal - 1)?))
            .and_then(|value| usize::try_from(value).ok())
            .ok_or("arena chunk offset overflow")?;
        let frame = frame_at(bytes, slot).ok_or("parent oracle malformed arena chunk")?;
        let frame_bytes =
            u64::try_from(frame_total(bytes, slot)?).map_err(|_| "arena chunk too large")?;
        let frame_end = slot
            .checked_add(usize::try_from(frame_bytes).map_err(|_| "arena chunk too large")?)
            .ok_or("arena chunk end overflow")?;
        let chunk_bytes = (payload_bytes - logical_offset).min(capacity);
        if frame.kind != CHUNK_KIND
            || frame.format != manifest.format
            || frame.identity != ordinal
            || frame_bytes > chunk_stride
            || frame_end > end
            || frame.payload.len()
                != CHUNK_METADATA_BYTES
                    + usize::try_from(chunk_bytes).map_err(|_| "chunk length overflow")?
            || frame.payload[..16] != record.allocation_epoch
            || read_u64(frame.payload, 16) != Some(record.ordinal)
            || read_u64(frame.payload, 24) != Some(extent)
            || read_u64(frame.payload, 32) != Some(generation)
            || read_u64(frame.payload, 40) != Some(payload_bytes)
            || read_u64(frame.payload, 48) != Some(logical_offset)
            || read_u32(frame.payload, 56) != u32::try_from(chunk_bytes).ok()
            || frame.payload[60..64] != [0; 4]
        {
            return Err("parent oracle root membership disagrees with arena chunk".to_owned());
        }
        result.extend_from_slice(&frame.payload[CHUNK_METADATA_BYTES..]);
        logical_offset += chunk_bytes;
    }
    if logical_offset != payload_bytes {
        return Err("parent oracle extent coverage disagrees with root".to_owned());
    }
    Ok((record, result))
}

fn align_up(bytes: u64, alignment: u64) -> Option<u64> {
    bytes
        .checked_add(alignment - 1)
        .map(|value| value & !(alignment - 1))
}
