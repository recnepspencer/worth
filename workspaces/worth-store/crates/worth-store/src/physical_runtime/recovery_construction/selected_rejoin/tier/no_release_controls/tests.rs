use super::*;
use worth_store_physical_format::{
    FailedIngestReclaimBasisV1, OriginalDropReservationRequestV1, OriginalDropReservedV1,
};

fn record(ordinal: u64) -> PersistedRecordIdentity {
    PersistedRecordIdentity::new([4; 16], ordinal).unwrap()
}

#[test]
fn forged_or_mismatched_manifest_binding_cannot_prove_failed_ingest() {
    let store = [1; 16];
    let basis =
        FailedIngestReclaimBasisV1::new([2; 16], record(1), [3; 32], record(2), [4; 32]).unwrap();
    let manifest = Manifest {
        store,
        attempt: [5; 16],
        basis,
        digest: [6; 32],
        count: 1,
        selected_slot: Some(7),
    };
    let descriptor = Descriptor {
        record: record(4),
        frame_digest: [7; 32],
        store,
        attempt: manifest.attempt,
        basis_digest: basis.digest(store),
        manifest: record(3),
        manifest_digest: manifest.digest,
        count: manifest.count,
        source_generation: 8,
        candidate_generation: 9,
    };
    assert!(descriptor_matches_manifest(
        &descriptor,
        &manifest,
        store,
        9
    ));
    let forged_basis = Descriptor {
        basis_digest: [9; 32],
        ..descriptor
    };
    assert!(!descriptor_matches_manifest(
        &forged_basis,
        &manifest,
        store,
        9
    ));
    let forged_manifest = Descriptor {
        manifest_digest: [9; 32],
        ..descriptor
    };
    assert!(!descriptor_matches_manifest(
        &forged_manifest,
        &manifest,
        store,
        9
    ));
    let wrong_attempt = Descriptor {
        attempt: [9; 16],
        ..descriptor
    };
    assert!(!descriptor_matches_manifest(
        &wrong_attempt,
        &manifest,
        store,
        9
    ));
    assert!(!descriptor_matches_manifest(
        &descriptor,
        &manifest,
        store,
        8
    ));
}

#[test]
fn reservation_retargeted_to_another_selected_manifest_is_denied() {
    let store = [1; 16];
    let basis =
        FailedIngestReclaimBasisV1::new([2; 16], record(1), [3; 32], record(2), [4; 32]).unwrap();
    let first = Manifest {
        store,
        attempt: [5; 16],
        basis,
        digest: [6; 32],
        count: 1,
        selected_slot: Some(7),
    };
    let second = Manifest {
        store,
        attempt: [8; 16],
        basis,
        digest: [9; 32],
        count: 1,
        selected_slot: Some(7),
    };
    let first_attempt = first.attempt;
    let first_digest = first.digest;
    let manifests = vec![(record(3), first), (record(4), second)];
    let request = OriginalDropReservationRequestV1::new([5; 32], [6; 32], 8, 12).unwrap();
    let valid = OriginalDropReservedV1::new(
        store,
        first_attempt,
        record(3),
        first_digest,
        basis.digest(store),
        7,
        8,
        request,
    )
    .unwrap();
    assert!(verify_reservation(&manifests, valid, store, 8).is_ok());
    let retargeted = OriginalDropReservedV1::new(
        store,
        first_attempt,
        record(4),
        first_digest,
        basis.digest(store),
        7,
        8,
        request,
    )
    .unwrap();
    assert!(verify_reservation(&manifests, retargeted, store, 8).is_err());
}
