use worth_proof::TransitionOutcome;
use worth_store::physical_runtime::{
    FilesystemAccessPosture, FilesystemMediaAdmission, PhysicalRuntimeAdmission, PhysicalStore,
    QualifiedRecoveryFilesystemMedia,
};
use worth_store_physical_format::{
    store_namespace::StableStoreIdentity, DurablePhysicalRootManifest, FreeSpaceBlockReference,
    FreeSpaceKey, PhysicalRecordFormatDeclaration, RecordAllocationClass,
};
use worth_store_physical_integrity::{
    validate_root_manifest, PhysicalArtifactScope, PhysicalByteRange, UntrustedPhysicalArtifact,
};

use super::super::{
    IntegrityAdmittedRecoveryArtifact, RecoveryIntegrityIngressCounters,
    RecoveryIntegrityIngressObservationOutcome, RecoveryIntegrityIngressRejection,
};

#[test]
fn checksum_valid_root_bytes_cannot_relabel_their_c4_store_locator_or_offset() {
    let parent = tempfile::tempdir().unwrap();
    let root_a = parent.path().join("source");
    let root_b = parent.path().join("foreign");
    let store_a = initialize_media(&root_a);
    let store_b = initialize_media(&root_b);
    assert_ne!(store_a, store_b);
    let format = PhysicalRecordFormatDeclaration::builder().admit().unwrap();
    let key = FreeSpaceKey::new(RecordAllocationClass::InlinePage, 1).unwrap();
    let free = FreeSpaceBlockReference::new(1, 1, 0, 41, key, key).unwrap();
    let bytes = DurablePhysicalRootManifest::builder(1, 71, 2, 43)
        .free_space_root(Some(free))
        .admit()
        .unwrap()
        .encode(format);
    let roots = root_a.join("families/records/roots");
    std::fs::create_dir_all(&roots).unwrap();
    std::fs::write(roots.join("root-0000000000000001.manifest"), &bytes).unwrap();
    let media = QualifiedRecoveryFilesystemMedia::qualify_existing(&root_a)
        .unwrap()
        .admit_persisted_store()
        .unwrap();
    let mut discovery = media.bounded_discovery(1, 4096).unwrap();
    let observed = discovery.read_root_manifest(1, 4096).unwrap();
    let range = PhysicalByteRange::new(0, bytes.len() as u64).unwrap();
    let wrong_locator = PhysicalArtifactScope::root_manifest(store_a, format, 2, range).unwrap();
    assert!(matches!(
        super::super::source::ObservedRecoverySource::complete(&observed, wrong_locator).input(),
        Err(RecoveryIntegrityIngressRejection::ScopeMismatch)
    ));

    for (store, generation, offset, expected) in [
        (
            store_a,
            1,
            0,
            RecoveryIntegrityIngressObservationOutcome::Admitted,
        ),
        (
            store_b,
            1,
            0,
            RecoveryIntegrityIngressObservationOutcome::Rejected(
                RecoveryIntegrityIngressRejection::ScopeMismatch,
            ),
        ),
        (
            store_a,
            1,
            64,
            RecoveryIntegrityIngressObservationOutcome::Rejected(
                RecoveryIntegrityIngressRejection::ScopeMismatch,
            ),
        ),
    ] {
        let scope = PhysicalArtifactScope::root_manifest(
            store,
            format,
            generation,
            PhysicalByteRange::new(offset, range.length()).unwrap(),
        )
        .unwrap();
        // This format does not embed Store identity. The C4 source, not a
        // successful checksum under a caller-supplied scope, must bind it.
        let validation = validate_root_manifest(
            UntrustedPhysicalArtifact::from_bounded_bytes(observed.bytes().unwrap()),
            scope,
        )
        .0;
        let mut counters = RecoveryIntegrityIngressCounters::default();
        let attempt = IntegrityAdmittedRecoveryArtifact::bind_root_manifest(
            &observed,
            scope,
            validation,
            &mut counters,
        );
        assert_eq!(attempt.observation().outcome(), expected);
        assert_eq!(counters.owner_decoder_entries(), 0);
    }
    drop(discovery.finish());
}

fn initialize_media(root: &std::path::Path) -> StableStoreIdentity {
    let runtime =
        PhysicalStore::admit(PhysicalRuntimeAdmission::new(root.to_path_buf()).unwrap()).unwrap();
    let admission =
        FilesystemMediaAdmission::production(FilesystemAccessPosture::CoordinatedServiceAccount);
    let TransitionOutcome::Success(media) =
        runtime.try_admit_filesystem_media(admission).into_raw()
    else {
        panic!("test source requires real C4 media admission")
    };
    let store = media.store_identity();
    media.close();
    store
}
