use super::{read_u16, read_u32, read_u64};

pub(super) const FRAME_BYTES: usize = 48;

#[derive(Debug, Clone, Copy)]
pub(super) struct Frame<'a> {
    pub(super) kind: u8,
    pub(super) identity: u64,
    pub(super) payload: &'a [u8],
}

pub(super) fn find_file<'a>(files: &'a [(String, Vec<u8>)], suffix: &str) -> Option<&'a [u8]> {
    files
        .iter()
        .find(|(path, _)| path == suffix || path.ends_with(&format!("/{suffix}")))
        .map(|(_, bytes)| bytes.as_slice())
}

pub(super) fn frame_at(bytes: &[u8], offset: usize) -> Option<Frame<'_>> {
    let total = frame_total(bytes, offset).ok()?;
    decode_frame(bytes.get(offset..offset + total)?)
}

pub(super) fn frame_total(bytes: &[u8], offset: usize) -> Result<usize, String> {
    let header = bytes
        .get(offset..offset + FRAME_BYTES)
        .ok_or("parent oracle frame header is truncated")?;
    if header[..8] != *b"WRC5FRM\0" {
        return Err("parent oracle frame magic is invalid".to_owned());
    }
    let payload =
        usize::try_from(read_u32(header, 24).ok_or("parent oracle frame length missing")?)
            .map_err(|_| "parent oracle frame length is too large")?;
    FRAME_BYTES
        .checked_add(payload)
        .ok_or_else(|| "parent oracle frame length overflow".to_owned())
}

pub(super) fn decode_frame(bytes: &[u8]) -> Option<Frame<'_>> {
    if bytes.len() < FRAME_BYTES
        || bytes[..8] != *b"WRC5FRM\0"
        || !matches!(bytes[8], 1..=11)
        || bytes[9] != 2
        || read_u16(bytes, 20)? as usize != FRAME_BYTES
        || bytes[22..24] != [0; 2]
        || bytes.len() != FRAME_BYTES + read_u32(bytes, 24)? as usize
    {
        return None;
    }
    let mut covered = Vec::with_capacity(bytes.len() - 4);
    covered.extend_from_slice(&bytes[..44]);
    covered.extend_from_slice(&bytes[FRAME_BYTES..]);
    (crc32c(&covered) == read_u32(bytes, 44)?).then_some(Frame {
        kind: bytes[8],
        identity: read_u64(bytes, 28)?,
        payload: &bytes[FRAME_BYTES..],
    })
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
