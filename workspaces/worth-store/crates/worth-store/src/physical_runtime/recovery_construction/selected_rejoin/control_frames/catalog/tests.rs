use std::num::NonZeroU64;

use worth_store_physical_format::store_namespace::{
    ProposedStoreIdentity, StoreNamespaceIdentityRecord, StoreNamespaceVersion,
};
use worth_store_physical_format::{
    release_checkpoint_batch_records_digest_v1, OriginalDropReservationRequestV1,
    PhysicalCheckpointIdentity, ReleaseCheckpointAccumulatorV1, ReleaseCheckpointCertificateV1,
    ReleasedDropWalFateWitnessV1,
};

use super::*;

fn checkpoint() -> PhysicalCheckpointIdentity {
    let store = StoreNamespaceIdentityRecord::new(
        StoreNamespaceVersion::CURRENT,
        ProposedStoreIdentity::from_nonzero_bytes([7; 16]).unwrap(),
    )
    .published_identity();
    PhysicalCheckpointIdentity::new(store, NonZeroU64::new(3).unwrap())
}

fn record(ordinal: u64) -> PersistedRecordIdentity {
    PersistedRecordIdentity::new([9; 16], ordinal).unwrap()
}

fn batch_for_claimed_count(claimed_count: u16) -> ReleaseCheckpointBatchV1 {
    let fate = ReleasedDropWalFateWitnessV1::new(100, 110, [7; 32], [8; 32]).unwrap();
    let cumulative = ReleasedDropCumulativeEvidenceV1::new(
        record(4),
        [2; 32],
        [3; 32],
        record(5),
        [4; 32],
        fate,
        12,
        [9; 32],
        None,
        claimed_count,
        false,
    )
    .unwrap()
    .advance(0, [0; 32])
    .unwrap();
    ReleaseCheckpointBatchV1::new(
        checkpoint(),
        15,
        [1; 32],
        0,
        record(4),
        [2; 32],
        [3; 32],
        record(5),
        [4; 32],
        OriginalDropReservationRequestV1::new([5; 32], [6; 32], 10, 20).unwrap(),
        fate,
        12,
        [9; 32],
        None,
        cumulative.0,
        cumulative.1,
        false,
    )
    .unwrap()
}

#[test]
fn coherently_encoded_wrong_cumulative_claim_denied_against_observed_manifest() {
    let forged = batch_for_claimed_count(1);
    let accumulator = ReleaseCheckpointAccumulatorV1::new(
        checkpoint(),
        15,
        [1; 32],
        0,
        [0; 32],
        [0; 32],
        0,
        [0; 32],
        1,
        release_checkpoint_batch_records_digest_v1(&[forged]).unwrap(),
        forged.tip_provenance().unwrap(),
        forged.cumulative_dropped(),
        forged.cumulative_digest(),
        false,
    )
    .unwrap();
    assert_eq!(
        ReleaseCheckpointCertificateV1::decode(
            &ReleaseCheckpointCertificateV1::Batch(forged).encode()
        ),
        Ok(ReleaseCheckpointCertificateV1::Batch(forged)),
    );
    assert_eq!(
        ReleaseCheckpointCertificateV1::decode(
            &ReleaseCheckpointCertificateV1::Accumulator(accumulator).encode()
        ),
        Ok(ReleaseCheckpointCertificateV1::Accumulator(accumulator)),
    );
    assert_eq!(
        (forged.cumulative_dropped(), forged.cumulative_digest()),
        (
            accumulator.cumulative_dropped(),
            accumulator.cumulative_digest()
        ),
    );
    assert!(fold_observed_batch(forged, 2, (0, [0; 32])).is_err());
    assert!(fold_observed_batch(batch_for_claimed_count(2), 2, (0, [0; 32])).is_ok());
}

#[test]
fn newer_uncertified_selected_v3_cannot_be_labeled_old_residue() {
    assert!(uncovered_descriptor_predates_checkpoint(15, 15));
    assert!(uncovered_descriptor_predates_checkpoint(14, 15));
    assert!(!uncovered_descriptor_predates_checkpoint(16, 15));
}
