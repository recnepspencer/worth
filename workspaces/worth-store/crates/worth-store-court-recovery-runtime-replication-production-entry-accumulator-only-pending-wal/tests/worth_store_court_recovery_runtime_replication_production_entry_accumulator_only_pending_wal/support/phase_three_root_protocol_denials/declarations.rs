

pub(crate) use {std::path::Path, crate::phase_three_support::admitted_recovery, crate::phase_three_support::admitted_recovery_for_format, crate::phase_three_support::expect_blocked, crate::phase_three_support::initialize_store, crate::phase_three_support::publish_synthetic_genesis, crate::phase_three_support::publish_synthetic_genesis_for_format};
pub(crate) use {worth_store_physical_format::DurableRootSelector, worth_store_physical_format::PhysicalPageSizeClass, worth_store_physical_format::PhysicalRecordFormatDeclaration, worth_store_physical_format::RootSelectorRole, worth_store_physical_format::ROOT_SELECTOR_BYTES};
pub(crate) use {worth_store_physical_integrity::PhysicalDamageCause, worth_store_physical_integrity::PhysicalIntegrityRejection};
pub(crate) use {worth_store_recovery_physics::PhysicalRootCandidateDenial, worth_store_recovery_physics::PhysicalRootSelectionDenial};
pub(crate) use {worth_store_recovery_runtime::PhysicalRecoveryBlock, worth_store_recovery_runtime::PhysicalRecoveryRootProtocolArtifact, worth_store_recovery_runtime::PhysicalRecoveryRootProtocolDenial, worth_store_recovery_runtime::PhysicalRecoverySourceDenial};

pub(crate) fn root_case(name: &str, mutate: impl FnOnce(&std::path::Path)) -> PhysicalRecoveryBlock {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join(name);
    let store = initialize_store(&root);
    publish_synthetic_genesis(&root, store);
    mutate(&root);
    expect_blocked(
        admitted_recovery(&root)
            .discover()
            .unwrap()
            .select()
            .err()
            .expect("root denial must block"),
    )
}

pub(crate) fn current_selector(root: &std::path::Path) -> std::path::PathBuf {
    root.join("families")
        .join("records")
        .join("root-current.selector")
}

pub(crate) fn poison_covered_byte(path: std::path::PathBuf, offset: usize) {
    let mut bytes = std::fs::read(&path).unwrap();
    bytes[offset] ^= 0x5a;
    std::fs::write(path, bytes).unwrap();
}