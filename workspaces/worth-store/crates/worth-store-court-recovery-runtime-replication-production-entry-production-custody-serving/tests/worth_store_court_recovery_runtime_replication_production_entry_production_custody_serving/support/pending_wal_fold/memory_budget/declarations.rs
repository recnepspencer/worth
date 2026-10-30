//! Final live planning state must fit before any staging or publication effect.

pub(crate) use {std::fs, std::io::ErrorKind, std::path::Path};

pub(crate) use worth_store_physical_format::RecordArtifactFile;
pub(crate) use worth_store_recovery_physics::RecoveryPlanCostDenial;
pub(crate) use {
    worth_store_recovery_runtime::PhysicalRecoveryBlockKind,
    worth_store_recovery_runtime::PhysicalRecoveryLimitDimension,
    worth_store_recovery_runtime::PhysicalRecoveryOutcome,
    worth_store_recovery_runtime::PhysicalRecoveryPlanningDenial,
    worth_store_recovery_runtime::WorthStoreRecovery,
};

pub(crate) fn selector(root: &Path, artifact: RecordArtifactFile) -> Option<Vec<u8>> {
    let path = root.join("families/records").join(artifact.file_name());
    match std::fs::read(path) {
        Ok(bytes) => Some(bytes),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => None,
        Err(error) => panic!("selected selector read failed: {error}"),
    }
}

pub(crate) fn assert_unchanged(
    root: &Path,
    current: &Option<Vec<u8>>,
    previous: &Option<Vec<u8>>,
    checkpoint: &[u8],
) {
    assert_eq!(
        &selector(
            root,
            worth_store_physical_format::RecordArtifactFile::CurrentRootSelector
        ),
        current,
    );
    assert_eq!(
        &selector(
            root,
            worth_store_physical_format::RecordArtifactFile::PreviousRootSelector
        ),
        previous,
    );
    assert_eq!(
        std::fs::read(root.join("families/checkpoint.current")).unwrap(),
        checkpoint
    );
}
