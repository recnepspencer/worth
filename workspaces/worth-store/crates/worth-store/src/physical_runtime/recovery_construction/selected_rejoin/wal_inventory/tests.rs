use super::*;
use worth_store_wal::{WalSegmentGeneration, WalSegmentId};

mod fixture;
mod native_storage;
#[cfg(windows)]
mod reread;

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
    let (_root, media, coordination) = fixture::coordination();
    let (owner, original, _) = coordination.sampling_allocation_basis().unwrap();
    let ports = owner.ports().clone();
    let observer = ports.allocation_events();
    let mut resident = StoreRejoinResidentLedger::for_test(original, 100, 4096).unwrap();
    let mut make_inventory = || {
        let mut artifacts = WalRoster::with_capacity(&coordination, 2).unwrap();
        let bytes = (artifacts.capacity() * std::mem::size_of::<WalArtifactFingerprint>()) as u64;
        resident.retain(bytes).unwrap();
        artifacts.push_reserved(entry(2, 1));
        artifacts.push_reserved(entry(10, 1));
        AdmittedWalInventory {
            frames: WalRoster::empty(),
            artifacts,
        }
    };
    let first = make_inventory();
    let inventory = make_inventory();
    assert!(first.matches_reread(&inventory));
    let charged =
        (inventory.artifacts.capacity() * std::mem::size_of::<WalArtifactFingerprint>()) as u64;
    assert_eq!(fixture::active(&ports), 2 * charged);
    first.discard_with_resident(&mut resident).unwrap();
    assert_eq!(fixture::active(&ports), charged);
    assert_eq!(resident.used(), 100 + charged);
    let artifacts = &inventory.artifacts;
    let original_ptr = artifacts.as_ptr();
    let before = observer.snapshot();
    let fingerprint = inventory
        .into_fingerprint_with_resident(&mut resident)
        .unwrap();
    assert_eq!(fingerprint.artifacts.as_ptr(), original_ptr);
    assert_eq!(fingerprint.owned_heap_bytes(), Some(charged));
    assert_eq!(fingerprint.charged_bytes(), charged);
    assert_eq!(resident.used(), 100 + charged);
    assert_eq!(observer.snapshot(), before);
    drop(coordination);
    drop(media);
    assert_eq!(fixture::active(&ports), charged);
    assert_eq!(fingerprint.artifacts.len(), 2);
    drop(fingerprint);
    assert_eq!(fixture::active(&ports), 0);
    fixture::assert_balanced(&observer);
}
