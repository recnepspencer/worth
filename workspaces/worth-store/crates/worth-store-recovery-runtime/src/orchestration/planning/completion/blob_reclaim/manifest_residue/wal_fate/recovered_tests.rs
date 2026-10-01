use super::*;
use worth_store_physical_format::{
    BlobManifestResidueCleanupV2, BlobReclaimDescriptorV1, OriginalDropReservationRequestV1,
    OriginalDropReservedV1, PersistedRecordIdentity, ReservedDropRecordV1,
};
use worth_store_recovery_physics::{
    reconcile_operation_fates, RecoveryBindingFreshness, RecoveryOperationEvidenceInput,
    RecoveryOperationIdentity,
};

fn recovered_intent() -> BlobManifestResidueCleanup {
    let reserved =
        ReservedDropRecordV1::new(PersistedRecordIdentity::new([3; 16], 5).unwrap(), [14; 32])
            .unwrap();
    BlobManifestResidueCleanupV2::intent(
        [1; 16],
        [2; 16],
        PersistedRecordIdentity::new([3; 16], 4).unwrap(),
        [5; 32],
        [6; 32],
        OriginalDropProofV1::RecoveredNoBinding {
            idempotency: expected_drop_key([1; 16], [2; 16], [7; 32], 1, 10),
            fingerprint: [8; 32],
            reserved,
        },
        9,
        10,
        [11; 32],
        12,
        13,
    )
    .unwrap()
    .into()
}

fn observed(
    key: [u8; 32],
    lease: (u64, u64),
    fate: RecoveryOperationFate,
) -> ReconciledOperationFates {
    let identity = RecoveryOperationIdentity::new([1; 16], 1, 1, 1, key).unwrap();
    reconcile_operation_fates(
        2,
        vec![RecoveryOperationEvidenceInput::new(
            identity,
            [8; 32],
            lease.0,
            lease.1,
            RecoveryBindingFreshness::Retained,
            fate,
        )],
        1,
    )
    .unwrap()
}

#[test]
fn recovered_reservation_source_requires_exact_rederived_lease_key() {
    let intent = recovered_intent();
    let request = OriginalDropReservationRequestV1::new(
        expected_drop_key([1; 16], [2; 16], [7; 32], 1, 10),
        [8; 32],
        1,
        10,
    )
    .unwrap();
    let reservation = OriginalDropReservedV1::new(
        [1; 16],
        [2; 16],
        intent.manifest_record(),
        [5; 32],
        [6; 32],
        8,
        9,
        request,
    )
    .unwrap();
    assert!(recovered_reservation_key_matches(
        intent,
        [7; 32],
        reservation
    ));
    assert!(!recovered_reservation_key_matches(
        intent,
        [9; 32],
        reservation
    ));
    let wrong_lease =
        OriginalDropReservationRequestV1::new(request.idempotency(), request.fingerprint(), 2, 10)
            .unwrap();
    let wrong = OriginalDropReservedV1::new(
        [1; 16],
        [2; 16],
        intent.manifest_record(),
        [5; 32],
        [6; 32],
        8,
        9,
        wrong_lease,
    )
    .unwrap();
    assert!(!recovered_reservation_key_matches(intent, [7; 32], wrong));
}

#[test]
fn recovered_post_selector_absence_denies_all_same_material_fates() {
    let intent = recovered_intent();
    let exact_key = match intent.proof() {
        OriginalDropProofV1::RecoveredNoBinding { idempotency, .. } => idempotency,
        _ => unreachable!(),
    };
    for fate in [
        RecoveryOperationFate::AcknowledgedDurable,
        RecoveryOperationFate::DurableUnacknowledged,
        RecoveryOperationFate::Indeterminate,
        RecoveryOperationFate::ProvenNoEffect,
    ] {
        let evidence = observed(exact_key, (1, 10), fate);
        assert!(is_recovered_drop_material(
            intent,
            [7; 32],
            exact_key,
            &evidence.operations()[0],
        ));
    }
    let empty = reconcile_operation_fates(2, vec![], 1).unwrap();
    assert!(empty.operations().is_empty());
    let alternate_key = expected_drop_key([1; 16], [2; 16], [7; 32], 2, 11);
    let alternate = observed(alternate_key, (2, 11), RecoveryOperationFate::Indeterminate);
    assert!(
        is_recovered_drop_material(intent, [7; 32], exact_key, &alternate.operations()[0],),
        "a different lease of the same original attempt also contradicts no binding"
    );
    let unrelated = observed([19; 32], (2, 11), RecoveryOperationFate::Indeterminate);
    assert!(!is_recovered_drop_material(
        intent,
        [7; 32],
        exact_key,
        &unrelated.operations()[0],
    ));
}

#[test]
fn recovered_cleanup_rejects_matching_redo_descriptor_even_without_a_binding() {
    let intent = recovered_intent();
    let matching = BlobReclaimDescriptorV1::new(
        intent.store(),
        intent.reclaim_attempt(),
        intent.source_basis_digest(),
        intent.manifest_record(),
        intent.manifest_frame_sha256(),
        1,
        8,
        9,
    )
    .unwrap();
    assert!(conflicting_recovered_redo_descriptor(
        intent,
        &matching.encode()
    ));
    let other = BlobReclaimDescriptorV1::new(
        intent.store(),
        [18; 16],
        intent.source_basis_digest(),
        PersistedRecordIdentity::new([3; 16], 99).unwrap(),
        intent.manifest_frame_sha256(),
        1,
        8,
        9,
    )
    .unwrap();
    assert!(!conflicting_recovered_redo_descriptor(
        intent,
        &other.encode()
    ));
}
