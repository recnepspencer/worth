

pub(crate) use {std::path::Path, worth_store_testkit_recovery_runtime::phase_three_support::admitted_recovery, worth_store_testkit_recovery_runtime::phase_three_support::expect_blocked, worth_store_testkit_recovery_runtime::phase_three_support::initialize_store, worth_store_testkit_recovery_runtime::phase_three_support::publish_synthetic_genesis};
pub(crate) use {worth_store_physical_format::BootstrapCatalog, worth_store_physical_format::CurrentRootCatalogEntry, worth_store_physical_format::CurrentRootCatalogGeneration, worth_store_physical_format::DurableRootSelector, worth_store_physical_format::PhysicalRecordFormatDeclaration, worth_store_physical_format::RootSelectorIdentity, worth_store_physical_format::RootSelectorRole};
pub(crate) use {worth_store_physical_integrity::PhysicalDamageCause, worth_store_physical_integrity::PhysicalIntegrityRejection};
pub(crate) use {worth_store_recovery_runtime::PhysicalRecoveryRootProtocolArtifact, worth_store_recovery_runtime::PhysicalRecoveryRootProtocolDenial, worth_store_recovery_runtime::PhysicalRecoverySourceDenial};

pub(crate) fn assert_rejected_before_projection(
    counters: worth_store_recovery_runtime::PhysicalRecoveryDiscoveryCounters,
    absent: bool,
) {
    assert_eq!(counters.bootstrap_integrity_attempts, 1);
    assert_eq!(counters.bootstrap_integrity_admissions, 0);
    assert_eq!(counters.bootstrap_integrity_rejections, u64::from(!absent));
    assert_eq!(counters.bootstrap_owner_projections, 0);
    assert_eq!(counters.bootstrap_owner_decoder_entries, 0);
}

pub(crate) fn assert_bootstrap_damage(
    denials: &[PhysicalRecoverySourceDenial],
    expected: impl Fn(PhysicalDamageCause) -> bool,
) {
    assert!(denials.iter().any(|denial| matches!(
        denial,
        worth_store_recovery_runtime::PhysicalRecoverySourceDenial::RootProtocol {
            artifact: worth_store_recovery_runtime::PhysicalRecoveryRootProtocolArtifact::BootstrapCatalog,
            denial: worth_store_recovery_runtime::PhysicalRecoveryRootProtocolDenial::Integrity(
                worth_store_physical_integrity::PhysicalIntegrityRejection::Damaged(localization)
            ),
        } if expected(localization.cause())
    )));
}

pub(crate) fn prepare_previous_fallback(
    root: &std::path::Path,
    store: worth_store_physical_format::store_namespace::StableStoreIdentity,
) {
    publish_synthetic_genesis(root, store);
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
    std::fs::write(
        records(root).join("root-previous.selector"),
        previous.encode(),
    )
    .unwrap();
    let current_path = records(root).join("root-current.selector");
    let mut current = std::fs::read(&current_path).unwrap();
    current[65] ^= 0x5a;
    std::fs::write(current_path, current).unwrap();
}

pub(crate) fn publish_catalog(
    root: &std::path::Path,
    store: worth_store_physical_format::store_namespace::StableStoreIdentity,
) {
    let catalog = worth_store_physical_format::BootstrapCatalog::new(
        store,
        worth_store_physical_format::PhysicalRecordFormatDeclaration::builder().admit().unwrap(),
        worth_store_physical_format::CurrentRootCatalogEntry::new(worth_store_physical_format::CurrentRootCatalogGeneration::new(2).unwrap()),
    );
    std::fs::write(records(root).join("bootstrap.catalog"), catalog.encode()).unwrap();
}

pub(crate) fn records(root: &std::path::Path) -> std::path::PathBuf {
    root.join("families").join("records")
}