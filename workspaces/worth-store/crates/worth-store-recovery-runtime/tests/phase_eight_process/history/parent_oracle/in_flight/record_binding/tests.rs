use super::{projection_contains_record, RecordIdentity, CURRENT_PROJECTION_DOMAIN};

#[test]
fn selected_binding_reads_current_frame_layout() {
    let record = record();
    for source_copy in [false, true] {
        let (mut encoded, tag_offset, _) =
            projection_fixture_variant(record, CURRENT_PROJECTION_DOMAIN, Some(&[0]), source_copy);
        assert!(projection_contains_record(&encoded, record).unwrap());

        encoded[tag_offset] = u8::from(!source_copy);
        assert!(!projection_contains_record(&encoded, record).unwrap_or(false));
        encoded[tag_offset] = u8::from(source_copy);
        encoded.truncate(encoded.len() - 8);
        assert!(projection_contains_record(&encoded, record).is_err());
    }
}

#[test]
fn selected_binding_requires_exact_current_operation_and_matching_record() {
    let record = record();
    for semantic in [None, Some(&[][..]), Some(&[0, 0][..]), Some(&[3][..])] {
        let (encoded, _, _) = projection_fixture(record, CURRENT_PROJECTION_DOMAIN, semantic);
        assert!(!projection_contains_record(&encoded, record).unwrap_or(false));
    }
    let mut bound = vec![1];
    bound.extend_from_slice(&record.allocation_epoch);
    bound.extend_from_slice(&record.ordinal.to_le_bytes());
    bound.extend_from_slice(&[0; 32]); // the digest may be zero; its identity is checked elsewhere
    bound.extend_from_slice(&2_u64.to_le_bytes());
    let (encoded, _, _) = projection_fixture(record, CURRENT_PROJECTION_DOMAIN, Some(&bound));
    assert!(projection_contains_record(&encoded, record).unwrap());

    bound[1] ^= 1;
    let (encoded, _, _) = projection_fixture(record, CURRENT_PROJECTION_DOMAIN, Some(&bound));
    assert!(!projection_contains_record(&encoded, record).unwrap());
    bound[1] ^= 1;
    bound[57..65].copy_from_slice(&3_u64.to_le_bytes());
    let (encoded, _, _) = projection_fixture(record, CURRENT_PROJECTION_DOMAIN, Some(&bound));
    assert!(!projection_contains_record(&encoded, record).unwrap());

    let (encoded, _, _) = projection_fixture(record, CURRENT_PROJECTION_DOMAIN, Some(&[0, 0]));
    assert!(!projection_contains_record(&encoded, record).unwrap_or(false));
}

#[test]
fn selected_binding_preserves_current_source_copy_and_rejects_nonempty_operation() {
    let record = record();
    let (encoded, _, _) =
        projection_fixture_variant(record, CURRENT_PROJECTION_DOMAIN, Some(&[0]), true);
    assert!(projection_contains_record(&encoded, record).unwrap());
    let mut bound = vec![1];
    bound.extend_from_slice(&record.allocation_epoch);
    bound.extend_from_slice(&record.ordinal.to_le_bytes());
    bound.extend_from_slice(&[8; 32]);
    bound.extend_from_slice(&2_u64.to_le_bytes());
    let (encoded, _, _) =
        projection_fixture_variant(record, CURRENT_PROJECTION_DOMAIN, Some(&bound), true);
    assert!(!projection_contains_record(&encoded, record).unwrap());
}

#[test]
fn selected_binding_reads_independent_current_classified_ordinary_wire() {
    let record = record();
    let (mut encoded, operation_tag, metadata) = current_ordinary_fixture(record);
    assert!(projection_contains_record(&encoded, record).unwrap());
    assert!(!projection_contains_record(
        &encoded,
        RecordIdentity {
            ordinal: record.ordinal + 1,
            ..record
        }
    )
    .unwrap());

    // A different operation tag cannot reinterpret the current ordinary append.
    for tag in [1, 2] {
        encoded[operation_tag] = tag;
        assert!(projection_contains_record(&encoded, record).is_err());
    }
    encoded[operation_tag] = 0;
    for (position, value) in [(0, 9), (1, 1), (4, 3), (5, 1)] {
        let previous = encoded[metadata + position];
        encoded[metadata + position] = value;
        assert!(!projection_contains_record(&encoded, record).unwrap());
        encoded[metadata + position] = previous;
    }
    encoded.truncate(metadata + 6);
    assert!(projection_contains_record(&encoded, record).is_err());
}

#[test]
fn selected_binding_rejects_unsupported_projection_version_and_noncanonical_operation() {
    let record = record();
    let (mut encoded, _, _) = current_ordinary_fixture(record);
    let start = 8;
    let end = start + CURRENT_PROJECTION_DOMAIN.len();
    encoded[start..end].copy_from_slice(b"store.physical.recovery-projection.v12");
    assert!(!projection_contains_record(&encoded, record).unwrap());

    let (mut encoded, operation_tag, _) = current_ordinary_fixture(record);
    encoded[operation_tag] = 9;
    assert!(projection_contains_record(&encoded, record).is_err());
}

fn record() -> RecordIdentity {
    RecordIdentity {
        allocation_epoch: [7; 16],
        ordinal: 9,
    }
}

fn projection_fixture(
    record: RecordIdentity,
    domain: &[u8],
    semantic: Option<&[u8]>,
) -> (Vec<u8>, usize, Option<usize>) {
    projection_fixture_variant(record, domain, semantic, false)
}

fn projection_fixture_variant(
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

fn current_ordinary_fixture(record: RecordIdentity) -> (Vec<u8>, usize, usize) {
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

fn field(target: &mut Vec<u8>, value: &[u8]) {
    target.extend_from_slice(&(value.len() as u64).to_le_bytes());
    target.extend_from_slice(value);
}
