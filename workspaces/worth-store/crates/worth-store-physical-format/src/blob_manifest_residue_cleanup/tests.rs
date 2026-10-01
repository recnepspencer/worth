use super::*;

fn intent() -> BlobManifestResidueCleanupV1 {
    BlobManifestResidueCleanupV1::intent(
        [1; 16],
        [2; 16],
        PersistedRecordIdentity::new([3; 16], 4).unwrap(),
        [5; 32],
        [6; 32],
        [7; 32],
        [8; 32],
        9,
        10,
        [11; 32],
        12,
        13,
    )
    .unwrap()
}

#[test]
fn cleanup_intent_and_exact_completion_roundtrip_without_payload_ids() {
    let intent = intent();
    let encoded = intent.encode();
    assert!(payload_is_blob_manifest_residue_cleanup(&encoded));
    assert_eq!(BlobManifestResidueCleanupV1::decode(&encoded), Ok(intent));
    let completed = intent.completed();
    assert_eq!(
        BlobManifestResidueCleanupV1::decode(&completed.encode()),
        Ok(completed),
    );
    assert_eq!(encoded.len(), WIRE_BYTES);
    assert!(encoded.len() < 1024);
}

#[test]
fn cleanup_wire_cannot_reinterpret_phase_domain_or_invalid_generation() {
    let mut encoded = intent().encode();
    encoded[8 + BLOB_MANIFEST_RESIDUE_CLEANUP_DOMAIN.len()] = 3;
    assert_eq!(
        BlobManifestResidueCleanupV1::decode(&encoded),
        Err(BlobManifestResidueCleanupDenial::Malformed),
    );
    let mut encoded = intent().encode();
    encoded[8] ^= 1;
    assert!(!payload_is_blob_manifest_residue_cleanup(&encoded));
    assert_eq!(
        BlobManifestResidueCleanupV1::decode(&encoded),
        Err(BlobManifestResidueCleanupDenial::Malformed),
    );
    let mut encoded = intent().encode();
    let candidate_offset = 8 + BLOB_MANIFEST_RESIDUE_CLEANUP_DOMAIN.len() + 1 + 192;
    encoded[candidate_offset..candidate_offset + 8].copy_from_slice(&11_u64.to_le_bytes());
    assert_eq!(
        BlobManifestResidueCleanupV1::decode(&encoded),
        Err(BlobManifestResidueCleanupDenial::InvalidBinding),
    );
}

#[test]
fn cleanup_binding_requires_exact_root_step_and_all_identity_digests() {
    let record = intent().manifest_record();
    assert_eq!(
        BlobManifestResidueCleanupV1::intent(
            [1; 16], [2; 16], record, [5; 32], [6; 32], [7; 32], [0; 32], 9, 10, [11; 32], 12, 13,
        ),
        Err(BlobManifestResidueCleanupDenial::InvalidBinding),
    );
    assert_eq!(
        BlobManifestResidueCleanupV1::intent(
            [1; 16], [2; 16], record, [5; 32], [6; 32], [7; 32], [8; 32], 9, 11, [11; 32], 12, 13,
        ),
        Err(BlobManifestResidueCleanupDenial::InvalidBinding),
    );
}
