use super::super::canonical_membership_frame::{find_file, frame_at, frame_total};
use super::super::{read_u16, read_u32, read_u64};
use super::RecordIdentity;

const INLINE_PAGE_KIND: u8 = 3;

pub(super) fn read_inline(
    files: &[(String, Vec<u8>)],
    record: RecordIdentity,
    segment: u64,
    page: u64,
    segment_generation: u64,
    page_generation: u64,
    slot_generation: u64,
    slot: u16,
    payload_bytes: u64,
) -> Result<(RecordIdentity, Vec<u8>), String> {
    let path =
        format!("families/records/segments/segment-{segment:016x}-{segment_generation:016x}.pages");
    let bytes = find_file(files, &path).ok_or_else(|| format!("parent oracle missing {path}"))?;
    let mut offset = 0;
    while offset < bytes.len() {
        let frame =
            frame_at(bytes, offset).ok_or_else(|| format!("parent oracle malformed {path}"))?;
        if frame.kind == INLINE_PAGE_KIND
            && frame.identity == page_generation
            && frame.payload.len() >= 24
            && read_u64(frame.payload, 0) == Some(segment)
            && read_u64(frame.payload, 8) == Some(page)
        {
            let count =
                usize::from(read_u16(frame.payload, 16).ok_or("inline slot count missing")?);
            let index = usize::from(slot.checked_sub(1).ok_or("inline slot is zero")?);
            let base = 24usize
                .checked_add(index.checked_mul(40).ok_or("inline slot offset overflow")?)
                .ok_or("inline slot offset overflow")?;
            let entry = frame
                .payload
                .get(base..base + 40)
                .ok_or("inline slot is missing")?;
            if index >= count
                || entry[..16] != record.allocation_epoch
                || read_u64(entry, 16) != Some(record.ordinal)
                || read_u64(entry, 32) != Some(slot_generation)
            {
                return Err("parent oracle inline root membership disagrees with page".to_owned());
            }
            let start = usize::try_from(read_u32(entry, 24).ok_or("inline offset missing")?)
                .map_err(|_| "inline offset does not fit usize")?;
            let length = usize::try_from(read_u32(entry, 28).ok_or("inline length missing")?)
                .map_err(|_| "inline length does not fit usize")?;
            if u64::try_from(length).ok() != Some(payload_bytes) {
                return Err("parent oracle inline payload length disagrees with root".to_owned());
            }
            return Ok((
                record,
                frame
                    .payload
                    .get(start..start + length)
                    .ok_or("inline payload range is missing")?
                    .to_vec(),
            ));
        }
        offset += frame_total(bytes, offset)?;
    }
    Err("parent oracle could not find the selected inline page".to_owned())
}
