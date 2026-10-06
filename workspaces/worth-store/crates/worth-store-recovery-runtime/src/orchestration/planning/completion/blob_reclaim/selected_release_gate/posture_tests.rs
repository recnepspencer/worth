//! Shape rejection at the active gate, not acceptance of obsolete custody.
//! These unit records exercise classification only; none is release authority.

use std::num::NonZeroU64;
use worth_store_physical_format::{
    encode_checkpoint_certificate, release_checkpoint_batch_records_digest_v1,
    store_namespace::{ProposedStoreIdentity, StoreNamespaceIdentityRecord, StoreNamespaceVersion},
    CheckpointCertificateKind, OriginalDropReservationRequestV1, PersistedRecordIdentity,
    PhysicalCheckpointIdentity, ReleaseCheckpointAccumulatorV1, ReleaseCheckpointAccumulatorV2,
    ReleaseCheckpointBatchV1, ReleaseCheckpointCertificateV1 as Certificate,
    ReleaseCheckpointNoReleaseV1, ReleasedDropWalFateWitnessV1,
};

use super::{
    absent_posture_admits, checkpoint_records_posture as classify,
    CheckpointReleasePosture as Posture,
};
use crate::progression::PlanningCustody;

#[test]
fn absent_release_posture_denies_any_selected_release() {
    // A routed V2 released descriptor, or a checkpoint without release
    // certificates, reaches this posture with a selected release to deny.
    assert!(!absent_posture_admits(true, &PlanningCustody::Unresolved));
    assert!(absent_posture_admits(false, &PlanningCustody::Unresolved));
    assert!(!absent_posture_admits(
        false,
        &PlanningCustody::NoCheckpoint
    ));
}

#[test]
fn positive_no_release_is_exclusive_and_malformed_frames_are_rejected() {
    let marker = frame(Certificate::NoRelease(
        ReleaseCheckpointNoReleaseV1::new(checkpoint(), 12, [3; 32], 0, [0; 32], [0; 32]).unwrap(),
    ));
    assert_eq!(classify(&[]), Ok(Posture::Absent));
    assert_eq!(classify(&[marker.clone()]), Ok(Posture::NoRelease));
    assert_eq!(classify(&[marker.clone(), marker.clone()]), Err(()));
    let (batch, accumulator) = released_shape();
    assert_eq!(
        classify(&[marker.clone(), frame(Certificate::Batch(batch))]),
        Err(())
    );
    assert_eq!(
        classify(&[frame(Certificate::Batch(batch)), marker.clone()]),
        Err(())
    );
    let head = frame(Certificate::AccumulatorV2(
        ReleaseCheckpointAccumulatorV2::new(accumulator, 1, [13; 32], 0, [0; 32]).unwrap(),
    ));
    assert_eq!(classify(&[marker.clone(), head.clone()]), Err(()));
    assert_eq!(classify(&[head, marker.clone()]), Err(()));
    let mut corrupt = marker.into_vec();
    corrupt[0] ^= 1;
    assert_eq!(classify(&[corrupt.into_boxed_slice()]), Err(()));
}

#[test]
fn current_heads_are_required_and_duplicate_or_reordered_accumulators_deny() {
    let (batch, accumulator) = released_shape();
    let batch = frame(Certificate::Batch(batch));
    let headless = frame(Certificate::Accumulator(accumulator));
    assert_eq!(classify(&[headless.clone()]), Err(()));
    assert_eq!(classify(&[batch.clone(), headless]), Err(()));
    assert_eq!(classify(&[batch.clone()]), Err(()));
    let head = frame(Certificate::AccumulatorV2(
        ReleaseCheckpointAccumulatorV2::new(accumulator, 1, [13; 32], 0, [0; 32]).unwrap(),
    ));
    assert_eq!(
        classify(&[batch.clone(), head.clone()]),
        Ok(Posture::HeadV2)
    );
    assert_eq!(classify(&[head.clone(), head.clone()]), Err(()));
    assert_eq!(classify(&[head, batch]), Err(()));
}

fn checkpoint() -> PhysicalCheckpointIdentity {
    let store = StoreNamespaceIdentityRecord::new(
        StoreNamespaceVersion::CURRENT,
        ProposedStoreIdentity::from_nonzero_bytes([1; 16]).unwrap(),
    )
    .published_identity();
    PhysicalCheckpointIdentity::new(store, NonZeroU64::new(3).unwrap())
}

fn frame(certificate: Certificate) -> Box<[u8]> {
    encode_checkpoint_certificate(
        CheckpointCertificateKind::ReleasedDrop,
        &certificate.encode(),
    )
    .unwrap()
    .into_boxed_slice()
}

fn released_shape() -> (ReleaseCheckpointBatchV1, ReleaseCheckpointAccumulatorV1) {
    let checkpoint = checkpoint();
    let record = |ordinal| PersistedRecordIdentity::new([2; 16], ordinal).unwrap();
    let batch = ReleaseCheckpointBatchV1::new(
        checkpoint,
        12,
        [3; 32],
        0,
        record(1),
        [4; 32],
        [5; 32],
        record(2),
        [6; 32],
        OriginalDropReservationRequestV1::new([7; 32], [8; 32], 1, 2).unwrap(),
        ReleasedDropWalFateWitnessV1::new(10, 20, [9; 32], [10; 32]).unwrap(),
        11,
        [11; 32],
        None,
        1,
        [12; 32],
        true,
    )
    .unwrap();
    let accumulator = ReleaseCheckpointAccumulatorV1::new(
        checkpoint,
        12,
        [3; 32],
        0,
        [0; 32],
        [0; 32],
        0,
        [0; 32],
        1,
        release_checkpoint_batch_records_digest_v1(&[batch]).unwrap(),
        batch.tip_provenance().unwrap(),
        1,
        [12; 32],
        true,
    )
    .unwrap();
    (batch, accumulator)
}
