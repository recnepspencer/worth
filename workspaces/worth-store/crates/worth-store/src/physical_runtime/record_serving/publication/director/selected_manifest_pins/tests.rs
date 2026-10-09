use super::*;
use sha2::{Digest, Sha256};
use worth_store_physical_format::{
    BlobGenerationPublicationV1, BlobReclaimSourceBasisV1, DropSetManifestV1, DropSetManifestV2,
    DropSetManifestV3, FailedIngestReclaimBasisV1, ReleasedGenerationReclaimBasisV1,
};

#[test]
fn selected_manifest_roster_denies_conflicting_attempt_and_corrupt_custody() {
    let record = |ordinal| PersistedRecordIdentity::new([3; 16], ordinal).unwrap();
    let basis =
        FailedIngestReclaimBasisV1::new([4; 16], record(1), [5; 32], record(2), [6; 32]).unwrap();
    let manifest = DropSetManifestV1::new([1; 16], [2; 16], basis, vec![record(3)])
        .unwrap()
        .encode();
    let mut pins = Vec::new();
    let mut attempts = HashSet::new();
    assert!(observe_manifest(
        &manifest,
        record(4),
        [1; 16],
        10,
        2,
        4096,
        1024,
        &mut pins,
        &mut attempts,
    )
    .is_ok());
    assert_eq!(pins.len(), 1);
    assert!(observe_manifest(
        &manifest,
        record(9),
        [1; 16],
        10,
        1,
        1024,
        MINIMUM_MANIFEST_FRAME_BYTES,
        &mut Vec::new(),
        &mut HashSet::new(),
    )
    .is_ok());
    let mut tight_pins = Vec::new();
    let mut tight_attempts = HashSet::new();
    assert_eq!(
        observe_manifest(
            &manifest,
            record(8),
            [1; 16],
            10,
            2,
            700,
            MINIMUM_MANIFEST_FRAME_BYTES,
            &mut tight_pins,
            &mut tight_attempts,
        ),
        Err(SelectedBlobManifestPinDenial::BudgetExceeded)
    );
    assert!(tight_pins.is_empty() && tight_attempts.is_empty());
    assert_eq!(
        observe_manifest(
            &manifest,
            record(5),
            [1; 16],
            10,
            2,
            4096,
            1024,
            &mut pins,
            &mut attempts,
        ),
        Err(SelectedBlobManifestPinDenial::ConflictingAttempt)
    );
    let mut corrupt = manifest.clone();
    *corrupt.last_mut().unwrap() ^= 1;
    assert_eq!(
        observe_manifest(
            &corrupt,
            record(6),
            [1; 16],
            10,
            2,
            4096,
            1024,
            &mut Vec::new(),
            &mut HashSet::new(),
        ),
        Err(SelectedBlobManifestPinDenial::MalformedManifest)
    );
    assert_eq!(
        observe_manifest(
            &manifest,
            record(7),
            [9; 16],
            10,
            2,
            4096,
            1024,
            &mut Vec::new(),
            &mut HashSet::new(),
        ),
        Err(SelectedBlobManifestPinDenial::ConflictingAttempt)
    );
}

#[test]
fn selected_v2_manifest_pins_exact_attempt_and_denies_future_slot() {
    let record = |ordinal| PersistedRecordIdentity::new([3; 16], ordinal).unwrap();
    let basis =
        FailedIngestReclaimBasisV1::new([4; 16], record(1), [5; 32], record(2), [6; 32]).unwrap();
    let manifest = DropSetManifestV2::new([1; 16], [2; 16], basis, vec![record(3)], 7)
        .unwrap()
        .encode();
    let mut pins = Vec::new();
    let mut attempts = HashSet::new();
    observe_manifest(
        &manifest,
        record(4),
        [1; 16],
        9,
        2,
        4096,
        1024,
        &mut pins,
        &mut attempts,
    )
    .unwrap();
    assert_eq!(pins.len(), 1);
    assert_eq!(pins[0].store(), [1; 16]);
    assert_eq!(pins[0].attempt(), [2; 16]);
    assert_eq!(
        observe_manifest(
            &manifest,
            record(4),
            [1; 16],
            6,
            2,
            4096,
            1024,
            &mut Vec::new(),
            &mut HashSet::new(),
        ),
        Err(SelectedBlobManifestPinDenial::MalformedManifest)
    );
}

#[test]
fn selected_v3_release_manifest_pins_completed_batch_across_checkpoint() {
    let record = |ordinal| PersistedRecordIdentity::new([3; 16], ordinal).unwrap();
    let publication = BlobGenerationPublicationV1::new(
        [1; 16],
        [4; 16],
        [5; 16],
        3,
        record(3),
        [6; 32],
        8,
        [7; 32],
        64 * 1024,
        [8; 32],
    )
    .unwrap();
    let basis = ReleasedGenerationReclaimBasisV1::new(
        publication,
        record(4),
        Sha256::digest(publication.encode()).into(),
        [9; 32],
    )
    .unwrap();
    let manifest = DropSetManifestV3::new(
        [1; 16],
        [2; 16],
        BlobReclaimSourceBasisV1::ReleasedGeneration(basis),
        vec![record(4), record(5)],
        7,
    )
    .unwrap()
    .encode();
    let mut pins = Vec::new();
    let mut attempts = HashSet::new();
    observe_manifest(
        &manifest,
        record(6),
        [1; 16],
        9,
        2,
        4096,
        1024,
        &mut pins,
        &mut attempts,
    )
    .unwrap();
    assert_eq!(pins.len(), 1);
    assert_eq!(pins[0].store(), [1; 16]);
    assert_eq!(pins[0].attempt(), [2; 16]);
    assert_eq!(attempts.len(), 1);
    assert_eq!(
        observe_manifest(
            &manifest,
            record(6),
            [1; 16],
            6,
            2,
            4096,
            1024,
            &mut Vec::new(),
            &mut HashSet::new(),
        ),
        Err(SelectedBlobManifestPinDenial::MalformedManifest)
    );
    let mut invalid_count = manifest;
    let count_offset = manifest_count_offset(&invalid_count).unwrap();
    invalid_count[count_offset..count_offset + 2].copy_from_slice(&u16::MAX.to_le_bytes());
    assert_eq!(
        observe_manifest(
            &invalid_count,
            record(6),
            [1; 16],
            9,
            2,
            4096,
            1024,
            &mut Vec::new(),
            &mut HashSet::new(),
        ),
        Err(SelectedBlobManifestPinDenial::MalformedManifest)
    );
}
