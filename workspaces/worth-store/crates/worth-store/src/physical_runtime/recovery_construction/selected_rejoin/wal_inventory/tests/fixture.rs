//! A real namespace/session/pool. Serialized WAL input supplies C9 grammar,
//! not a semantic transaction fate or a recovered Serving seal.
use crate::physical_runtime::{
    AdmittedPhysicalRecordFormat, AdmittedPhysicalRecordResidencyPolicy,
    AdmittedRecoveryFilesystemMedia, FilesystemAccessPosture, FilesystemMediaAdmission,
    PhysicalRecoveryCoordination, PhysicalRecoveryCoordinationCapacity,
    PhysicalRecoveryFreshnessPort, PhysicalRuntimeAdmission, PhysicalStore,
    QualifiedRecoveryFilesystemMedia,
};
use worth_proof::TransitionOutcome;
use worth_store_buffer_pool::{
    PhysicalOperationAllocationScope as Scope, PhysicalResidencyDimension as Dimension,
};

pub(super) const ORIGINAL: u64 = 2 << 20;

pub(super) fn coordination() -> (
    tempfile::TempDir,
    AdmittedRecoveryFilesystemMedia,
    PhysicalRecoveryCoordination,
) {
    let root = tempfile::tempdir().unwrap();
    let runtime =
        PhysicalStore::admit(PhysicalRuntimeAdmission::new(root.path()).unwrap()).unwrap();
    let TransitionOutcome::Success(media) = runtime
        .try_admit_filesystem_media(FilesystemMediaAdmission::production(
            FilesystemAccessPosture::CoordinatedServiceAccount,
        ))
        .into_raw()
    else {
        panic!("actual namespace initialization must admit");
    };
    media.close();
    let qualified = QualifiedRecoveryFilesystemMedia::qualify_existing(root.path()).unwrap();
    let freshness = PhysicalRecoveryFreshnessPort::admit(&qualified).unwrap();
    let media = qualified.admit_persisted_store().unwrap();
    let format = AdmittedPhysicalRecordFormat::admit(
        worth_store_physical_format::PhysicalRecordFormatDeclaration::builder()
            .admit()
            .unwrap(),
    );
    let coordination = freshness
        .register_session()
        .unwrap()
        .admit_coordination(
            &media,
            PhysicalRecoveryCoordinationCapacity::admit(2, 4096, 2, 4096)
                .unwrap()
                .with_recovery_allocation_bytes(ORIGINAL)
                .unwrap(),
            AdmittedPhysicalRecordResidencyPolicy::canonical(format),
            None,
        )
        .unwrap();
    (root, media, coordination)
}

pub(super) fn active(ports: &crate::physical_runtime::record_serving::RecordFramePorts) -> u64 {
    ports.counters().active_operation_bytes_for(Scope::Recovery)
}

pub(super) fn assert_balanced(
    observer: &worth_store_buffer_pool::PhysicalResidencyAllocationEventObserver,
) {
    let snapshot = observer.snapshot();
    let recovery = snapshot.for_dimension(Dimension::OperationScope(Scope::Recovery));
    assert_eq!(recovery.active_units(), 0);
    assert_eq!(recovery.admitted_units(), recovery.released_units());
}

#[cfg(windows)]
pub(super) fn wal(root: &std::path::Path) -> (std::path::PathBuf, Vec<u8>) {
    use worth_store_physical_format::wal_frame::{encode_wal_frame_v1, WalFrameV1EncodeRequest};
    let identity = worth_store_physical_format::WalSegmentIdentity::new(1, 1).unwrap();
    let bytes = encode_wal_frame_v1(
        WalFrameV1EncodeRequest::from_segment_identity(
            identity,
            2,
            3,
            b"inventory-owner",
            b"payload",
        )
        .unwrap(),
    );
    let directory = root.join("families/wal");
    std::fs::create_dir_all(&directory).unwrap();
    let path = directory.join("segment-1-generation-1.wal");
    std::fs::write(&path, &bytes).unwrap();
    (path, bytes)
}
