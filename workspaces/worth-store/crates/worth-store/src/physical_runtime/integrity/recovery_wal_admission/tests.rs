use std::num::NonZeroU64;

use worth_proof::TransitionOutcome;
use worth_store_physical_format::wal_frame::{encode_wal_frame_v1, WalFrameV1EncodeRequest};
use worth_store_physical_integrity::{validate_wal_frame, UntrustedPhysicalArtifact};

use super::*;
use crate::physical_runtime::{
    FilesystemAccessPosture, FilesystemMediaAdmission, PhysicalRuntimeAdmission, PhysicalStore,
    QualifiedRecoveryFilesystemMedia,
};

#[test]
fn c4_observation_cannot_be_substituted_during_segment_assembly() {
    let parent = tempfile::tempdir().unwrap();
    let root_a = parent.path().join("store-a");
    let root_b = parent.path().join("store-b");
    let store_a = initialize(&root_a);
    let _store_b = initialize(&root_b);
    let identity = WalSegmentIdentity::new(1, 1).unwrap();
    let frame = encode_wal_frame_v1(
        WalFrameV1EncodeRequest::from_segment_identity(
            identity,
            2,
            3,
            b"store-substitution",
            b"payload",
        )
        .unwrap(),
    );
    let file_name = "segment-1-generation-1.wal";
    for root in [&root_a, &root_b] {
        let wal = root.join("families").join("wal");
        std::fs::create_dir_all(&wal).unwrap();
        std::fs::write(wal.join(file_name), &frame).unwrap();
    }
    let (media_a, coordination_a) =
        super::media_generation_tests::recovery_media_and_coordination(&root_a);
    let media_b = QualifiedRecoveryFilesystemMedia::qualify_existing(&root_b)
        .unwrap()
        .admit_persisted_store()
        .unwrap();
    let mut discovery_a = media_a.bounded_discovery(2, 4096).unwrap();
    let mut discovery_b = media_b.bounded_discovery(2, 4096).unwrap();
    let observed_a = discovery_a
        .read_wal_artifacts(NonZeroU64::MIN, 4096)
        .unwrap();
    let repeated_a = discovery_a
        .read_wal_artifacts(NonZeroU64::MIN, 4096)
        .unwrap();
    let observed_b = discovery_b
        .read_wal_artifacts(NonZeroU64::MIN, 4096)
        .unwrap();
    let media_a = discovery_a.finish();
    let mut rediscovery_a = media_a.bounded_discovery(1, 4096).unwrap();
    let rediscovered_a = rediscovery_a
        .read_wal_artifacts(NonZeroU64::MIN, 4096)
        .unwrap();
    let range = PhysicalByteRange::new(0, frame.len() as u64).unwrap();
    let scope = PhysicalArtifactScope::wal_frame(store_a, identity, range);
    let validation = validate_wal_frame(
        UntrustedPhysicalArtifact::from_bounded_bytes(observed_a[0].bytes().unwrap()),
        scope,
    )
    .0;
    let worth_store_physical_integrity::WalFrameIntegrityValidation::Intact(validated) = validation
    else {
        panic!("fixture frame must be intact")
    };
    let admitted = coordination_a
        .admit_recovery_wal_frame(&observed_a[0], scope, range, validated)
        .unwrap();
    let artifact_identity = worth_store_wal::WalSegmentArtifactIdentity::parse(file_name)
        .expect("canonical fixture identity");
    for observed in [&observed_b[0], &repeated_a[0], &rediscovered_a[0]] {
        match coordination_a.begin_recovery_wal_segment(observed, artifact_identity) {
            Ok(mut builder) => assert!(builder.push(admitted.clone()).is_err()),
            Err(
                RecoveryWalIntegrityAdmissionDenial::ScopeMismatch
                | RecoveryWalIntegrityAdmissionDenial::SourceIncarnationMismatch,
            ) => {}
            Err(other) => panic!("wrong source-substitution denial: {other:?}"),
        }
    }
    let mut builder = coordination_a
        .begin_recovery_wal_segment(&observed_a[0], artifact_identity)
        .unwrap();
    builder.push(admitted).unwrap();
    assert!(builder.finish().is_ok());
    drop(rediscovery_a.finish());
    drop(discovery_b.finish());
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
