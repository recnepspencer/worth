pub(crate) use {crate::phase_eight_process::history::parent_oracle::in_flight::record_binding::projection_contains_record, crate::phase_eight_process::history::parent_oracle::canonical_membership_placement::RecordIdentity, crate::phase_eight_process::history::parent_oracle::in_flight::record_binding::CURRENT_PROJECTION_DOMAIN};

pub(crate) fn record() -> RecordIdentity {
    RecordIdentity {
        allocation_epoch: [7; 16],
        ordinal: 9,
    }
}

pub(crate) fn projection_fixture(
    record: RecordIdentity,
    domain: &[u8],
    semantic: Option<&[u8]>,
) -> (Vec<u8>, usize, Option<usize>) {
    projection_fixture_variant(record, domain, semantic, false)
}

pub(crate) fn projection_fixture_variant(
    record: RecordIdentity,
    domain: &[u8],
    semantic: Option<&[u8]>,
    source_copy: bool,
) -> (Vec<u8>, usize, Option<usize>) {
    let mut encoded = Vec::new();
    field(&mut encoded, domain);
    encoded.extend_from_slice(&1_u64.to_le_bytes());
    field(&mut encoded, b"root-state");
    encoded.extend_from_slice(&1_u64.to_le_bytes());
    let mut identity = Vec::from(record.allocation_epoch);
    identity.extend_from_slice(&record.ordinal.to_le_bytes());
    field(&mut encoded, &identity);
    let tag_offset = encoded.len();
    if source_copy {
        encoded.push(1);
        field(&mut encoded, b"copy-intent");
        encoded.extend_from_slice(&3_u64.to_le_bytes());
        encoded.extend_from_slice(&[9; 32]);
    } else {
        encoded.push(0);
        encoded.extend_from_slice(&1_u64.to_le_bytes());
        field(&mut encoded, b"frame");
    }
    encoded.extend_from_slice(&1_u64.to_le_bytes());
    let mut placement = vec![2];
    placement.extend_from_slice(&identity);
    for value in [3_u64, 4, 1024, 5, 4096, 8192] {
        placement.extend_from_slice(&value.to_le_bytes());
    }
    placement.extend_from_slice(&[1, 0, 0, 0, 0, 0, 0]);
    field(&mut encoded, &placement);
    encoded.extend_from_slice(&0_u64.to_le_bytes());
    encoded.extend_from_slice(&0_u64.to_le_bytes());
    let semantic_offset = semantic.map(|bytes| {
        let offset = encoded.len() + 8;
        field(&mut encoded, bytes);
        offset
    });
    (encoded, tag_offset, semantic_offset)
}

pub(crate) fn current_ordinary_fixture(record: RecordIdentity) -> (Vec<u8>, usize, usize) {
    let mut encoded = Vec::new();
    field(&mut encoded, CURRENT_PROJECTION_DOMAIN);
    encoded.extend_from_slice(&1_u64.to_le_bytes());
    field(&mut encoded, b"root-state");
    encoded.extend_from_slice(&1_u64.to_le_bytes());
    let mut identity = Vec::from(record.allocation_epoch);
    identity.extend_from_slice(&record.ordinal.to_le_bytes());
    field(&mut encoded, &identity);
    encoded.push(0); // durable frame payload, not source copy
    encoded.extend_from_slice(&1_u64.to_le_bytes());
    field(&mut encoded, b"frame");
    encoded.extend_from_slice(&1_u64.to_le_bytes());
    let mut placement = vec![2];
    placement.extend_from_slice(&identity);
    for value in [3_u64, 4, 1024, 5, 4096, 8192] {
        placement.extend_from_slice(&value.to_le_bytes());
    }
    placement.extend_from_slice(&[1, 0, 0, 0, 0, 0, 0]); // Opaque, primary
    encoded.extend_from_slice(&(placement.len() as u64).to_le_bytes());
    let metadata = encoded.len() + placement.len() - 7;
    encoded.extend_from_slice(&placement);
    encoded.extend_from_slice(&0_u64.to_le_bytes());
    encoded.extend_from_slice(&0_u64.to_le_bytes());
    let operation_tag = encoded.len() + 8;
    field(&mut encoded, &[0]);
    (encoded, operation_tag, metadata)
}

pub(crate) fn field(target: &mut Vec<u8>, value: &[u8]) {
    target.extend_from_slice(&(value.len() as u64).to_le_bytes());
    target.extend_from_slice(value);
}