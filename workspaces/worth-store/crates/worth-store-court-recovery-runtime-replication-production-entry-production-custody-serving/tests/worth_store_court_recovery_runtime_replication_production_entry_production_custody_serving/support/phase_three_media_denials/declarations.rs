

pub(crate) use {crate::phase_three_media_denials::phase_three_support::synthetic_topology::publish_synthetic_nonempty_genesis};
pub(crate) use {std::path::Path, worth_store_testkit_recovery_runtime::phase_three_support::admitted_recovery, worth_store_testkit_recovery_runtime::phase_three_support::expect_blocked, worth_store_testkit_recovery_runtime::phase_three_support::initialize_store, worth_store_testkit_recovery_runtime::phase_three_support::publish_synthetic_genesis};
pub(crate) use {worth_store::physical_runtime::ArtifactTreeFailureKind, worth_store::physical_runtime::RecoveryDiscoveryArtifact};
pub(crate) use {worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily, worth_store_physical_format::RecordArtifactFile};
pub(crate) use {worth_store_physical_integrity::PhysicalDamageCause, worth_store_physical_integrity::PhysicalIntegrityRejection};
pub(crate) use {worth_store_recovery_runtime::PhysicalRecoveryBlockKind, worth_store_recovery_runtime::PhysicalRecoveryIntegrityObservationOutcome, worth_store_recovery_runtime::PhysicalRecoveryMediaObservationFailure, worth_store_recovery_runtime::PhysicalRecoveryRootProtocolArtifact, worth_store_recovery_runtime::PhysicalRecoveryRootProtocolDenial, worth_store_recovery_runtime::PhysicalRecoverySourceDenial};

pub(crate) fn assert_media_denial(
    blocked: &worth_store_recovery_runtime::PhysicalRecoveryBlock,
    expected_artifact: RecoveryDiscoveryArtifact,
) {
    assert_eq!(
        blocked.cause().damage(),
        Some(worth_store_recovery_runtime::PhysicalRecoveryBlockKind::MediaObservation)
    );
    let Some((artifact, failure)) =
        blocked
            .evidence()
            .source_denials
            .iter()
            .find_map(|denial| match denial {
                worth_store_recovery_runtime::PhysicalRecoverySourceDenial::MediaObservation { artifact, failure } => {
                    Some((artifact, failure))
                }
                _ => None,
            })
    else {
        panic!("media failure must retain one exact typed denial")
    };
    assert_eq!(artifact, &expected_artifact);
    assert!(
        matches!(
            failure,
            worth_store_recovery_runtime::PhysicalRecoveryMediaObservationFailure::Backend {
                kind: worth_store::physical_runtime::ArtifactTreeFailureKind::DeniedBeforeEffect,
                io_kind: Some(_),
            }
        ),
        "unexpected media failure: {failure:?}"
    );
    assert_eq!(blocked.recovery_effects(), 0);
}

pub(crate) fn current_selector(root: &std::path::Path) -> std::path::PathBuf {
    root.join("families")
        .join("records")
        .join("root-current.selector")
}

pub(crate) fn replace_with_unreadable_entry(path: &std::path::Path) {
    std::fs::remove_file(path).unwrap();
    replace_absent_with_unreadable_entry(path);
}

#[cfg(windows)]
pub(crate) fn replace_absent_with_unreadable_entry(path: &std::path::Path) {
    std::fs::create_dir(path).unwrap();
}

#[cfg(unix)]
pub(crate) fn replace_absent_with_unreadable_entry(path: &std::path::Path) {
    std::os::unix::fs::symlink(path.file_name().unwrap(), path).unwrap();
}