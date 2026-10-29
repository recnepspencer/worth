#[allow(unused_imports)] use crate::phase_eight_process::history::parent_oracle::in_flight::record_binding::tests::*;
use {crate::phase_eight_process::history::parent_oracle::in_flight::record_binding::projection_contains_record, crate::phase_eight_process::history::parent_oracle::canonical_membership_placement::RecordIdentity, crate::phase_eight_process::history::parent_oracle::in_flight::record_binding::CURRENT_PROJECTION_DOMAIN};

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
