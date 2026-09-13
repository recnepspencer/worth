use worth_proof::TransitionOutcome;
use worth_store_physical_format::wal_frame::{encode_wal_frame_v1, WalFrameV1EncodeRequest};
use worth_store_physical_integrity::{validate_wal_frame, UntrustedPhysicalArtifact};

use super::*;
use crate::physical_runtime::{
    AdmittedRecoveryFilesystemMedia, FilesystemAccessPosture, FilesystemMediaAdmission,
    PhysicalRecoveryCoordination, PhysicalRecoveryCoordinationCapacity,
    PhysicalRecoveryFreshnessPort, PhysicalRuntimeAdmission, PhysicalStore,
    QualifiedRecoveryFilesystemMedia,
};

#[test]
fn coordination_rejects_same_store_observed_under_another_media_generation() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("store");
    let store = initialize(&root);
    let identity = WalSegmentIdentity::new(1, 1).unwrap();
    let bytes = encode_wal_frame_v1(
        WalFrameV1EncodeRequest::from_segment_identity(
            identity,
            2,
            3,
            b"media-generation",
            b"payload",
        )
        .unwrap(),
    );
    let wal = root.join("families").join("wal");
    std::fs::create_dir_all(&wal).unwrap();
    std::fs::write(wal.join("segment-1-generation-1.wal"), &bytes).unwrap();
    let (media_b, coordination_b) = recovery_media_and_coordination(&root);
    let mut discovery_b = media_b.bounded_discovery(1, 4096).unwrap();
    let observed = discovery_b.read_wal_artifacts(1, 4096).unwrap();
    let range = PhysicalByteRange::new(0, bytes.len() as u64).unwrap();
    let scope = PhysicalArtifactScope::wal_frame(store, identity, range);
    let admitted = coordination_b
        .admit_recovery_wal_frame(&observed[0], scope, range, intact(&observed[0], scope))
        .unwrap();
    let media_b = discovery_b.finish();
    drop(coordination_b);
    drop(media_b);
    let (_media_a, coordination_a) = recovery_media_and_coordination(&root);
    assert!(matches!(
        coordination_a.admit_recovery_wal_frame(
            &observed[0],
            scope,
            range,
            intact(&observed[0], scope),
        ),
        Err(RecoveryWalIntegrityAdmissionDenial::SourceIncarnationMismatch)
    ));
    let artifact =
        worth_store_wal::WalSegmentArtifactIdentity::parse("segment-1-generation-1.wal").unwrap();
    assert!(coordination_a
        .retain_admitted_recovery_wal_segment(&observed[0], artifact, vec![admitted])
        .is_none());
}

fn intact(
    observed: &ObservedWalArtifact,
    scope: PhysicalArtifactScope,
) -> worth_store_physical_integrity::IntegrityValidatedWalFrame<'_> {
    let validation = validate_wal_frame(
        UntrustedPhysicalArtifact::from_bounded_bytes(observed.bytes().unwrap()),
        scope,
    )
    .0;
    let worth_store_physical_integrity::WalFrameIntegrityValidation::Intact(validation) =
        validation
    else {
        panic!("fixture frame must be intact")
    };
    validation
}

fn recovery_media_and_coordination(
    root: &std::path::Path,
) -> (
    AdmittedRecoveryFilesystemMedia,
    PhysicalRecoveryCoordination,
) {
    let qualified = QualifiedRecoveryFilesystemMedia::qualify_existing(root).unwrap();
    let freshness = PhysicalRecoveryFreshnessPort::admit(&qualified).unwrap();
    let media = qualified.admit_persisted_store().unwrap();
    let session = freshness.register_session().unwrap();
    let capacity = PhysicalRecoveryCoordinationCapacity::admit(2, 4096, 2, 4096).unwrap();
    let coordination = session.admit_coordination(&media, capacity, None).unwrap();
    (media, coordination)
}

fn initialize(
    root: &std::path::Path,
) -> worth_store_physical_format::store_namespace::StableStoreIdentity {
    let runtime =
        PhysicalStore::admit(PhysicalRuntimeAdmission::new(root.to_owned()).unwrap()).unwrap();
    let admission =
        FilesystemMediaAdmission::production(FilesystemAccessPosture::CoordinatedServiceAccount);
    let media = match runtime.try_admit_filesystem_media(admission).into_raw() {
        TransitionOutcome::Success(media) => media,
        _ => panic!("store initialization failed"),
    };
    let store = media.store_identity();
    let _ = media.close();
    store
}
