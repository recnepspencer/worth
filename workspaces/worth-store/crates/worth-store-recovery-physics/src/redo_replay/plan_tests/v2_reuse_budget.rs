use sha2::{Digest, Sha256};
use worth_store_physical_format::{
    decode_blob_record, validate_blob_record_header, BlobChunkReuseClaimV1,
    BlobChunkReuseClaimV2, BlobGenerationPublicationV1, BlobRecordKind,
    PersistedBlobSemanticRecordBinding, PersistedPhysicalRecoveryOperation,
    PersistedRecordIdentity,
};

use super::semantic_budget::{encode_extent_member, find_exact_budget, generous_limits};
use super::*;

#[test]
fn v2_chunk_reuse_claim_semantic_record_admits_at_exact_budget() {
    let store_bytes = test_store().bytes();
    let record = PersistedRecordIdentity::new([0x41; 16], 1).unwrap();
    let ref_rec = |ordinal| PersistedRecordIdentity::new([0x42; 16], ordinal).unwrap();
    let publication = BlobGenerationPublicationV1::new(
        store_bytes,
        [0x43; 16],
        [0x44; 16],
        1,
        ref_rec(4),
        [0x45; 32],
        64 * 1024 + 17,
        [0x46; 32],
        64 * 1024,
        [0x47; 32],
    )
    .unwrap();
    let claim = BlobChunkReuseClaimV1::new(
        store_bytes,
        [0x48; 16],
        0,
        [0x47; 32],
        64 * 1024,
        17,
        [0x49; 32],
        ref_rec(11),
        ref_rec(12),
        1,
    )
    .unwrap();
    let pub_digest: [u8; 32] = Sha256::digest(publication.encode()).into();
    let v2 = BlobChunkReuseClaimV2::new(claim, publication, pub_digest).unwrap();
    let payload = v2.encode();
    let header = validate_blob_record_header(&payload).expect("v2 claim validates");
    assert_eq!(header.kind(), BlobRecordKind::ChunkReuseClaimV2);
    assert_eq!(header.store(), store_bytes);
    assert_eq!(decode_blob_record(&payload).unwrap().kind(), header.kind());

    let binding =
        PersistedBlobSemanticRecordBinding::new(record, Sha256::digest(&payload).into(), 2)
            .unwrap();
    let member = encode_extent_member(
        record,
        &payload,
        PersistedPhysicalRecoveryOperation::ChunkReused(binding),
        false,
    );
    let limits = generous_limits();
    let exact = find_exact_budget(&member, limits);
    assert!(exact > 0);
    assert_eq!(
        admit_physical_redo_members_with_memory_budget(
            vec![member.clone()],
            test_store(),
            test_format(),
            limits,
            exact - 1,
        )
        .err(),
        Some(PhysicalRedoPlanningDenial::RecoveryMemoryLimit {
            observed: exact,
            admitted: exact - 1,
        })
    );
    let admitted = admit_physical_redo_members_with_memory_budget(
        vec![member],
        test_store(),
        test_format(),
        limits,
        exact,
    )
    .expect("exact budget admits extent-backed V2 chunk reuse claim");
    assert_eq!(admitted.target_identity_count(), 1);
    assert_eq!(admitted.distinct_target_count(), 1);
}
