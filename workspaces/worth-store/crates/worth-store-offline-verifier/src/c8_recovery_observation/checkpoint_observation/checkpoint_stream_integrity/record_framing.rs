pub(super) const PREFIX_BYTES: usize = 16;
pub(super) const CHECKSUM_BYTES: usize = 4;
pub(super) const CERTIFIED_SCHEMA: u8 = 3;

pub(super) fn supported_schema(prefix: &[u8]) -> Option<u8> {
    let schema = *prefix.get(8)?;
    matches!(schema, 1..=CERTIFIED_SCHEMA).then_some(schema)
}

pub(super) fn prefix_payload_bytes(prefix: &[u8], schema: u8, kind: u8) -> Option<usize> {
    if prefix.len() != PREFIX_BYTES
        || prefix.get(..8)? != b"WCP7REC\0"
        || supported_schema(prefix)? != schema
        || prefix[9] != kind
        || prefix.get(10..12)? != [0; 2]
    {
        return None;
    }
    usize::try_from(read_u32(prefix, 12)?).ok()
}

pub(super) fn fixed_record<'a>(
    record: &'a [u8],
    schema: u8,
    kind: u8,
    payload_bytes: usize,
) -> Option<&'a [u8]> {
    if prefix_payload_bytes(record.get(..PREFIX_BYTES)?, schema, kind)? != payload_bytes
        || record.len()
            != PREFIX_BYTES
                .checked_add(payload_bytes)?
                .checked_add(CHECKSUM_BYTES)?
    {
        return None;
    }
    let checksum_offset = record.len() - CHECKSUM_BYTES;
    (read_u32(record, checksum_offset)? == crc32c(&record[..checksum_offset]))
        .then_some(record.get(PREFIX_BYTES..checksum_offset)?)
}

pub(super) fn read_u32(bytes: &[u8], offset: usize) -> Option<u32> {
    Some(u32::from_le_bytes(
        bytes.get(offset..offset.checked_add(4)?)?.try_into().ok()?,
    ))
}

pub(super) fn read_u64(bytes: &[u8], offset: usize) -> Option<u64> {
    Some(u64::from_le_bytes(
        bytes.get(offset..offset.checked_add(8)?)?.try_into().ok()?,
    ))
}

fn crc32c(bytes: &[u8]) -> u32 {
    let mut crc = !0_u32;
    for byte in bytes {
        crc ^= u32::from(*byte);
        for _ in 0..8 {
            let mask = 0_u32.wrapping_sub(crc & 1);
            crc = (crc >> 1) ^ (0x82f6_3b78 & mask);
        }
    }
    !crc
}
