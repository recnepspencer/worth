use super::*;
use crate::physical_runtime::PhysicalRecoveryAllocationAdmission;
use worth_store_physical_format::store_namespace::{
    ProposedStoreIdentity, StoreNamespaceIdentityRecord, StoreNamespaceVersion,
};
use worth_store_wal::{WalSegmentGeneration, WalSegmentId};

fn identity(segment: u64, generation: u64) -> WalSegmentArtifactIdentity {
    WalSegmentArtifactIdentity::new(
        WalSegmentId::new(segment).unwrap(),
        WalSegmentGeneration::new(generation).unwrap(),
    )
}

fn entry(segment: u64, generation: u64) -> WalArtifactFingerprint {
    WalArtifactFingerprint {
        identity: identity(segment, generation),
        length: segment,
        sha256: [segment as u8; 32],
    }
}

fn admission(bytes: u64) -> PhysicalRecoveryAllocationAdmission {
    let store = StoreNamespaceIdentityRecord::new(
        StoreNamespaceVersion::CURRENT,
        ProposedStoreIdentity::from_nonzero_bytes([1; 16]).unwrap(),
    )
    .published_identity();
    PhysicalRecoveryAllocationAdmission::new(store, bytes)
}

#[test]
fn wal_fingerprint_uses_typed_numeric_order_and_denies_duplicate_identity() {
    let mut fingerprints = vec![entry(10, 1), entry(2, 2), entry(2, 1)];
    finish_inventory_identity(&mut fingerprints).unwrap();
    assert_eq!(
        fingerprints
            .iter()
            .map(|item| item.identity)
            .collect::<Vec<_>>(),
        [identity(2, 1), identity(2, 2), identity(10, 1)]
    );
    let mut duplicate = vec![entry(10, 1), entry(2, 2), entry(2, 1), entry(2, 1)];
    assert!(matches!(
        finish_inventory_identity(&mut duplicate),
        Err(Denial::WalFate)
    ));
}

#[test]
fn resident_fingerprint_handoff_moves_precharged_backing_without_clone() {
    let mut resident = StoreRejoinResidentLedger::for_test(admission(8192), 100, 1024).unwrap();
    let mut artifacts = resident.reserve_vec::<WalArtifactFingerprint>(2).unwrap();
    artifacts.push(entry(2, 1));
    artifacts.push(entry(10, 1));
    finish_inventory_identity(&mut artifacts).unwrap();
    let original_ptr = artifacts.as_ptr();
    let charged = resident.vector_bytes(&artifacts).unwrap();
    let inventory = AdmittedWalInventory {
        frames: Vec::new(),
        artifacts,
    };
    let fingerprint = inventory
        .into_fingerprint_with_resident(&mut resident)
        .unwrap();
    assert_eq!(fingerprint.artifacts.as_ptr(), original_ptr);
    assert_eq!(fingerprint.owned_heap_bytes(), Some(charged));
    assert_eq!(resident.used(), 100 + charged);
}
