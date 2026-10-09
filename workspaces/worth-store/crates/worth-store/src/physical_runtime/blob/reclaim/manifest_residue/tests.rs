use super::*;
use worth_store_physical_format::{
    BlobReclaimDescriptorV1, FailedIngestReclaimBasisV1, OriginalDropReservationRequestV1,
};

fn record(ordinal: u64) -> PersistedRecordIdentity {
    PersistedRecordIdentity::new([7; 16], ordinal).unwrap()
}

fn manifest() -> DropSetManifestV1 {
    let basis =
        FailedIngestReclaimBasisV1::new([3; 16], record(1), [4; 32], record(2), [5; 32]).unwrap();
    DropSetManifestV1::new([1; 16], [2; 16], basis, vec![record(3)]).unwrap()
}

#[test]
fn only_manifest_metadata_can_remain_selected() {
    let manifest = manifest();
    assert!(selected_payload_present(record(3), &manifest));
    assert!(!selected_payload_present(record(4), &manifest));
}

#[test]
fn descriptor_referencing_manifest_is_denied_even_with_another_attempt() {
    let manifest = manifest();
    let descriptor = BlobReclaimDescriptorV1::new(
        [1; 16],
        [9; 16],
        manifest.source_basis_digest(),
        record(4),
        [8; 32],
        1,
        6,
        7,
    )
    .unwrap();
    assert!(conflicts_with_manifest(
        &BlobRecordV1::ReclaimDescriptor(descriptor),
        record(6),
        &manifest,
        record(4),
    ));
}

#[test]
fn completed_descriptor_ignore_requires_exact_manifest_binding() {
    let manifest = manifest();
    let sha256 = [8; 32];
    let exact = DescriptorLink {
        record: record(4),
        descriptor: BlobReclaimDescriptorV1::new(
            manifest.store(),
            manifest.reclaim_attempt(),
            manifest.source_basis_digest(),
            record(4),
            sha256,
            manifest.count(),
            6,
            7,
        )
        .unwrap(),
    };
    assert!(descriptor_matches_manifest(&exact, &manifest, sha256));
    assert!(!descriptor_matches_manifest(&exact, &manifest, [9; 32]));
}

#[test]
fn reserved_transition_must_name_the_exact_selected_manifest_and_slot() {
    let old = manifest();
    let manifest = DropSetManifestV2::new(
        old.store(),
        old.reclaim_attempt(),
        old.source_basis(),
        old.dropped().to_vec(),
        6,
    )
    .unwrap();
    let request = OriginalDropReservationRequestV1::new([3; 32], [4; 32], 6, 9).unwrap();
    let exact = OriginalDropReservedV1::new(
        manifest.store(),
        manifest.reclaim_attempt(),
        record(4),
        [8; 32],
        manifest.source_basis_digest(),
        6,
        7,
        request,
    )
    .unwrap();
    assert!(reservation_matches_manifest(
        exact,
        &manifest,
        record(4),
        [8; 32]
    ));
    assert!(!reservation_matches_manifest(
        exact,
        &manifest,
        record(4),
        [9; 32]
    ));
    assert!(!reservation_matches_manifest(
        exact,
        &manifest,
        record(5),
        [8; 32]
    ));
    let wrong_attempt = OriginalDropReservedV1::new(
        manifest.store(),
        [9; 16],
        record(4),
        [8; 32],
        manifest.source_basis_digest(),
        6,
        7,
        request,
    )
    .unwrap();
    assert!(!reservation_matches_manifest(
        wrong_attempt,
        &manifest,
        record(4),
        [8; 32]
    ));
}

#[test]
fn surviving_descriptor_cannot_point_at_reserved_metadata() {
    let descriptor =
        BlobReclaimDescriptorV1::new([1; 16], [2; 16], [3; 32], record(8), [4; 32], 1, 6, 7)
            .unwrap();
    assert!(conflicts_with_metadata_reference(
        &BlobRecordV1::ReclaimDescriptor(descriptor),
        record(8),
    ));
    assert!(!conflicts_with_metadata_reference(
        &BlobRecordV1::ReclaimDescriptor(descriptor),
        record(9),
    ));
}
