use worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily;
use worth_store_physical_integrity::{
    validate_wal_frame, PhysicalArtifactScope, PhysicalBlastRadius, PhysicalByteRange,
    PhysicalDamageCause, PhysicalIntegrityValidationDigest, PhysicalIntegrityValidationMechanism,
    UntrustedPhysicalArtifact, WalFrameIntegrityValidation, WalPayloadProjectionDenial,
};

use super::sha256::independent_sha256;
use super::support::{
    assert_damage, assert_independent_frame_checksums, assert_rejected_counters, literal,
    rejection, scope, store, CLEAN_HEX, FRAME_OFFSET,
};

#[test]
fn frozen_literal_exposes_lsn_payload_and_record_only_after_complete_validation() {
    let bytes = literal(CLEAN_HEX);
    assert_independent_frame_checksums(&bytes);
    assert_eq!(independent_sha256(b"c9-wal-v1-golden"), bytes[52..84]);
    let scope = scope(7, 1, 2, bytes.len() as u64);
    assert_eq!(scope.wal_segment_identity().unwrap().segment().get(), 1);
    let input = UntrustedPhysicalArtifact::from_bounded_bytes(&bytes);
    let (validation, counters) = validate_wal_frame(input, scope);
    let WalFrameIntegrityValidation::Intact(validated) = validation else {
        panic!("frozen WAL frame must validate");
    };

    assert!(validated.matches_input(input));
    assert_eq!(validated.scope(), scope);
    assert_eq!(
        validated.segment_identity(),
        scope.wal_segment_identity().unwrap()
    );
    assert_eq!((validated.lsn_start(), validated.lsn_end()), (3, 4));
    let payload = validated
        .project_payload(input, validated.segment_identity())
        .unwrap();
    assert_eq!((payload.lsn_start(), payload.lsn_end()), (3, 4));
    assert_eq!(&bytes[payload.payload_range()], [0x10, 0x20, 0x30]);
    assert_eq!(validated.payload_digest(), bytes[84..116]);
    assert_eq!(validated.identity_digest(), bytes[52..84]);
    let record = validated.into_validation_record();
    assert!(record.matches_scope(scope));
    assert_eq!(
        record.mechanism(),
        PhysicalIntegrityValidationMechanism::Sha256V1
    );
    assert_eq!(
        record.byte_range_digest(),
        PhysicalIntegrityValidationDigest::sha256(bytes[bytes.len() - 32..].try_into().unwrap())
    );
    assert_eq!(counters.family(), PhysicalIntegrityArtifactFamily::WalFrame);
    assert_eq!(
        (counters.inspected_frames(), counters.inspected_bytes()),
        (1, 151)
    );
    assert_eq!(
        (counters.intact_frames(), counters.rejected_frames()),
        (1, 0)
    );
}

#[test]
fn wal_payload_projection_denies_equal_copy_and_foreign_segment() {
    let bytes = literal(CLEAN_HEX);
    let scope = scope(7, 1, 2, bytes.len() as u64);
    let input = UntrustedPhysicalArtifact::from_bounded_bytes(&bytes);
    let (validation, _) = validate_wal_frame(input, scope);
    let WalFrameIntegrityValidation::Intact(validated) = validation else {
        panic!("frozen WAL frame must validate");
    };
    let equal_copy = bytes.clone();
    assert_eq!(
        validated
            .project_payload(
                UntrustedPhysicalArtifact::from_bounded_bytes(&equal_copy),
                validated.segment_identity(),
            )
            .unwrap_err(),
        WalPayloadProjectionDenial::InputIncarnationMismatch
    );
    assert_eq!(
        validated
            .project_payload(
                input,
                worth_store_physical_format::WalSegmentIdentity::new(2, 2).unwrap(),
            )
            .unwrap_err(),
        WalPayloadProjectionDenial::SegmentIdentityMismatch
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
    let shifted = PhysicalArtifactScope::wal_frame(
        store(7),
        left_scope.wal_segment_identity().unwrap(),
        PhysicalByteRange::new(FRAME_OFFSET + 1, bytes.len() as u64).unwrap(),
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
        PhysicalDamageCause::Truncated,
        retained.len() as u64,
        1,
        None,
        PhysicalBlastRadius::CanonicalFrame,
    );
    assert_rejected_counters(
        counters,
        retained.len() as u64,
        PhysicalDamageCause::Truncated,
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
        PhysicalDamageCause::FramingLengthMismatch,
        0,
        150,
        None,
        PhysicalBlastRadius::CanonicalFrame,
    );
    assert_rejected_counters(
        counters,
        bytes.len() as u64,
        PhysicalDamageCause::FramingLengthMismatch,
    );
}

fn intact_record(
    bytes: &[u8],
    scope: PhysicalArtifactScope,
) -> worth_store_physical_integrity::PhysicalIntegrityValidationRecord {
    let (validation, _) =
        validate_wal_frame(UntrustedPhysicalArtifact::from_bounded_bytes(bytes), scope);
    let WalFrameIntegrityValidation::Intact(validated) = validation else {
        panic!("frozen frame must validate under a source-bound descriptive scope");
    };
    validated.into_validation_record()
}
