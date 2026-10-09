use super::*;

fn record(ordinal: u64) -> PersistedRecordIdentity {
    PersistedRecordIdentity::new([3; 16], ordinal).unwrap()
}

fn intent(proof: OriginalDropProofV1) -> BlobManifestResidueCleanupV2 {
    BlobManifestResidueCleanupV2::intent(
        [1; 16],
        [2; 16],
        record(4),
        [5; 32],
        [6; 32],
        proof,
        7,
        8,
        [9; 32],
        4096,
        10,
    )
    .unwrap()
}

#[test]
fn explicit_no_prep_and_exact_reserved_positive_proofs_round_trip() {
    let never = intent(OriginalDropProofV1::NeverReserved);
    assert_eq!(
        BlobManifestResidueCleanup::decode(&never.encode()),
        Ok(BlobManifestResidueCleanup::V2(never))
    );
    assert_eq!(
        BlobManifestResidueCleanup::decode(&never.completed().encode()),
        Ok(BlobManifestResidueCleanup::V2(never.completed()))
    );
    let reserved = ReservedDropRecordV1::new(record(5), [7; 32]).unwrap();
    let positive = intent(OriginalDropProofV1::ProvenNoEffect {
        idempotency: [8; 32],
        fingerprint: [9; 32],
        reserved: Some(reserved),
    });
    let decoded = BlobManifestResidueCleanup::decode(&positive.encode()).unwrap();
    assert_eq!(decoded, BlobManifestResidueCleanup::V2(positive));
    assert_eq!(decoded.reserved_record(), Some(reserved));
    let recovered = intent(OriginalDropProofV1::RecoveredNoBinding {
        idempotency: [8; 32],
        fingerprint: [9; 32],
        reserved,
    });
    assert_eq!(
        BlobManifestResidueCleanup::decode(&recovered.encode()),
        Ok(BlobManifestResidueCleanup::V2(recovered))
    );
    assert_eq!(
        BlobManifestResidueCleanup::decode(&recovered.completed().encode()),
        Ok(BlobManifestResidueCleanup::V2(recovered.completed()))
    );
    assert_eq!(
        BlobManifestResidueCleanup::V2(recovered).reserved_record(),
        Some(reserved)
    );
}

#[test]
fn invalid_or_truncated_v2_proof_does_not_fall_back_to_v1() {
    let mut bytes = intent(OriginalDropProofV1::NeverReserved).encode();
    assert!(payload_is_blob_manifest_residue_cleanup_any(&bytes));
    bytes.pop();
    assert!(BlobManifestResidueCleanup::decode(&bytes).is_err());
    let positive_without_reserved = OriginalDropProofV1::ProvenNoEffect {
        idempotency: [8; 32],
        fingerprint: [9; 32],
        reserved: None,
    };
    assert!(BlobManifestResidueCleanupV2::intent(
        [1; 16],
        [2; 16],
        record(4),
        [5; 32],
        [6; 32],
        positive_without_reserved,
        7,
        8,
        [9; 32],
        4096,
        10,
    )
    .is_err());
    let same_record = ReservedDropRecordV1::new(record(4), [7; 32]).unwrap();
    assert!(BlobManifestResidueCleanupV2::intent(
        [1; 16],
        [2; 16],
        record(4),
        [5; 32],
        [6; 32],
        OriginalDropProofV1::ProvenNoEffect {
            idempotency: [8; 32],
            fingerprint: [9; 32],
            reserved: Some(same_record),
        },
        7,
        8,
        [9; 32],
        4096,
        10,
    )
    .is_err());
    assert!(BlobManifestResidueCleanupV2::intent(
        [1; 16],
        [2; 16],
        record(4),
        [5; 32],
        [6; 32],
        OriginalDropProofV1::RecoveredNoBinding {
            idempotency: [0; 32],
            fingerprint: [9; 32],
            reserved: same_record,
        },
        7,
        8,
        [9; 32],
        4096,
        10,
    )
    .is_err());
    let mut invalid_tag = intent(OriginalDropProofV1::NeverReserved).encode();
    invalid_tag[8 + BLOB_MANIFEST_RESIDUE_CLEANUP_V2_DOMAIN.len() + 1] = 3;
    assert!(
        BlobManifestResidueCleanup::decode(&invalid_tag).is_err(),
        "tag3 may not interpret an all-zero NeverReserved body as a recovered certificate"
    );
}
