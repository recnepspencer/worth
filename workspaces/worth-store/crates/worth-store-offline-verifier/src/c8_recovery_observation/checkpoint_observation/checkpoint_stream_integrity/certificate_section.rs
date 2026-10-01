use sha2::{Digest, Sha256};

use super::record_framing::{fixed_record, prefix_payload_bytes, CHECKSUM_BYTES, PREFIX_BYTES};

const TIER_KIND: u8 = 6;
const RELEASE_KIND: u8 = 7;
const MAX_RECORDS: u64 = 64;
const MAX_ENCODED_BYTES: u64 = 65_536;
const MAX_PAYLOAD_BYTES: usize = 65_516;

pub(super) fn observe(
    bytes: &[u8],
    mut offset: usize,
    footer_offset: usize,
    count: u64,
    encoded_bytes: u64,
    digest: [u8; 32],
) -> Option<usize> {
    if count > MAX_RECORDS || encoded_bytes > MAX_ENCODED_BYTES {
        return None;
    }
    let start = offset;
    let mut observed_digest = Sha256::new();
    let mut tier_seen = false;
    for index in 0..count {
        let prefix_end = offset.checked_add(PREFIX_BYTES)?;
        let prefix = bytes.get(offset..prefix_end)?;
        let kind = *prefix.get(9)?;
        if !matches!(kind, TIER_KIND | RELEASE_KIND)
            || (kind == TIER_KIND && (tier_seen || index != 0))
        {
            return None;
        }
        let payload_bytes = prefix_payload_bytes(prefix, 3, kind)?;
        if payload_bytes == 0 || payload_bytes > MAX_PAYLOAD_BYTES {
            return None;
        }
        let end = prefix_end
            .checked_add(payload_bytes)?
            .checked_add(CHECKSUM_BYTES)?;
        if end > footer_offset {
            return None;
        }
        let record = bytes.get(offset..end)?;
        fixed_record(record, 3, kind, payload_bytes)?;
        observed_digest.update(record);
        tier_seen |= kind == TIER_KIND;
        offset = end;
    }
    (offset == footer_offset
        && u64::try_from(offset.checked_sub(start)?).ok()? == encoded_bytes
        && observed_digest.finalize().as_slice() == digest)
        .then_some(offset)
}
