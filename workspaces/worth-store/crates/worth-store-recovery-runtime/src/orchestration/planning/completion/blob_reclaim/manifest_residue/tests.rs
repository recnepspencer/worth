use super::*;
use worth_store_physical_format::{BlobManifestResidueCleanupV1, FailedIngestReclaimBasisV1};

fn record(ordinal: u64) -> PersistedRecordIdentity {
    PersistedRecordIdentity::new([7; 16], ordinal).unwrap()
}

fn basis() -> FailedIngestReclaimBasisV1 {
    FailedIngestReclaimBasisV1::new([9; 16], record(1), [10; 32], record(2), [11; 32]).unwrap()
}

fn intent(
    attempt: [u8; 16],
    manifest_sha: [u8; 32],
    source_digest: [u8; 32],
) -> BlobManifestResidueCleanupV1 {
    BlobManifestResidueCleanupV1::intent(
        [1; 16],
        attempt,
        record(4),
        manifest_sha,
        source_digest,
        [12; 32],
        [13; 32],
        5,
        6,
        [14; 32],
        4096,
        8,
    )
    .unwrap()
}

#[test]
fn selected_manifest_requires_exact_attempt_source_and_frame_sha() {
    let manifest = DropSetManifestV1::new([1; 16], [2; 16], basis(), vec![record(3)]).unwrap();
    let bytes = manifest.encode();
    let frame_sha = Sha256::digest(&bytes).into();
    let matching = intent([2; 16], frame_sha, manifest.source_basis_digest());
    assert!(manifest_matches_intent(matching, &manifest, &bytes));
    assert!(!manifest_matches_intent(
        intent([8; 16], frame_sha, manifest.source_basis_digest()),
        &manifest,
        &bytes,
    ));
    assert!(!manifest_matches_intent(
        intent([2; 16], [5; 32], manifest.source_basis_digest()),
        &manifest,
        &bytes,
    ));
    assert!(!manifest_matches_intent(
        intent([2; 16], frame_sha, [6; 32]),
        &manifest,
        &bytes,
    ));
    let mut altered = bytes.clone();
    altered[55] ^= 1;
    assert!(!manifest_matches_intent(matching, &manifest, &altered));
}
