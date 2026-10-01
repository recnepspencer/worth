use super::reservation::{reservation_blocks_fresh_attempt, reservation_matches_selected_manifest};
use worth_store_physical_format::{
    DropSetManifestV2, FailedIngestReclaimBasisV1, OriginalDropReservationRequestV1,
    OriginalDropReservedV1, PersistedRecordIdentity,
};

#[test]
fn selected_exact_reservation_blocks_new_reclaim_attempt() {
    let request = OriginalDropReservationRequestV1::new([1; 32], [2; 32], 3, 4).unwrap();
    let reserved = OriginalDropReservedV1::new(
        [5; 16],
        [6; 16],
        PersistedRecordIdentity::new([7; 16], 8).unwrap(),
        [9; 32],
        [10; 32],
        11,
        12,
        request,
    )
    .unwrap();
    assert!(reservation_blocks_fresh_attempt(
        reserved, [5; 16], [10; 32]
    ));
    assert!(!reservation_blocks_fresh_attempt(
        reserved, [4; 16], [10; 32]
    ));
    assert!(!reservation_blocks_fresh_attempt(
        reserved, [5; 16], [11; 32]
    ));
}

#[test]
fn terminal_retry_requires_exact_reserved_manifest_basis_and_sha() {
    let declaration = PersistedRecordIdentity::new([7; 16], 1).unwrap();
    let abandoned = PersistedRecordIdentity::new([7; 16], 2).unwrap();
    let manifest_record = PersistedRecordIdentity::new([7; 16], 3).unwrap();
    let basis =
        FailedIngestReclaimBasisV1::new([3; 16], declaration, [4; 32], abandoned, [5; 32]).unwrap();
    let payload = PersistedRecordIdentity::new([7; 16], 4).unwrap();
    let manifest = DropSetManifestV2::new([5; 16], [6; 16], basis, vec![payload], 11).unwrap();
    let request = OriginalDropReservationRequestV1::new([1; 32], [2; 32], 3, 4).unwrap();
    let reserved = OriginalDropReservedV1::new(
        [5; 16],
        [6; 16],
        manifest_record,
        [9; 32],
        manifest.source_basis_digest(),
        11,
        12,
        request,
    )
    .unwrap();
    assert!(reservation_matches_selected_manifest(
        reserved,
        &manifest,
        manifest_record,
        [9; 32],
        basis,
    ));
    assert!(!reservation_matches_selected_manifest(
        reserved,
        &manifest,
        manifest_record,
        [8; 32],
        basis,
    ));
    assert!(!reservation_matches_selected_manifest(
        reserved,
        &manifest,
        manifest_record,
        [9; 32],
        FailedIngestReclaimBasisV1::new([4; 16], declaration, [4; 32], abandoned, [5; 32]).unwrap(),
    ));
}
