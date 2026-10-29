pub(crate) use {worth_store_recovery_runtime::PhysicalRecoveryOutcome, worth_store_testkit_recovery_runtime::phase_three_support::admitted_recovery, worth_store_testkit_recovery_runtime::phase_three_support::initialize_store, crate::phase_four_planning::publish_secured_synthetic_checkpoint, worth_store_testkit_recovery_runtime::phase_three_support::publish_synthetic_covered_wal, worth_store_testkit_recovery_runtime::phase_three_support::publish_synthetic_genesis};
pub(crate) use {worth_store_physical_integrity::PhysicalDamageCause, worth_store_physical_integrity::PhysicalIntegrityRejection};
pub(crate) use {worth_store_recovery_runtime::PhysicalRecoveryRootProtocolArtifact, worth_store_recovery_runtime::PhysicalRecoveryRootProtocolDenial, worth_store_recovery_runtime::PhysicalRecoverySourceDenial, worth_store_recovery_runtime::SelectedPhysicalRecovery};

/// The selection, and the directory that holds its store until the test
/// drops it.
pub(crate) fn selected_with_damaged_previous(label: &str) -> (tempfile::TempDir, SelectedPhysicalRecovery) {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join(label);
    let store = initialize_store(&root);
    publish_synthetic_genesis(&root, store);
    publish_secured_synthetic_checkpoint(&root, store);
    publish_synthetic_covered_wal(&root);
    let records = root.join("families").join("records");
    let mut previous = std::fs::read(records.join("root-current.selector")).unwrap();
    previous[65] ^= 0x5a;
    std::fs::write(records.join("root-previous.selector"), previous).unwrap();
    let selected = admitted_recovery(&root)
        .discover()
        .unwrap()
        .select()
        .unwrap();
    (parent, selected)
}

pub(crate) fn has_damaged_previous(denials: &[PhysicalRecoverySourceDenial]) -> bool {
    denials.iter().any(|denial| {
        matches!(
            denial,
            worth_store_recovery_runtime::PhysicalRecoverySourceDenial::RootProtocol {
                artifact: worth_store_recovery_runtime::PhysicalRecoveryRootProtocolArtifact::PreviousSelector,
                denial: worth_store_recovery_runtime::PhysicalRecoveryRootProtocolDenial::Integrity(
                    worth_store_physical_integrity::PhysicalIntegrityRejection::Damaged(localization)
                ),
            } if localization.cause() == worth_store_physical_integrity::PhysicalDamageCause::ChecksumMismatch
        )
    })
}