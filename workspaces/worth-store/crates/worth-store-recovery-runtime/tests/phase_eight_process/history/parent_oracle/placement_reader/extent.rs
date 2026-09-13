use super::super::canonical_membership_frame::{find_file, frame_at, frame_total};
use super::super::{read_u32, read_u64};
use super::RecordIdentity;

const EXTENT_KIND: u8 = 4;

pub(super) fn read_extent(
    files: &[(String, Vec<u8>)],
    record: RecordIdentity,
    extent: u64,
    generation: u64,
    payload_bytes: u64,
) -> Result<(RecordIdentity, Vec<u8>), String> {
    let path = format!("families/records/extents/extent-{extent:016x}-{generation:016x}.data");
    let bytes = find_file(files, &path).ok_or_else(|| format!("parent oracle missing {path}"))?;
    let mut chunks = Vec::new();
    let mut offset = 0;
    while offset < bytes.len() {
        let frame =
            frame_at(bytes, offset).ok_or_else(|| format!("parent oracle malformed {path}"))?;
        if frame.kind == EXTENT_KIND {
            if frame.payload.len() < 64
                || frame.payload[..16] != record.allocation_epoch
                || read_u64(frame.payload, 16) != Some(record.ordinal)
                || read_u64(frame.payload, 24) != Some(extent)
                || read_u64(frame.payload, 32) != Some(generation)
                || read_u64(frame.payload, 40) != Some(payload_bytes)
            {
                return Err("parent oracle extent root membership disagrees with chunk".to_owned());
            }
            let logical_offset = read_u64(frame.payload, 48).ok_or("extent offset missing")?;
            let chunk_bytes =
                usize::try_from(read_u32(frame.payload, 56).ok_or("extent length missing")?)
                    .map_err(|_| "extent chunk length does not fit usize")?;
            let chunk = frame
                .payload
                .get(64..64 + chunk_bytes)
                .ok_or("extent chunk payload is missing")?;
            chunks.push((logical_offset, chunk.to_vec()));
        }
        offset += frame_total(bytes, offset)?;
    }
    chunks.sort_by_key(|(offset, _)| *offset);
    let mut result = Vec::with_capacity(
        usize::try_from(payload_bytes).map_err(|_| "extent length is too large")?,
    );
    let mut next = 0_u64;
    for (logical_offset, chunk) in chunks {
        if logical_offset != next {
            return Err("parent oracle extent chunks are not contiguous".to_owned());
        }
        next = next
            .checked_add(u64::try_from(chunk.len()).map_err(|_| "extent chunk is too large")?)
            .ok_or("extent length overflow")?;
        result.extend_from_slice(&chunk);
    }
    (next == payload_bytes)
        .then_some((record, result))
        .ok_or_else(|| "parent oracle extent coverage disagrees with root".to_owned())
}
