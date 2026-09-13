use sha2::{Digest, Sha256};

use super::super::WalFacts;

const HEADER_BYTES: usize = 116;
const FOOTER_BYTES: usize = 32;

pub(super) struct WalTailSuffix {
    pub(super) bytes: Vec<u8>,
    pub(super) first_lsn: u64,
    pub(super) last_lsn: u64,
}

pub(super) fn select(
    bytes: &[u8],
    facts: WalFacts,
    frontier: u64,
) -> Result<Option<WalTailSuffix>, String> {
    let valid = usize::try_from(facts.valid_bytes)
        .map_err(|_| "selected-basis WAL prefix is too large".to_owned())?;
    if valid > bytes.len() {
        return Err("selected-basis WAL prefix exceeds artifact".to_owned());
    }
    let mut offset = 0;
    let mut selected_offset = None;
    let mut first_lsn = None;
    let mut last_lsn = None;
    while offset < valid {
        let frame = read_frame(&bytes[offset..valid])?;
        first_lsn.get_or_insert(frame.start_lsn);
        last_lsn = Some(frame.end_lsn);
        if frame.start_lsn == frontier {
            selected_offset = Some(offset);
        } else if frame.start_lsn > frontier && selected_offset.is_none() {
            return Err("selected-basis WAL frontier is not a complete frame start".to_owned());
        }
        offset = offset
            .checked_add(frame.length)
            .ok_or_else(|| "selected-basis WAL frame offset overflowed".to_owned())?;
    }
    if offset != valid {
        return Err("selected-basis WAL valid prefix ended inside a frame".to_owned());
    }
    let first_lsn =
        first_lsn.ok_or_else(|| "selected-basis WAL omitted its first LSN".to_owned())?;
    let last_lsn = last_lsn.ok_or_else(|| "selected-basis WAL omitted its last LSN".to_owned())?;
    if last_lsn <= frontier {
        return Ok(None);
    }
    let start = selected_offset.ok_or_else(|| {
        "selected-basis WAL frontier is absent from the complete frame sequence".to_owned()
    })?;
    Ok(Some(WalTailSuffix {
        bytes: bytes[start..valid].to_vec(),
        first_lsn: if start == 0 { first_lsn } else { frontier },
        last_lsn,
    }))
}

struct Frame {
    length: usize,
    start_lsn: u64,
    end_lsn: u64,
}

fn read_frame(bytes: &[u8]) -> Result<Frame, String> {
    let header = bytes
        .get(..HEADER_BYTES)
        .ok_or_else(|| "selected-basis WAL frame header is truncated".to_owned())?;
    if header.get(..8) != Some(b"WORTHWAL")
        || read_u16(header, 8) != Some(1)
        || read_u16(header, 10) != Some(116)
    {
        return Err("selected-basis WAL frame header is invalid".to_owned());
    }
    let payload_bytes = usize::try_from(
        read_u64(header, 44)
            .ok_or_else(|| "selected-basis WAL frame omitted payload length".to_owned())?,
    )
    .map_err(|_| "selected-basis WAL payload length overflowed".to_owned())?;
    let length = HEADER_BYTES
        .checked_add(payload_bytes)
        .and_then(|value| value.checked_add(FOOTER_BYTES))
        .ok_or_else(|| "selected-basis WAL frame length overflowed".to_owned())?;
    let frame = bytes
        .get(..length)
        .ok_or_else(|| "selected-basis WAL frame is truncated".to_owned())?;
    let start_lsn = read_u64(header, 28)
        .ok_or_else(|| "selected-basis WAL frame omitted start LSN".to_owned())?;
    let end_lsn = read_u64(header, 36)
        .ok_or_else(|| "selected-basis WAL frame omitted end LSN".to_owned())?;
    let payload = &frame[HEADER_BYTES..HEADER_BYTES + payload_bytes];
    let payload_digest: [u8; 32] = Sha256::digest(payload).into();
    let frame_digest: [u8; 32] = Sha256::digest(&frame[..HEADER_BYTES + payload_bytes]).into();
    if read_u64(header, 12).is_none()
        || read_u64(header, 20).is_none()
        || payload_bytes == 0
        || start_lsn >= end_lsn
        || header[84..116] != payload_digest
        || frame[HEADER_BYTES + payload_bytes..] != frame_digest
    {
        return Err("selected-basis WAL frame integrity is invalid".to_owned());
    }
    Ok(Frame {
        length,
        start_lsn,
        end_lsn,
    })
}

fn read_u16(bytes: &[u8], offset: usize) -> Option<u16> {
    bytes
        .get(offset..offset + 2)?
        .try_into()
        .ok()
        .map(u16::from_le_bytes)
}

fn read_u64(bytes: &[u8], offset: usize) -> Option<u64> {
    bytes
        .get(offset..offset + 8)?
        .try_into()
        .ok()
        .map(u64::from_le_bytes)
}
