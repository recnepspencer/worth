//! A real recovery pool for effect-media ownership checks.

use crate::physical_runtime::{
    AdmittedPhysicalRecordFormat, AdmittedPhysicalRecordResidencyPolicy,
    AdmittedRecoveryFilesystemMedia, FilesystemAccessPosture, FilesystemMediaAdmission,
    PhysicalRecoveryCoordination, PhysicalRecoveryCoordinationCapacity,
    PhysicalRecoveryFreshnessPort, PhysicalRuntimeAdmission, PhysicalStore,
    QualifiedRecoveryFilesystemMedia,
};
use worth_proof::TransitionOutcome;
use worth_store_physical_format::{PersistedReleaseCustodyHeadEffectV1, RecordArtifactFile};

pub(in super::super) fn coordination() -> (
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
    let format = AdmittedPhysicalRecordFormat::admit(
        super::PhysicalRecordFormatDeclaration::builder()
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
                .with_recovery_allocation_bytes(2 << 20)
                .unwrap(),
            AdmittedPhysicalRecordResidencyPolicy::canonical(format),
            None,
        )
        .unwrap();
    (directory, media, coordination)
}

/// Install the format owner's actual planned frames for a C4 read test. This
/// fixture does not create a selected root or claim C8 release authority.
pub(super) fn install_planned_nodes(
    root: &std::path::Path,
    effect: &PersistedReleaseCustodyHeadEffectV1,
) -> Vec<std::path::PathBuf> {
    let directory = root.join("families/records/roots");
    std::fs::create_dir_all(&directory).unwrap();
    effect
        .source_path()
        .iter()
        .map(|node| (node.reference(), node.frame()))
        .chain(
            effect
                .node_writes()
                .iter()
                .map(|node| (node.reference(), node.frame())),
        )
        .map(|(reference, frame)| {
            let path = directory.join(
                RecordArtifactFile::ReleaseCustodyHeadBlock {
                    generation: reference.generation(),
                    block: reference.block(),
                }
                .file_name(),
            );
            std::fs::write(&path, frame).unwrap();
            path
        })
        .collect()
}
