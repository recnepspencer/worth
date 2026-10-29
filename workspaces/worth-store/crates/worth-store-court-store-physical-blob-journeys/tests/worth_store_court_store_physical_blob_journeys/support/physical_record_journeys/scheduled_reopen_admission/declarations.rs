pub(crate) use {std::path::Path};

pub(crate) use {worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial, worth_store::physical_runtime::PhysicalRecoveryCoordination, worth_store::physical_runtime::PhysicalRecoveryCoordinationCapacity, worth_store::physical_runtime::PhysicalRecoveryFreshReopenCommand, worth_store::physical_runtime::PhysicalRecoveryFreshReopenOutcome, worth_store::physical_runtime::PhysicalRecoveryFreshnessPort, worth_store::physical_runtime::PhysicalRootProtocolRoute, worth_store::physical_runtime::QualifiedRecoveryFilesystemMedia, worth_store::physical_runtime::RecordAppendBatch, worth_store::physical_runtime::RootProtocolAdmissionDenial};
pub(crate) use {worth_store_physical_format::DurablePhysicalRootManifest, worth_store_physical_format::DurableRootSelector, worth_store_physical_format::RecordArtifactFile, worth_store_physical_format::RootSelectorRole};

pub(crate) use {worth_store_testkit_store::physical_record_journeys::configuration, worth_store_testkit_store::durable_publication, worth_store_testkit_store::physical_record_journeys::serving_from_initialization};

pub(crate) fn published_root_protocol(root: &Path) -> (DurablePhysicalRootManifest, DurableRootSelector) {
    let serving = serving_from_initialization(root);
    let (_, placement, _) = configuration();
    let completed = worth_store_testkit_store::durable_publication::root::publish_single(
        &serving,
        placement,
        worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial::new([0x93; 32]),
        worth_store::physical_runtime::RecordAppendBatch::try_from_iter([b"root-protocol-cutover".as_slice()]).unwrap(),
    );
    let expected_root = completed.current_root().clone();
    serving.close();
    let expected_selector =
        worth_store_physical_format::DurableRootSelector::decode(&std::fs::read(selector_path(root)).unwrap()).unwrap();
    (expected_root, expected_selector)
}

pub(crate) fn recovery_coordination(
    root: &Path,
) -> (
    PhysicalRecoveryCoordination,
    worth_store::physical_runtime::AdmittedRecoveryFilesystemMedia,
) {
    let qualified = worth_store::physical_runtime::QualifiedRecoveryFilesystemMedia::qualify_existing(root).unwrap();
    let freshness = worth_store::physical_runtime::PhysicalRecoveryFreshnessPort::admit(&qualified).unwrap();
    let media = qualified.admit_persisted_store().unwrap();
    let coordination = freshness
        .register_session()
        .unwrap()
        .admit_coordination(
            &media,
            worth_store::physical_runtime::PhysicalRecoveryCoordinationCapacity::admit(64, 1 << 20, 16, 1 << 20).unwrap(),
            worth_store::physical_runtime::AdmittedPhysicalRecordResidencyPolicy::canonical(
                configuration().0,
            ),
            None,
        )
        .unwrap();
    (coordination, media)
}

pub(crate) fn command(
    root: DurablePhysicalRootManifest,
    selector: DurableRootSelector,
) -> PhysicalRecoveryFreshReopenCommand {
    worth_store::physical_runtime::PhysicalRecoveryFreshReopenCommand::new([0x57; 32], root, selector, selector.format()).unwrap()
}

pub(crate) fn selector_path(root: &Path) -> std::path::PathBuf {
    root.join("families")
        .join("records")
        .join(worth_store_physical_format::RecordArtifactFile::CurrentRootSelector.file_name())
}

pub(crate) fn root_path(root: &Path, generation: u64) -> std::path::PathBuf {
    root.join("families")
        .join("records")
        .join("roots")
        .join(worth_store_physical_format::RecordArtifactFile::RootManifest { generation }.file_name())
}