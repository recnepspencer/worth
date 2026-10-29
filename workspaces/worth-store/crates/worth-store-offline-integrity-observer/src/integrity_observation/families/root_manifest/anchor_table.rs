//! Independent schema-12 head and anchor-table validation.
use super::*;
pub(super) fn validate(payload: &[u8]) -> Result<(), OfflineIntegrityOutcome> {
    if payload.len() != 3760
        || payload[560] > 1
        || payload[561] > 1
        || payload[562..568] != [0; 6]
        || read_u64(payload, 568) == 0
    {
        return Err(malformed_manifest());
    }
    if payload[561] == 0 && payload[576..680] != [0; 104] {
        return Err(pointer_damage(624, 104));
    }
    if payload[528..560] != [0; 32] && payload[560] != 1 {
        return Err(malformed_manifest());
    }
    if payload[561] == 1 {
        let head = &payload[576..680];
        if read_u64(head, 0) == 0
            || read_u64(head, 0) > read_u64(payload, 0)
            || read_u64(head, 8) == 0
            || read_u64(head, 8) >= read_u64(payload, 568)
            || head[18..24] != [0; 6]
            || !valid_record_identity(&head[24..48])
            || !valid_record_identity(&head[48..72])
            || !record_identity_le(&head[24..48], &head[48..72])
            || head[72..104] == [0; 32]
        {
            return Err(pointer_damage(624, 104));
        }
    }
    let table = &payload[680..];
    let count = usize::from(table[0]);
    if count == 0 || count > 32 || table[1..8] != [0; 7] {
        return Err(malformed_manifest());
    }
    let mut previous: Option<&[u8]> = None;
    for index in 0..32 {
        let entry = &table[8 + 96 * index..][..96];
        if index >= count {
            if entry != [0; 96] {
                return Err(malformed_manifest());
            }
            continue;
        }
        let name = &entry[..32];
        let end = name.iter().position(|b| *b == 0).unwrap_or(32);
        if end == 0
            || name[end..].iter().any(|b| *b != 0)
            || !name[..end]
                .iter()
                .all(|b| matches!(b, b'a'..=b'z' | b'0'..=b'9' | b'.' | b'-'))
            || previous.is_some_and(|p| p >= name)
            || read_u64(entry, 88) == 0
            || !valid_record_identity(&entry[32..56])
            || payload[40] != 1
            || !routing_contains(&payload[48..120], &entry[32..56])
        {
            return Err(malformed_manifest());
        }
        previous = Some(name);
    }
    Ok(())
}
