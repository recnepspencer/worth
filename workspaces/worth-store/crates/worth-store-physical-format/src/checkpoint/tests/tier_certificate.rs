use sha2::{Digest, Sha256};

use super::identity;
use crate::{TierEpochActivationV1, TierEpochCheckpointCertificateV1, TierEpochWalFrameWitnessV1};

fn sample() -> TierEpochCheckpointCertificateV1 {
    let intent = TierEpochActivationV1::intent(
        [7; 16], [2; 16], 3, [4; 32], [5; 32], 7, 4, [6; 32], 4096, 8,
    )
    .unwrap();
    let first =
        TierEpochWalFrameWitnessV1::new(10, 11, [1; 32], Sha256::digest(intent.encode()).into())
            .unwrap();
    let completed = TierEpochWalFrameWitnessV1::new(
        20,
        21,
        [3; 32],
        Sha256::digest(intent.completed().encode()).into(),
    )
    .unwrap();
    TierEpochCheckpointCertificateV1::new(
        identity(9),
        6,
        [8; 32],
        intent.epoch_anchor(),
        intent,
        first,
        completed,
    )
    .unwrap()
}

#[test]
fn tier_certificate_binds_checkpoint_root_anchor_and_selected_wal_members() {
    let selected = sample();
    assert!(selected.encode_in_reserved(Vec::new()).is_none());
    let mut reserved = Vec::with_capacity(crate::TIER_EPOCH_CHECKPOINT_CERTIFICATE_WIRE_BYTES);
    reserved.extend_from_slice(&[0xa5; 3]);
    let capacity = reserved.capacity();
    let pointer = reserved.as_ptr();
    let emitted = selected.encode_in_reserved(reserved).unwrap();
    assert_eq!(emitted, selected.encode());
    assert_eq!(emitted.capacity(), capacity);
    assert_eq!(emitted.as_ptr(), pointer);
    assert_eq!(
        TierEpochCheckpointCertificateV1::decode(&selected.encode()),
        Ok(selected)
    );
    assert!(TierEpochCheckpointCertificateV1::new(
        selected.checkpoint(),
        selected.root_generation(),
        [9; 32],
        selected.anchor(),
        selected.intent(),
        selected.intent_frame(),
        selected.completed_frame(),
    )
    .is_ok());
    assert!(TierEpochCheckpointCertificateV1::new(
        selected.checkpoint(),
        selected.root_generation(),
        selected.root_sha256(),
        [9; 32],
        selected.intent(),
        selected.intent_frame(),
        selected.completed_frame(),
    )
    .is_err());
    let mut corrupted = selected.encode();
    let last = corrupted.len() - 1;
    corrupted[last] ^= 1;
    assert!(TierEpochCheckpointCertificateV1::decode(&corrupted).is_err());
    assert!(TierEpochCheckpointCertificateV1::decode(&selected.encode()[..last]).is_err());
}

#[test]
fn immediate_candidate_checkpoint_requires_exact_candidate_root_digest() {
    let selected = sample();
    assert!(TierEpochCheckpointCertificateV1::new(
        selected.checkpoint(),
        selected.intent().candidate_root_generation(),
        selected.root_sha256(),
        selected.anchor(),
        selected.intent(),
        selected.intent_frame(),
        selected.completed_frame(),
    )
    .is_err());
    assert!(TierEpochCheckpointCertificateV1::new(
        selected.checkpoint(),
        selected.intent().candidate_root_generation(),
        selected.intent().candidate_root_sha256(),
        selected.anchor(),
        selected.intent(),
        selected.intent_frame(),
        selected.completed_frame(),
    )
    .is_ok());
}
