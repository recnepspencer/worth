

pub(crate) use {crate::phase_three_manifest_denials::phase_three_support::synthetic_topology::publish_synthetic_chained_genesis, crate::phase_three_manifest_denials::phase_three_support::synthetic_topology::publish_synthetic_nonempty_genesis, crate::phase_three_manifest_denials::phase_three_support::synthetic_topology::publish_synthetic_paired_genesis};
pub(crate) use {std::path::Path, worth_store_testkit_recovery_runtime::phase_three_support::admitted_recovery, worth_store_testkit_recovery_runtime::phase_three_support::admitted_recovery_with_limits, worth_store_testkit_recovery_runtime::phase_three_support::expect_blocked, worth_store_testkit_recovery_runtime::phase_three_support::initialize_store, worth_store_testkit_recovery_runtime::phase_three_support::limit_declaration};
pub(crate) use {worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily};
pub(crate) use {worth_store_physical_integrity::PhysicalDamageCause, worth_store_physical_integrity::PhysicalIntegrityRejection};
pub(crate) use {worth_store_recovery_physics::PhysicalPageFactDenial};
pub(crate) use {worth_store_recovery_runtime::PhysicalManifestObservationDenial, worth_store_recovery_runtime::PhysicalRecoveryBlockKind, worth_store_recovery_runtime::PhysicalRecoveryIntegrityObservationOutcome, worth_store_recovery_runtime::PhysicalRecoveryIntegrityRejection, worth_store_recovery_runtime::PhysicalRecoveryLimitDimension, worth_store_recovery_runtime::PhysicalRecoveryLimits, worth_store_recovery_runtime::PhysicalRecoveryRootProtocolDenial, worth_store_recovery_runtime::PhysicalRecoverySourceDenial};

pub(crate) fn discover_under(
    manifest_entries: u64,
    publish: impl FnOnce(
        &std::path::Path,
        worth_store_physical_format::store_namespace::StableStoreIdentity,
    ),
) -> (
    tempfile::TempDir,
    Result<
        worth_store_recovery_runtime::DiscoveredPhysicalRecovery,
        worth_store_recovery_runtime::PhysicalRecoveryOutcome,
    >,
) {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("routed");
    let store = initialize_store(&root);
    publish(&root, store);
    let mut declaration = limit_declaration(2, 8, 64 * 1024);
    declaration.manifest_entries = manifest_entries;
    declaration.observation_bytes = 64 * 1024;
    let discovered =
        admitted_recovery_with_limits(&root, worth_store_recovery_runtime::PhysicalRecoveryLimits::admit(declaration).unwrap())
            .discover();
    (parent, discovered)
}

pub(crate) fn manifest_denial(
    blocked: &worth_store_recovery_runtime::PhysicalRecoveryBlock,
) -> &PhysicalManifestObservationDenial {
    blocked
        .evidence()
        .source_denials
        .iter()
        .find_map(|denial| match denial {
            worth_store_recovery_runtime::PhysicalRecoverySourceDenial::ManifestObservation(denial) => Some(denial),
            _ => None,
        })
        .expect("the later typed manifest denial must remain present")
}

pub(crate) fn manifest_case(
    name: &str,
    mutate: impl FnOnce(&std::path::Path),
) -> worth_store_recovery_runtime::PhysicalRecoveryBlock {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join(name);
    let store = initialize_store(&root);
    publish_synthetic_nonempty_genesis(&root, store);
    mutate(
        &root
            .join("families")
            .join("records")
            .join("roots")
            .join("root-0000000000000001-block-0000000000000001.manifest"),
    );
    expect_blocked(
        admitted_recovery(&root)
            .discover()
            .unwrap()
            .select()
            .err()
            .expect("manifest observation denial must block"),
    )
}