use super::*;
use crate::physical_runtime::durability::{
    DisplacedArtifact, PhysicalPublicationAdmission, PhysicalRetentionProfile,
};
use crate::physical_runtime::{
    LifecycleGeneration, PhysicalMutationIdentity, PhysicalOperationIdentity,
    PhysicalWorkGeneration, PhysicalWorkIdentity, RuntimeIdentity,
};
use std::num::NonZeroU64;
use std::sync::Mutex;
use worth_store_physical_format::store_namespace::{
    ProposedStoreIdentity, StoreNamespaceIdentityRecord, StoreNamespaceVersion,
};

fn mutation() -> PhysicalMutationIdentity {
    let store = StoreNamespaceIdentityRecord::new(
        StoreNamespaceVersion::CURRENT,
        ProposedStoreIdentity::from_nonzero_bytes([0x41; 16]).unwrap(),
    )
    .published_identity();
    PhysicalMutationIdentity::from_reserved_operation(PhysicalWorkIdentity::from_instance_owner(
        store,
        RuntimeIdentity::from_reopened(NonZeroU64::new(1).unwrap()),
        PhysicalWorkGeneration::from_lifecycle(LifecycleGeneration::from_reopened(
            NonZeroU64::new(1).unwrap(),
        )),
        PhysicalOperationIdentity::from_reopened(NonZeroU64::new(2).unwrap()),
    ))
}

#[test]
fn reconciled_fate_is_bound_to_exact_manifest_attempt_and_root() {
    let root = worth_store_physical_format::DurablePhysicalRootManifest::builder(1, 1, 2, 1)
        .admit()
        .unwrap()
        .root_cell();
    let other_root = worth_store_physical_format::DurablePhysicalRootManifest::builder(2, 1, 2, 1)
        .admit()
        .unwrap()
        .root_cell();
    let declaration = PersistedRecordIdentity::new([9; 16], 1).unwrap();
    let abandoned = PersistedRecordIdentity::new([9; 16], 2).unwrap();
    let manifest = PersistedRecordIdentity::new([9; 16], 3).unwrap();
    let basis =
        FailedIngestReclaimBasisV1::new([4; 16], declaration, [5; 32], abandoned, [6; 32]).unwrap();
    let fate = PhysicalReconciledReclaimDescriptorFate {
        store: [3; 16],
        original_attempt: [7; 16],
        basis_digest: basis.digest([3; 16]),
        manifest_record: manifest,
        manifest_sha256: [8; 32],
        selected_root: root,
        drop_idempotency: [10; 32],
        drop_fingerprint: [11; 32],
    };
    assert!(fate.matches([3; 16], [7; 16], basis, manifest, [8; 32], root));
    assert!(!fate.matches([3; 16], [1; 16], basis, manifest, [8; 32], root));
    assert!(!fate.matches([3; 16], [7; 16], basis, manifest, [1; 32], root));
    assert!(!fate.matches([3; 16], [7; 16], basis, manifest, [8; 32], other_root));
}

#[test]
fn never_reserved_proof_is_bound_to_selected_manifest_slot() {
    let basis = FailedIngestReclaimBasisV1::new(
        [4; 16],
        PersistedRecordIdentity::new([9; 16], 1).unwrap(),
        [5; 32],
        PersistedRecordIdentity::new([9; 16], 2).unwrap(),
        [6; 32],
    )
    .unwrap();
    let manifest_record = PersistedRecordIdentity::new([9; 16], 3).unwrap();
    let manifest = worth_store_physical_format::DropSetManifestV2::new(
        [3; 16],
        [7; 16],
        basis,
        vec![PersistedRecordIdentity::new([9; 16], 4).unwrap()],
        5,
    )
    .unwrap();
    let root = worth_store_physical_format::DurablePhysicalRootManifest::builder(5, 1, 2, 1)
        .admit()
        .unwrap()
        .root_cell();
    let other_root = worth_store_physical_format::DurablePhysicalRootManifest::builder(6, 1, 2, 1)
        .admit()
        .unwrap()
        .root_cell();
    let proof =
        ManifestResidueProof::never_reserved(&manifest, manifest_record, [8; 32], root).unwrap();
    assert!(proof.matches([3; 16], [7; 16], basis, manifest_record, [8; 32], root));
    assert!(!proof.matches(
        [3; 16],
        [7; 16],
        basis,
        manifest_record,
        [8; 32],
        other_root
    ));
    assert_eq!(
        proof.wire_proof(),
        worth_store_physical_format::OriginalDropProofV1::NeverReserved
    );
}

#[test]
fn one_manifest_fence_reserves_one_slot_and_releases_only_before_effect() {
    let admission = Arc::new(PhysicalPublicationAdmission::new(
        PhysicalRetentionProfile::new(100, 4, 20, 1).unwrap(),
    ));
    let capacity = admission.reserve_displaced_entries(1, 40).unwrap();
    let fence = Arc::new(Mutex::new(Some(ReclaimFenceState {
        id: PhysicalReclaimAttemptId([7; 16]),
        expected_root: worth_store_physical_format::DurablePhysicalRootManifest::builder(
            1, 1, 2, 1,
        )
        .admit()
        .unwrap()
        .root_cell(),
        manifest_mutation: None,
        reservation_mutation: None,
        drop_mutation: None,
        retirement_mutation: None,
        drop_records: Vec::new(),
        displaced: vec![DisplacedArtifact {
            source_root: 1,
            artifact: RetiredArtifact::Extent {
                extent: 1,
                generation: 1,
                range: worth_store_physical_format::ExtentArenaRange::new(
                    worth_store_physical_format::ExtentArenaId::new(1).unwrap(),
                    0,
                    40,
                )
                .unwrap(),
            },
            bytes: 40,
        }],
        phase: ReclaimPhase::BeforeEffect,
        purpose: ReclaimPurpose::ManifestResidue,
        _capacity: Some(capacity),
        _recovered_reservations: Vec::new(),
        release_certificate_pending: None,
    })));
    assert!(admission.reserve_displaced_entries(2, 41).is_err());
    let attempt = PhysicalReclaimAttempt {
        fence: Arc::clone(&fence),
        state: std::sync::Weak::new(),
        id: PhysicalReclaimAttemptId([7; 16]),
    };
    assert!(!attempt.register_mutation(mutation()));
    assert!(!fence.lock().unwrap().as_ref().unwrap().accepts(mutation()));
    drop(attempt);
    assert!(fence.lock().unwrap().is_none());
    assert!(admission.reserve_displaced_entries(2, 41).is_ok());
}
