use super::super::canonical_membership_placement::RecordIdentity;

/// Count completed checkpoint terminal records that bind the operation to the
/// expected physical record. The checkpoint wire stores record identities,
/// while the parent root-membership oracle proves that identity's payload.
pub(crate) fn scan_checkpoint_binding(
    bytes: &[u8],
    identity: &[u8],
    expected_record: RecordIdentity,
    expected_redo_digest: Option<&[u8; 32]>,
) -> Result<usize, String> {
    let mut offset = 0;
    let mut matches = 0;
    while offset < bytes.len() {
        let (total, payload, kind) = checkpoint_record(bytes, offset)?;
        if kind == 4 {
            if let Some(digest) = completed_binding_redo_digest(payload, identity, expected_record)
            {
                if expected_redo_digest.is_none_or(|expected| digest == *expected) {
                    matches += 1;
                }
            }
        }
        offset += total;
    }
    Ok(matches)
}

pub(super) fn scan_identity(bytes: &[u8], identity: &[u8]) -> Result<bool, String> {
    let mut offset = 0;
    let mut found_identity = false;
    while offset < bytes.len() {
        let (total, payload, kind) = checkpoint_record(bytes, offset)?;
        if kind == 4 {
            found_identity |= super::binding_record_matches(payload, identity);
        }
        offset += total;
    }
    Ok(found_identity)
}

fn checkpoint_record(bytes: &[u8], offset: usize) -> Result<(usize, &[u8], u8), String> {
    let prefix = bytes
        .get(offset..offset + super::CHECKPOINT_PREFIX_BYTES)
        .ok_or_else(|| "semantic checkpoint oracle found a truncated record".to_owned())?;
    if &prefix[..8] != b"WCP7REC\0" || prefix[8] != 1 {
        return Err("semantic checkpoint oracle found an invalid record".to_owned());
    }
    let payload_bytes = usize::try_from(super::read_u32(prefix, 12).unwrap_or(0))
        .map_err(|_| "semantic checkpoint oracle payload length overflowed".to_owned())?;
    let total = super::CHECKPOINT_PREFIX_BYTES
        .checked_add(payload_bytes)
        .and_then(|value| value.checked_add(4))
        .ok_or_else(|| "semantic checkpoint oracle record length overflowed".to_owned())?;
    let record = bytes
        .get(offset..offset + total)
        .ok_or_else(|| "semantic checkpoint oracle found a truncated record".to_owned())?;
    if super::crc32c(&record[..super::CHECKPOINT_PREFIX_BYTES + payload_bytes])
        != super::read_u32(record, super::CHECKPOINT_PREFIX_BYTES + payload_bytes).unwrap_or(0)
    {
        return Err("semantic checkpoint oracle rejected a record checksum".to_owned());
    }
    Ok((
        total,
        &record[super::CHECKPOINT_PREFIX_BYTES..super::CHECKPOINT_PREFIX_BYTES + payload_bytes],
        prefix[9],
    ))
}

fn completed_binding_redo_digest(
    bytes: &[u8],
    identity: &[u8],
    expected_record: RecordIdentity,
) -> Option<[u8; 32]> {
    let mut cursor = super::Cursor::new(bytes);
    if cursor.field() != Some(super::COMPACTION_DOMAIN) || cursor.byte() != Some(3) {
        return None;
    }
    if !super::basis_matches(&mut cursor, identity) || cursor.byte() != Some(2) {
        return None;
    }
    let binding = cursor.field()?;
    let binding_redo_digest = super::binding_redo_digest(binding, identity, None)?;
    if cursor.u32().is_none() || cursor.u64().is_none() {
        return None;
    }
    let record_count = cursor.u32()?;
    let mut record_matches = 0;
    for _ in 0..record_count {
        let allocation_epoch = cursor.array_field(16)?;
        let ordinal = cursor.u64()?;
        record_matches += usize::from(
            RecordIdentity {
                allocation_epoch: allocation_epoch.try_into().unwrap_or([0; 16]),
                ordinal,
            } == expected_record,
        );
    }
    for _ in 0..13 {
        cursor.u64()?;
    }
    (record_matches == 1 && cursor.is_empty()).then_some(binding_redo_digest)
}
