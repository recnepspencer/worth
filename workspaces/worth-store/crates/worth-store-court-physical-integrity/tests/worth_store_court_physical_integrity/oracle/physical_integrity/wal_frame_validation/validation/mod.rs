#[allow(unused_imports)]
use crate::wal_frame_validation::validation::*;

#[test]
fn frozen_literal_exposes_lsn_payload_and_record_only_after_complete_validation() {
    let bytes = literal(CLEAN_HEX);
    assert_independent_frame_checksums(&bytes);
    assert_eq!(independent_sha256(b"c9-wal-v1-golden"), bytes[52..84]);
    let scope = scope(7, 1, 2, bytes.len() as u64);
    assert_eq!(scope.wal_segment_identity().unwrap().segment().get(), 1);
    let input =
        worth_store_physical_integrity::UntrustedPhysicalArtifact::from_bounded_bytes(&bytes);
    let (validation, counters) = validate_wal_frame(input, scope);
    let worth_store_physical_integrity::WalFrameIntegrityValidation::Intact(validated) = validation
    else {
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
        worth_store_physical_integrity::PhysicalIntegrityValidationMechanism::Sha256V1
    );
    assert_eq!(
        record.byte_range_digest(),
        worth_store_physical_integrity::PhysicalIntegrityValidationDigest::sha256(
            bytes[bytes.len() - 32..].try_into().unwrap()
        )
    );
    assert_eq!(counters.family(), worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily::WalFrame);
    assert_eq!(
        (counters.inspected_frames(), counters.inspected_bytes()),
        (1, 151)
    );
    assert_eq!(
        (counters.intact_frames(), counters.rejected_frames()),
        (1, 0)
    );
}
