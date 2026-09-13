use std::ops::Range;

pub(crate) const FRAME_HEADER_BYTES: usize = 48;
pub(crate) const FRAME_LENGTH_RANGE: Range<usize> = 24..28;
pub(crate) const FRAME_CHECKSUM_RANGE: Range<usize> = 44..48;
pub(crate) const FRAME_FORMAT_VERSION_RANGE: Range<usize> = 10..12;

pub(crate) fn checksum_is_valid(bytes: &[u8]) -> bool {
    bytes.len() >= FRAME_HEADER_BYTES
        && read_u32(bytes, FRAME_CHECKSUM_RANGE.start) == frame_checksum(bytes)
}

pub(crate) fn refresh_checksum(bytes: &mut [u8]) {
    let checksum = frame_checksum(bytes);
    bytes[FRAME_CHECKSUM_RANGE].copy_from_slice(&checksum.to_le_bytes());
}

pub(crate) fn frame_checksum(bytes: &[u8]) -> u32 {
    let mut crc = !0_u32;
    for byte in bytes
        .get(..FRAME_CHECKSUM_RANGE.start)
        .into_iter()
        .chain(bytes.get(FRAME_HEADER_BYTES..))
        .flatten()
    {
        crc ^= u32::from(*byte);
        for _ in 0..8 {
            let mask = (crc & 1).wrapping_neg();
            crc = (crc >> 1) ^ (0x82f6_3b78 & mask);
        }
    }
    !crc
}

pub(crate) fn encoded_payload_length(bytes: &[u8]) -> Option<u32> {
    bytes
        .get(FRAME_LENGTH_RANGE)
        .and_then(|field| field.try_into().ok())
        .map(u32::from_le_bytes)
}

fn read_u32(bytes: &[u8], offset: usize) -> u32 {
    bytes
        .get(offset..offset + 4)
        .and_then(|field| field.try_into().ok())
        .map(u32::from_le_bytes)
        .unwrap_or_default()
}
