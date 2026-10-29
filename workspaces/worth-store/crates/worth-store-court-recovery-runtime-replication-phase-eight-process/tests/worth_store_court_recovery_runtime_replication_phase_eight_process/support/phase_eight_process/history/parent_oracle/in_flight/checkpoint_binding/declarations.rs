pub(crate) use {crate::phase_eight_process::history::parent_oracle::canonical_membership_placement::RecordIdentity};

/// Count completed checkpoint terminal records that bind the operation to the
/// expected physical record. The checkpoint wire stores record identities,
/// while the parent root-membership oracle proves that identity's payload.
pub(crate) fn scan_checkpoint_binding(
    bytes: &[u8],
    identity: &[u8],
    expected_record: RecordIdentity,
    expected_redo_digest: Option<&[u8; 32]>,
) -> Result<usize, String> {
    let mut matches = 0;
    let bindings = crate::phase_eight_process::history::parent_oracle::wire::validated_checkpoint_binding_payloads(bytes)
        .ok_or_else(|| "semantic checkpoint oracle rejected checkpoint".to_owned())?;
    for payload in bindings {
        if let Some(digest) = completed_binding_redo_digest(payload, identity, expected_record)
            && expected_redo_digest.is_none_or(|expected| digest == *expected)
        {
            matches += 1;
        }
    }
    Ok(matches)
}

pub(crate) fn scan_identity(bytes: &[u8], identity: &[u8]) -> Result<bool, String> {
    let mut found_identity = false;
    let bindings = crate::phase_eight_process::history::parent_oracle::wire::validated_checkpoint_binding_payloads(bytes)
        .ok_or_else(|| "semantic checkpoint oracle rejected checkpoint".to_owned())?;
    for payload in bindings {
        found_identity |= crate::history::parent_oracle::in_flight::binding_record_matches(payload, identity);
    }
    Ok(found_identity)
}

pub(crate) fn completed_binding_redo_digest(
    bytes: &[u8],
    identity: &[u8],
    expected_record: RecordIdentity,
) -> Option<[u8; 32]> {
    let mut cursor = crate::history::parent_oracle::in_flight::Cursor::new(bytes);
    if cursor.field() != Some(crate::history::parent_oracle::in_flight::COMPACTION_DOMAIN) || cursor.byte() != Some(3) {
        return None;
    }
    if !crate::history::parent_oracle::in_flight::basis_matches(&mut cursor, identity) || cursor.byte() != Some(2) {
        return None;
    }
    let binding = cursor.field()?;
    let binding_redo_digest = crate::history::parent_oracle::in_flight::binding_redo_digest(binding, identity, None)?;
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