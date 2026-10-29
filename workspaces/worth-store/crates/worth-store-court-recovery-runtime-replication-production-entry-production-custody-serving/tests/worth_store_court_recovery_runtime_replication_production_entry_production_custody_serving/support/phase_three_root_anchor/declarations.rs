

pub(crate) use {crate::phase_three_root_anchor::phase_three_support::synthetic_topology::publish_synthetic_nonempty_genesis};
pub(crate) use {std::path::Path, worth_store_testkit_recovery_runtime::phase_three_support::admitted_recovery, worth_store_testkit_recovery_runtime::phase_three_support::admitted_recovery_with_limits, worth_store_testkit_recovery_runtime::phase_three_support::expect_blocked, worth_store_testkit_recovery_runtime::phase_three_support::initialize_store, worth_store_testkit_recovery_runtime::phase_three_support::limit_declaration, worth_store_testkit_recovery_runtime::phase_three_support::publish_synthetic_genesis};
pub(crate) use {worth_store_physical_format::BootstrapCatalog, worth_store_physical_format::CurrentRootCatalogEntry, worth_store_physical_format::CurrentRootCatalogGeneration, worth_store_physical_format::DurableRootSelector, worth_store_physical_format::PhysicalRecordFormatDeclaration, worth_store_physical_format::RootSelectorIdentity, worth_store_physical_format::RootSelectorRole};
pub(crate) use {worth_store_physical_integrity::PhysicalDamageCause, worth_store_physical_integrity::PhysicalIntegrityRejection};
pub(crate) use {worth_store_recovery_physics::PhysicalRootCandidateDenial, worth_store_recovery_physics::PhysicalRootSelectionDenial, worth_store_recovery_physics::SelectedPhysicalRootRole};
pub(crate) use {worth_store_recovery_runtime::PhysicalRecoveryLimits, worth_store_recovery_runtime::PhysicalRecoveryOutcome, worth_store_recovery_runtime::PhysicalRecoveryRootProtocolArtifact, worth_store_recovery_runtime::PhysicalRecoveryRootProtocolDenial, worth_store_recovery_runtime::PhysicalRecoverySourceDenial};

pub(crate) fn assert_refusal_retains(
    outcome: PhysicalRecoveryOutcome,
    artifact: PhysicalRecoveryRootProtocolArtifact,
) {
    let worth_store_recovery_runtime::PhysicalRecoveryOutcome::Refused(refusal) = outcome else {
        panic!("pre-reconstruction cancellation must refuse recovery");
    };
    assert!(refusal
        .root_protocol_denials()
        .iter()
        .any(|denial| matches!(
            denial,
            worth_store_recovery_runtime::PhysicalRecoverySourceDenial::RootProtocol {
                artifact: observed,
                ..
            } if *observed == artifact
        )));
}

pub(crate) fn publish_previous_and_anchor(
    root: &std::path::Path,
    store: worth_store_physical_format::store_namespace::StableStoreIdentity,
    anchor_generation: u64,
) {
    let format = worth_store_physical_format::PhysicalRecordFormatDeclaration::builder().admit().unwrap();
    let previous = worth_store_physical_format::DurableRootSelector::new(
        store,
        format,
        worth_store_physical_format::RootSelectorIdentity::new(1).unwrap(),
        worth_store_physical_format::RootSelectorRole::Previous,
        1,
        worth_store_physical_format::RootSelectorIdentity::new(2),
        Some(2),
    )
    .unwrap();
    let records = root.join("families").join("records");
    std::fs::write(records.join("root-previous.selector"), previous.encode()).unwrap();
    let catalog = worth_store_physical_format::BootstrapCatalog::new(
        store,
        format,
        worth_store_physical_format::CurrentRootCatalogEntry::new(worth_store_physical_format::CurrentRootCatalogGeneration::new(anchor_generation).unwrap()),
    );
    std::fs::write(records.join("bootstrap.catalog"), catalog.encode()).unwrap();
    let current_path = records.join("root-current.selector");
    let mut current = std::fs::read(&current_path).unwrap();
    current[65] ^= 0x5a;
    std::fs::write(current_path, current).unwrap();
}