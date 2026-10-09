use super::{CheckpointCustodyOrigin, CheckpointCustodyState, SealedTierEpochCustodyBasis};
use sha2::{Digest, Sha256};
use std::num::NonZeroU64;
use worth_store_physical_format::store_namespace::{
    ProposedStoreIdentity, StoreNamespaceIdentityRecord, StoreNamespaceVersion,
};
use worth_store_physical_format::{
    DurablePhysicalRootManifest, PhysicalCheckpointIdentity, TierEpochActivationV1,
    TierEpochCheckpointCertificateV1, TierEpochWalFrameWitnessV1,
};

#[test]
fn only_trusted_fresh_genesis_installs_legacy_empty_custody() {
    let genesis = DurablePhysicalRootManifest::builder(1, 1, 2, 1)
        .admit()
        .unwrap();
    let later = DurablePhysicalRootManifest::builder(2, 1, 2, 1)
        .admit()
        .unwrap();
    assert!(matches!(
        CheckpointCustodyState::from_origin(CheckpointCustodyOrigin::FreshGenesis, &genesis),
        CheckpointCustodyState::VerifiedLegacyNoCertificates
    ));
    assert!(matches!(
        CheckpointCustodyState::from_origin(CheckpointCustodyOrigin::ReopenRequiresC8, &genesis),
        CheckpointCustodyState::Unavailable
    ));
    assert!(matches!(
        CheckpointCustodyState::from_origin(CheckpointCustodyOrigin::FreshGenesis, &later),
        CheckpointCustodyState::Unavailable
    ));
}

#[test]
fn tier_basis_is_rebound_to_exact_checkpoint_identity_and_root() {
    let intent = TierEpochActivationV1::intent(
        [7; 16], [2; 16], 3, [4; 32], [5; 32], 7, 4, [6; 32], 4096, 8,
    )
    .unwrap();
    let basis = SealedTierEpochCustodyBasis {
        intent,
        intent_frame: TierEpochWalFrameWitnessV1::new(
            10,
            11,
            [1; 32],
            Sha256::digest(intent.encode()).into(),
        )
        .unwrap(),
        completed_frame: TierEpochWalFrameWitnessV1::new(
            20,
            21,
            [3; 32],
            Sha256::digest(intent.completed().encode()).into(),
        )
        .unwrap(),
    };
    let root = DurablePhysicalRootManifest::builder(5, 1, 2, 1)
        .tier_epoch_anchor(Some(intent.epoch_anchor()))
        .admit()
        .unwrap();
    let store = StoreNamespaceIdentityRecord::new(
        StoreNamespaceVersion::CURRENT,
        ProposedStoreIdentity::from_nonzero_bytes([7; 16]).unwrap(),
    )
    .published_identity();
    let first = PhysicalCheckpointIdentity::new(store, NonZeroU64::new(1).unwrap());
    let second = PhysicalCheckpointIdentity::new(store, NonZeroU64::new(2).unwrap());
    let first_payload = basis.bind(first, &root, [8; 32]).unwrap().encode();
    let second_payload = basis.bind(second, &root, [8; 32]).unwrap().encode();
    assert_ne!(first_payload, second_payload);
    assert_eq!(
        TierEpochCheckpointCertificateV1::decode(&first_payload)
            .unwrap()
            .checkpoint(),
        first
    );
    assert!(basis.bind(first, &root, [0; 32]).is_err());
    let mut state = CheckpointCustodyState::CertifiedTier(basis);
    assert!(state.require_release_certificate([1; 16]));
    assert!(state.require_release_certificate([1; 16]));
    assert!(!state.require_release_certificate([2; 16]));
    assert!(matches!(
        &state,
        CheckpointCustodyState::ReleaseCertificatePending { attempt, prior }
            if *attempt == [1; 16]
                && matches!(prior.as_ref(), CheckpointCustodyState::CertifiedTier(_))
    ));
    assert!(!state.restore_proven_no_effect([2; 16]));
    assert!(state.restore_proven_no_effect([1; 16]));
    assert!(matches!(state, CheckpointCustodyState::CertifiedTier(_)));
}
