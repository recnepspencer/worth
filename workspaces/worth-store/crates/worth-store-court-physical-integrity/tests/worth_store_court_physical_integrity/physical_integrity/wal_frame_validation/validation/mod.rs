#[allow(unused_imports)]
use crate::wal_frame_validation::validation::*;

#[test]
fn wal_payload_projection_denies_equal_copy_and_foreign_segment() {
    let bytes = literal(CLEAN_HEX);
    let scope = scope(7, 1, 2, bytes.len() as u64);
    let input =
        worth_store_physical_integrity::UntrustedPhysicalArtifact::from_bounded_bytes(&bytes);
    let (validation, _) = validate_wal_frame(input, scope);
    let worth_store_physical_integrity::WalFrameIntegrityValidation::Intact(validated) = validation
    else {
        panic!("frozen WAL frame must validate");
    };
    let equal_copy = bytes.clone();
    assert_eq!(
        validated
            .project_payload(
                worth_store_physical_integrity::UntrustedPhysicalArtifact::from_bounded_bytes(
                    &equal_copy
                ),
                validated.segment_identity(),
            )
            .unwrap_err(),
        worth_store_physical_integrity::WalPayloadProjectionDenial::InputIncarnationMismatch
    );
    assert_eq!(
        validated
            .project_payload(
                input,
                worth_store_physical_format::WalSegmentIdentity::new(2, 2).unwrap(),
            )
            .unwrap_err(),
        worth_store_physical_integrity::WalPayloadProjectionDenial::SegmentIdentityMismatch
    );
}

#[test]
fn validation_record_binds_store_segment_generation_and_exact_range() {
    let bytes = literal(CLEAN_HEX);
    let left_scope = scope(7, 1, 2, bytes.len() as u64);
    let right_scope = scope(8, 1, 2, bytes.len() as u64);
    let left = intact_record(&bytes, left_scope);
    let right = intact_record(&bytes, right_scope);

    assert_ne!(left.exact_scope_digest(), right.exact_scope_digest());
    assert_eq!(left.byte_range_digest(), right.byte_range_digest());
    assert!(!left.matches_scope(right_scope));
    assert!(!left.matches_scope(scope(7, 1, 3, bytes.len() as u64)));
    assert!(!left.matches_scope(scope(7, 2, 2, bytes.len() as u64)));
    let shifted = worth_store_physical_integrity::PhysicalArtifactScope::wal_frame(
        store(7),
        left_scope.wal_segment_identity().unwrap(),
        worth_store_physical_integrity::PhysicalByteRange::new(
            FRAME_OFFSET + 1,
            bytes.len() as u64,
        )
        .unwrap(),
    );
    assert!(!left.matches_scope(shifted));
}

#[test]
fn strict_frame_truncation_localizes_only_the_missing_tail() {
    let bytes = literal(CLEAN_HEX);
    let scope = scope(7, 1, 2, bytes.len() as u64);
    let retained = &bytes[..bytes.len() - 1];
    let (rejection, counters) = rejection(retained, scope);
    assert_damage(
        rejection,
        scope,
        worth_store_physical_integrity::PhysicalDamageCause::Truncated,
        retained.len() as u64,
        1,
        None,
        worth_store_physical_integrity::PhysicalBlastRadius::CanonicalFrame,
    );
    assert_rejected_counters(
        counters,
        retained.len() as u64,
        worth_store_physical_integrity::PhysicalDamageCause::Truncated,
    );
}

#[test]
fn oversized_input_scope_mismatch_does_not_invent_a_corrupt_length_field() {
    let bytes = literal(CLEAN_HEX);
    let scope = scope(7, 1, 2, bytes.len() as u64 - 1);
    let (rejection, counters) = rejection(&bytes, scope);
    assert_damage(
        rejection,
        scope,
        worth_store_physical_integrity::PhysicalDamageCause::FramingLengthMismatch,
        0,
        150,
        None,
        worth_store_physical_integrity::PhysicalBlastRadius::CanonicalFrame,
    );
    assert_rejected_counters(
        counters,
        bytes.len() as u64,
        worth_store_physical_integrity::PhysicalDamageCause::FramingLengthMismatch,
    );
}
