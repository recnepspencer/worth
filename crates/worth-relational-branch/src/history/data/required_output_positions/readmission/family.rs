//! Chunked UTF-8 validation, owned copying and lexicographic route comparison.
use super::{PublicationRoute, Readmission, Refusal};
use std::cmp::Ordering;

const CHUNK_BYTES: usize = 64 * 1024;

pub(super) fn copy(bytes: &[u8], context: &Readmission<'_, '_, '_>) -> Result<String, Refusal> {
    // The caller already admitted the exact String backing and its route Arc.
    let mut owned = String::with_capacity(bytes.len());
    let mut at = 0;
    while at < bytes.len() {
        let mut end = at.saturating_add(CHUNK_BYTES).min(bytes.len());
        if end < bytes.len() {
            // A UTF-8 scalar has at most three continuation octets. Inspect only
            // this bounded boundary; malformed longer runs fail UTF-8 validation.
            for _ in 0..3 {
                if bytes[end] & 0xc0 != 0x80 {
                    break;
                }
                end -= 1;
            }
        }
        let raw = &bytes[at..end];
        context.charge(raw.len() as u64)?;
        let text = std::str::from_utf8(raw).map_err(|_| Refusal::InvalidProjectedMeaning)?;
        context.charge(raw.len() as u64)?;
        owned.push_str(text);
        at = end;
    }
    Ok(owned)
}
pub(super) fn compare(
    left: &PublicationRoute,
    right: &PublicationRoute,
    context: &Readmission<'_, '_, '_>,
) -> Result<Ordering, Refusal> {
    let left_bytes = left.family.as_bytes();
    let right_bytes = right.family.as_bytes();
    let length = left_bytes.len().min(right_bytes.len());
    let mut at = 0;
    while at < length {
        // Both compared inputs together fit the declared maximum poll interval.
        let end = at.saturating_add(CHUNK_BYTES / 2).min(length);
        context.charge(((end - at) * 2) as u64)?;
        let ordering = left_bytes[at..end].cmp(&right_bytes[at..end]);
        if ordering != Ordering::Equal {
            return Ok(ordering);
        }
        at = end;
    }
    // Length, EntityId and the two fixed partition digests have bounded width.
    context.charge(128)?;
    Ok(left_bytes
        .len()
        .cmp(&right_bytes.len())
        .then_with(|| left.source.cmp(&right.source))
        .then_with(|| left.partition.cmp(&right.partition)))
}
