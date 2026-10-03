use std::num::NonZeroU64;

use sha2::Digest;

use crate::store_namespace::{ProposedStoreIdentity, StableStoreIdentity};
use crate::{OriginalDropReservationRequestV1, PersistedRecordIdentity, ReleasedDropPredecessorV1};

use super::{
    release_checkpoint_batch_records_digest_v1, PhysicalCheckpointIdentity,
    ReleaseCheckpointAccumulatorV1, ReleaseCheckpointAccumulatorV2, ReleaseCheckpointBatchV1,
    ReleaseCheckpointCertificateV1, ReleaseCheckpointNoReleaseV1, ReleasedDropCumulativeEvidenceV1,
    ReleasedDropTipProvenanceV1, ReleasedDropWalFateWitnessV1,
    RELEASE_CHECKPOINT_ACCUMULATOR_V2_WIRE_BYTES, RELEASE_CHECKPOINT_ACCUMULATOR_WIRE_BYTES,
    RELEASE_CHECKPOINT_BATCH_WIRE_BYTES, RELEASE_CHECKPOINT_NO_RELEASE_WIRE_BYTES,
};

#[test]
fn no_release_genesis_and_successor_are_exact_versioned_roundtrips() {
    let genesis =
        ReleaseCheckpointNoReleaseV1::new(checkpoint(1), 4, [1; 32], 0, [0; 32], [0; 32]).unwrap();
    let genesis_bytes = ReleaseCheckpointCertificateV1::NoRelease(genesis).encode();
    assert!(genesis.encode_in_reserved(Vec::new()).is_none());
    let mut reserved = Vec::with_capacity(RELEASE_CHECKPOINT_NO_RELEASE_WIRE_BYTES);
    reserved.extend_from_slice(&[0xa5; 3]);
    let capacity = reserved.capacity();
    let emitted = genesis.encode_in_reserved(reserved).unwrap();
    assert_eq!(emitted, genesis_bytes);
    assert_eq!(emitted.capacity(), capacity);
    assert_eq!(
        genesis_bytes.len(),
        RELEASE_CHECKPOINT_NO_RELEASE_WIRE_BYTES
    );
    assert_eq!(
        ReleaseCheckpointCertificateV1::decode(&genesis_bytes),
        Ok(ReleaseCheckpointCertificateV1::NoRelease(genesis))
    );
    let predecessor_digest: [u8; 32] = sha2::Sha256::digest(&genesis_bytes).into();
    let successor = ReleaseCheckpointNoReleaseV1::new(
        checkpoint(2),
        6,
        [2; 32],
        1,
        [1; 32],
        predecessor_digest,
    )
    .unwrap();
    assert_eq!(
        ReleaseCheckpointCertificateV1::decode(&successor.encode()),
        Ok(ReleaseCheckpointCertificateV1::NoRelease(successor))
    );
    assert!(ReleaseCheckpointNoReleaseV1::new(
        checkpoint(2),
        6,
        [2; 32],
        1,
        [0; 32],
        predecessor_digest,
    )
    .is_err());
    assert!(ReleaseCheckpointNoReleaseV1::new(
        checkpoint(2),
        6,
        [2; 32],
        2,
        [1; 32],
        predecessor_digest,
    )
    .is_err());
    let mut truncated = successor.encode();
    truncated.pop();
    assert!(ReleaseCheckpointCertificateV1::decode(&truncated).is_err());
}

fn checkpoint(sequence: u64) -> PhysicalCheckpointIdentity {
    let store = StableStoreIdentity::from_published_record(
        ProposedStoreIdentity::from_nonzero_bytes([7; 16]).unwrap(),
    );
    PhysicalCheckpointIdentity::new(store, NonZeroU64::new(sequence).unwrap())
}

fn record(ordinal: u64) -> PersistedRecordIdentity {
    PersistedRecordIdentity::new([9; 16], ordinal).unwrap()
}

fn batch() -> ReleaseCheckpointBatchV1 {
    ReleaseCheckpointBatchV1::new(
        checkpoint(3),
        15,
        [1; 32],
        0,
        record(4),
        [2; 32],
        [3; 32],
        record(5),
        [4; 32],
        OriginalDropReservationRequestV1::new([5; 32], [6; 32], 10, 20).unwrap(),
        ReleasedDropWalFateWitnessV1::new(100, 110, [7; 32], [8; 32]).unwrap(),
        12,
        [9; 32],
        None,
        2,
        [10; 32],
        false,
    )
    .unwrap()
}

fn tip() -> ReleasedDropTipProvenanceV1 {
    batch().tip_provenance().unwrap()
}

#[test]
fn batch_and_accumulator_are_exact_bounded_roundtrips() {
    let batch = batch();
    let encoded_batch = ReleaseCheckpointCertificateV1::Batch(batch).encode();
    assert!(batch.encode_in_reserved(Vec::new()).is_none());
    let mut reserved = Vec::with_capacity(RELEASE_CHECKPOINT_BATCH_WIRE_BYTES);
    reserved.extend_from_slice(&[0xa5; 3]);
    let capacity = reserved.capacity();
    let pointer = reserved.as_ptr();
    let emitted = batch.encode_in_reserved(reserved).unwrap();
    assert_eq!(emitted, encoded_batch);
    assert_eq!(emitted.as_slice(), batch.encode_fixed());
    assert_eq!(emitted.capacity(), capacity);
    assert_eq!(emitted.as_ptr(), pointer);
    assert_eq!(encoded_batch.len(), RELEASE_CHECKPOINT_BATCH_WIRE_BYTES);
    assert_eq!(
        ReleaseCheckpointCertificateV1::decode(&encoded_batch),
        Ok(ReleaseCheckpointCertificateV1::Batch(batch))
    );
    let accumulator = ReleaseCheckpointAccumulatorV1::new(
        checkpoint(3),
        15,
        [1; 32],
        0,
        [0; 32],
        [0; 32],
        0,
        [0; 32],
        1,
        [11; 32],
        tip(),
        2,
        [10; 32],
        false,
    )
    .unwrap();
    let encoded_accumulator = ReleaseCheckpointCertificateV1::Accumulator(accumulator).encode();
    assert_eq!(
        encoded_accumulator.len(),
        RELEASE_CHECKPOINT_ACCUMULATOR_WIRE_BYTES
    );
    assert_eq!(
        ReleaseCheckpointCertificateV1::decode(&encoded_accumulator),
        Ok(ReleaseCheckpointCertificateV1::Accumulator(accumulator))
    );
    let carried = ReleaseCheckpointAccumulatorV1::new(
        checkpoint(4),
        17,
        [12; 32],
        3,
        [1; 32],
        [13; 32],
        2,
        [10; 32],
        0,
        [0; 32],
        tip(),
        2,
        [10; 32],
        false,
    )
    .unwrap();
    assert_eq!(
        ReleaseCheckpointCertificateV1::decode(&carried.encode()),
        Ok(ReleaseCheckpointCertificateV1::Accumulator(carried))
    );
    assert_eq!(carried.tip(), batch.tip_provenance().unwrap());
    assert_eq!(
        carried.tip().reservation_record(),
        batch.reservation_record()
    );
    assert_eq!(carried.tip().request(), batch.request());
    assert_eq!(carried.tip().fate(), batch.fate());
    assert_eq!(
        carried.tip().candidate_root_sha256(),
        batch.candidate_root_sha256()
    );
}

#[test]
fn v2_accumulator_requires_a_source_roster_and_an_exact_prior_ratchet() {
    let base = ReleaseCheckpointAccumulatorV1::new(
        checkpoint(3),
        15,
        [1; 32],
        0,
        [0; 32],
        [0; 32],
        0,
        [0; 32],
        1,
        [11; 32],
        tip(),
        2,
        [10; 32],
        false,
    )
    .unwrap();
    let current = ReleaseCheckpointAccumulatorV2::new(base, 1, [21; 32], 0, [0; 32]).unwrap();
    let encoded = ReleaseCheckpointCertificateV1::AccumulatorV2(current).encode();
    assert!(current.encode_in_reserved(Vec::new()).is_none());
    let mut reserved = Vec::with_capacity(RELEASE_CHECKPOINT_ACCUMULATOR_V2_WIRE_BYTES);
    reserved.extend_from_slice(&[0xa5; 3]);
    let capacity = reserved.capacity();
    let emitted = current.encode_in_reserved(reserved).unwrap();
    assert_eq!(emitted, encoded);
    assert_eq!(emitted.capacity(), capacity);
    assert_eq!(encoded.len(), RELEASE_CHECKPOINT_ACCUMULATOR_V2_WIRE_BYTES);
    assert_eq!(
        ReleaseCheckpointCertificateV1::decode(&encoded),
        Ok(ReleaseCheckpointCertificateV1::AccumulatorV2(current)),
    );
    assert!(ReleaseCheckpointAccumulatorV2::new(base, 0, [21; 32], 0, [0; 32]).is_ok());
    assert!(ReleaseCheckpointAccumulatorV2::new(base, 1, [0; 32], 0, [0; 32]).is_err());
    assert!(ReleaseCheckpointAccumulatorV2::new(base, 1, [21; 32], 1, [22; 32]).is_err());
    let carried = ReleaseCheckpointAccumulatorV1::new(
        checkpoint(4),
        17,
        [12; 32],
        3,
        [1; 32],
        [13; 32],
        2,
        [10; 32],
        0,
        [0; 32],
        tip(),
        2,
        [10; 32],
        false,
    )
    .unwrap();
    assert!(ReleaseCheckpointAccumulatorV2::new(carried, 0, [23; 32], 0, [21; 32]).is_ok());
    assert!(ReleaseCheckpointAccumulatorV2::new(carried, 0, [23; 32], 0, [0; 32]).is_err());
    let mut transplanted = encoded;
    transplanted[8 + super::DOMAIN.len()] = 1;
    assert!(ReleaseCheckpointCertificateV1::decode(&transplanted).is_err());
}

#[test]
fn malformed_transplants_and_ambiguous_chain_claims_are_denied() {
    let encoded = batch().encode();
    assert!(ReleaseCheckpointCertificateV1::decode(&encoded[..encoded.len() - 1]).is_err());
    let mut trailing = encoded.clone();
    trailing.push(0);
    assert!(ReleaseCheckpointCertificateV1::decode(&trailing).is_err());
    let mut version = encoded.clone();
    version[8 + super::DOMAIN.len()] = 2;
    assert!(ReleaseCheckpointCertificateV1::decode(&version).is_err());
    let mut root = encoded.clone();
    root[super::PREFIX_BYTES + 24 + 8..super::PREFIX_BYTES + 24 + 8 + 32].fill(0);
    assert!(ReleaseCheckpointCertificateV1::decode(&root).is_err());
    let prior = ReleasedDropPredecessorV1::new(record(4), [2; 32]).unwrap();
    assert!(ReleaseCheckpointBatchV1::new(
        checkpoint(3),
        15,
        [1; 32],
        0,
        record(4),
        [2; 32],
        [3; 32],
        record(5),
        [4; 32],
        OriginalDropReservationRequestV1::new([5; 32], [6; 32], 10, 20).unwrap(),
        ReleasedDropWalFateWitnessV1::new(100, 110, [7; 32], [8; 32]).unwrap(),
        12,
        [9; 32],
        Some(prior),
        2,
        [10; 32],
        false,
    )
    .is_err());
    assert!(ReleaseCheckpointAccumulatorV1::new(
        checkpoint(3),
        15,
        [1; 32],
        3,
        [2; 32],
        [3; 32],
        2,
        [6; 32],
        0,
        [0; 32],
        tip(),
        2,
        [5; 32],
        false,
    )
    .is_err());
    let valid_tip = tip();
    assert!(ReleasedDropTipProvenanceV1::new(
        valid_tip.descriptor_record(),
        valid_tip.descriptor_frame_sha256(),
        valid_tip.descriptor_record(),
        valid_tip.reservation_frame_sha256(),
        valid_tip.request(),
        valid_tip.fate(),
        valid_tip.candidate_root_generation(),
        valid_tip.candidate_root_sha256(),
    )
    .is_err());
    assert!(ReleasedDropTipProvenanceV1::new(
        valid_tip.descriptor_record(),
        valid_tip.descriptor_frame_sha256(),
        valid_tip.reservation_record(),
        valid_tip.reservation_frame_sha256(),
        valid_tip.request(),
        valid_tip.fate(),
        0,
        valid_tip.candidate_root_sha256(),
    )
    .is_err());
}

#[test]
fn cumulative_step_is_checked_and_sensitive_to_selected_fate() {
    let evidence = |fate_sha| {
        ReleasedDropCumulativeEvidenceV1::new(
            record(4),
            [2; 32],
            [3; 32],
            record(5),
            [4; 32],
            ReleasedDropWalFateWitnessV1::new(100, 110, [7; 32], fate_sha).unwrap(),
            12,
            [9; 32],
            None,
            2,
            false,
        )
        .unwrap()
    };
    let (count, digest) = evidence([8; 32]).advance(0, [0; 32]).unwrap();
    assert_eq!(count, 2);
    assert_ne!(digest, [0; 32]);
    assert_ne!(digest, evidence([6; 32]).advance(0, [0; 32]).unwrap().1);
    let (next_count, next_digest) = evidence([8; 32]).advance(count, digest).unwrap();
    assert_eq!(next_count, 4);
    assert_ne!(next_digest, digest);
    assert!(evidence([8; 32]).advance(1, [0; 32]).is_err());
    assert!(evidence([8; 32]).advance(0, [1; 32]).is_err());
}

#[test]
fn batch_roster_digest_requires_order_and_one_checkpoint_root() {
    let first = batch();
    let digest = release_checkpoint_batch_records_digest_v1(&[first]).unwrap();
    assert_ne!(digest, [0; 32]);
    assert!(release_checkpoint_batch_records_digest_v1(&[]).is_err());
    let mut other_root = batch();
    other_root = ReleaseCheckpointBatchV1::new(
        checkpoint(4),
        15,
        [1; 32],
        0,
        other_root.descriptor_record(),
        other_root.descriptor_frame_sha256(),
        other_root.custody_digest(),
        other_root.reservation_record(),
        other_root.reservation_frame_sha256(),
        other_root.request(),
        other_root.fate(),
        other_root.candidate_root_generation(),
        other_root.candidate_root_sha256(),
        other_root.predecessor(),
        other_root.cumulative_dropped(),
        other_root.cumulative_digest(),
        other_root.terminal(),
    )
    .unwrap();
    assert_ne!(
        digest,
        release_checkpoint_batch_records_digest_v1(&[other_root]).unwrap()
    );
    assert!(release_checkpoint_batch_records_digest_v1(&[first, other_root]).is_err());
}
