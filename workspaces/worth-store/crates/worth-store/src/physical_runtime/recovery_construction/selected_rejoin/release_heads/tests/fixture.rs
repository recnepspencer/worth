//! Actual persisted namespace, registered recovery session and native pool.

use crate::physical_runtime::{
    AdmittedPhysicalRecordFormat, AdmittedPhysicalRecordResidencyPolicy,
    AdmittedRecoveryFilesystemMedia, FilesystemAccessPosture, FilesystemMediaAdmission,
    PhysicalRecoveryCoordination, PhysicalRecoveryCoordinationCapacity,
    PhysicalRecoveryFreshnessPort, PhysicalRuntimeAdmission, PhysicalStore,
    QualifiedRecoveryFilesystemMedia,
};
use worth_proof::TransitionOutcome;

pub(super) fn coordination() -> (
    tempfile::TempDir,
    AdmittedRecoveryFilesystemMedia,
    PhysicalRecoveryCoordination,
) {
    let directory = tempfile::tempdir().unwrap();
    let runtime =
        PhysicalStore::admit(PhysicalRuntimeAdmission::new(directory.path()).unwrap()).unwrap();
    let TransitionOutcome::Success(media) = runtime
        .try_admit_filesystem_media(FilesystemMediaAdmission::production(
            FilesystemAccessPosture::CoordinatedServiceAccount,
        ))
        .into_raw()
    else {
        panic!("namespace initialization must admit");
    };
    media.close();
    let qualified = QualifiedRecoveryFilesystemMedia::qualify_existing(directory.path()).unwrap();
    let freshness = PhysicalRecoveryFreshnessPort::admit(&qualified).unwrap();
    let media = qualified.admit_persisted_store().unwrap();
    let format = AdmittedPhysicalRecordFormat::admit(super::format());
    let coordination = freshness
        .register_session()
        .unwrap()
        .admit_coordination(
            &media,
            PhysicalRecoveryCoordinationCapacity::admit(2, 4096, 2, 4096)
                .unwrap()
                .with_recovery_allocation_bytes(2 << 20)
                .unwrap(),
            AdmittedPhysicalRecordResidencyPolicy::canonical(format),
            None,
        )
        .unwrap();
    (directory, media, coordination)
}
