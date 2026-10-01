use super::*;
use worth_store_physical_format::{BlobGenerationPublicationV1, ReleasedGenerationReclaimBasisV1};

fn record(ordinal: u64) -> PersistedRecordIdentity {
    PersistedRecordIdentity::new([4; 16], ordinal).unwrap()
}

fn fixture() -> (DropSetManifestV3, BlobReclaimDescriptorV2, Vec<u8>) {
    let publication = BlobGenerationPublicationV1::new(
        [1; 16],
        [2; 16],
        [3; 16],
        4,
        record(2),
        [5; 32],
        64 << 10,
        [6; 32],
        64 << 10,
        [7; 32],
    )
    .unwrap();
    let source = ReleasedGenerationReclaimBasisV1::new(
        publication,
        record(1),
        Sha256::digest(publication.encode()).into(),
        [8; 32],
    )
    .unwrap();
    let manifest = DropSetManifestV3::new(
        [1; 16],
        [9; 16],
        BlobReclaimSourceBasisV1::ReleasedGeneration(source),
        vec![record(1)],
        6,
    )
    .unwrap();
    let bytes = manifest.encode();
    let descriptor = BlobReclaimDescriptorV2::new(
        [1; 16],
        [9; 16],
        BlobReclaimSourceKind::ReleasedGeneration,
        manifest.source_basis_digest(),
        record(3),
        Sha256::digest(&bytes).into(),
        manifest.count(),
        7,
        8,
        None,
        1,
        false,
    )
    .unwrap();
    (manifest, descriptor, bytes)
}

#[test]
fn released_preflight_rejects_changed_manifest_and_wrong_source_kind() {
    let (manifest, descriptor, bytes) = fixture();
    assert!(binding_matches(descriptor, &manifest, &bytes, record(4)));
    let mut altered = bytes.clone();
    altered[16] ^= 1;
    assert!(!binding_matches(descriptor, &manifest, &altered, record(4)));
    let wrong_kind = BlobReclaimDescriptorV2::new(
        descriptor.store(),
        descriptor.reclaim_attempt(),
        BlobReclaimSourceKind::FailedIngest,
        descriptor.source_basis_digest(),
        descriptor.manifest_record(),
        descriptor.manifest_frame_sha256(),
        descriptor.manifest_count(),
        descriptor.source_root_generation(),
        descriptor.candidate_root_generation(),
        None,
        1,
        false,
    )
    .unwrap();
    assert!(!binding_matches(wrong_kind, &manifest, &bytes, record(4)));
}
