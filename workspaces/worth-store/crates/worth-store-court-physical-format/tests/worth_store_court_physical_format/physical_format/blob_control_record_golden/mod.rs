//! Frozen blob control-record frames shared with the offline observer.
//!
//! The observer holds byte-identical copies of these literals in
//! `worth-store-offline-integrity-observer/src/integrity_observation/blob_record/`
//! (`reuse_claim.rs` for kinds 11 and 15, `quarantine.rs` for kind 12) and
//! parses them with its independent reader. Change both copies together: a
//! field-order change in this encoder then fails here, not silently there.



#[allow(unused_imports)] use crate::blob_control_record_golden::*;
use {crate::store_physical_format_support::independent_sha256};
use {worth_store_physical_format::BlobChunkReuseClaimV1, worth_store_physical_format::BlobChunkReuseClaimV2, worth_store_physical_format::BlobDedupeQuarantineV1, worth_store_physical_format::BlobGenerationPublicationV1, worth_store_physical_format::PersistedRecordIdentity};

#[test]
fn reuse_claim_v1_encoder_matches_the_shared_golden() {
    let encoded = claim(7, record(5, 6), record(7, 8)).encode();
    assert_eq!(hex(&encoded), REUSE_CLAIM_V1_FRAME_HEX);
}

#[test]
fn dedupe_quarantine_v1_encoder_matches_the_shared_golden() {
    let encoded = worth_store_physical_format::BlobDedupeQuarantineV1::new(
        [2; 16],
        [3; 32],
        [4; 32],
        CHUNK,
        record(1, 1),
        0,
        record(1, 2),
        [5; 16],
        0,
        record(1, 3),
    )
    .expect("valid dedupe quarantine")
    .encode();
    assert_eq!(hex(&encoded), DEDUPE_QUARANTINE_V1_FRAME_HEX);
}

#[test]
fn reuse_claim_v2_encoder_matches_the_shared_golden() {
    let publication = worth_store_physical_format::BlobGenerationPublicationV1::new(
        [1; 16],
        [9; 16],
        [4; 16],
        1,
        record(7, 8),
        [6; 32],
        u64::from(CHUNK),
        [5; 32],
        CHUNK,
        [3; 32],
    )
    .expect("valid publication");
    let digest = independent_sha256(&publication.encode());
    let encoded =
        worth_store_physical_format::BlobChunkReuseClaimV2::new(claim(0, record(7, 6), record(7, 9)), publication, digest)
            .expect("valid versioned reuse claim")
            .encode();
    assert_eq!(hex(&encoded), REUSE_CLAIM_V2_FRAME_HEX);
}
