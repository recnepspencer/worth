pub(crate) use std::num::NonZeroU64;

pub(crate) use {
    std::path::Path, worth_store_physical_format::RootSelectorIdentity,
    worth_store_testkit_recovery_runtime::phase_three_support::admitted_recovery,
    worth_store_testkit_recovery_runtime::phase_three_support::admitted_recovery_with_limits,
    worth_store_testkit_recovery_runtime::phase_three_support::expect_blocked,
    worth_store_testkit_recovery_runtime::phase_three_support::initialize_store,
    worth_store_testkit_recovery_runtime::phase_three_support::limit_declaration,
    worth_store_testkit_recovery_runtime::phase_three_support::limits,
    worth_store_testkit_recovery_runtime::phase_three_support::publish_synthetic_checkpoint,
    worth_store_testkit_recovery_runtime::phase_three_support::publish_synthetic_genesis,
};
pub(crate) use {
    worth_store_physical_format::CheckpointBindingCompactionHeader,
    worth_store_physical_format::CheckpointDirtyFrameBasis,
    worth_store_physical_format::CheckpointRootBasis,
    worth_store_physical_format::CheckpointStreamEncoder,
    worth_store_physical_format::CheckpointWalSourceRange,
    worth_store_physical_format::DurableRootSelector,
    worth_store_physical_format::PhysicalCheckpointIdentity,
    worth_store_physical_format::PhysicalCheckpointSource,
    worth_store_physical_format::RecordArtifactFile,
    worth_store_physical_format::RecordFrameCoordinate,
    worth_store_physical_format::RootSelectorRole,
    worth_store_physical_format::ROOT_SELECTOR_BYTES,
};
pub(crate) use {
    worth_store_recovery_physics::PhysicalRootCandidateDenial,
    worth_store_recovery_physics::PhysicalRootSelectionDenial,
    worth_store_recovery_physics::SelectedPhysicalWalTailDenial,
};
pub(crate) use {
    worth_store_recovery_runtime::PhysicalRecoveryBlock,
    worth_store_recovery_runtime::PhysicalRecoveryBlockKind,
    worth_store_recovery_runtime::PhysicalRecoveryCheckpointIntegrityDenial,
    worth_store_recovery_runtime::PhysicalRecoveryLimitDimension,
    worth_store_recovery_runtime::PhysicalRecoveryLimits,
    worth_store_recovery_runtime::PhysicalRecoveryRootProtocolArtifact,
    worth_store_recovery_runtime::PhysicalRecoveryRootProtocolDenial,
    worth_store_recovery_runtime::PhysicalRecoverySourceDenial,
};

pub(crate) fn root_case(
    name: &str,
    mutate: impl FnOnce(&std::path::Path),
) -> PhysicalRecoveryBlock {
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

pub(crate) fn checkpoint_case(
    name: &str,
    limits: PhysicalRecoveryLimits,
    write: impl FnOnce(
        &std::path::Path,
        worth_store_physical_format::store_namespace::StableStoreIdentity,
    ),
) -> PhysicalRecoveryBlock {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join(name);
    let store = initialize_store(&root);
    publish_synthetic_genesis(&root, store);
    let path = root.join("families").join("checkpoint.current");
    write(&path, store);
    expect_blocked(
        admitted_recovery_with_limits(&root, limits)
            .discover()
            .unwrap()
            .select()
            .err()
            .expect("checkpoint denial must block"),
    )
}

pub(crate) fn assert_checkpoint_denial(
    blocked: &PhysicalRecoveryBlock,
    expected: impl Fn(&PhysicalRecoveryCheckpointIntegrityDenial) -> bool,
) {
    assert!(blocked
        .evidence()
        .source_denials
        .iter()
        .any(|denial| match denial {
            worth_store_recovery_runtime::PhysicalRecoverySourceDenial::CheckpointIntegrity(
                observed,
            ) => expected(observed),
            _ => false,
        }));
}

pub(crate) fn current_selector(root: &std::path::Path) -> std::path::PathBuf {
    root.join("families")
        .join("records")
        .join("root-current.selector")
}

pub(crate) fn write_checkpoint_with_two_dirty_records(
    path: &std::path::Path,
    store: worth_store_physical_format::store_namespace::StableStoreIdentity,
) {
    let identity = worth_store_physical_format::PhysicalCheckpointIdentity::new(
        store,
        std::num::NonZeroU64::new(1).unwrap(),
    );
    let source = worth_store_physical_format::PhysicalCheckpointSource::concurrent(
        identity,
        worth_store_physical_format::CheckpointWalSourceRange::new(1, 2).unwrap(),
        worth_store_physical_format::CheckpointRootBasis::new(1, 7),
        1,
    );
    let (mut encoder, header) = worth_store_physical_format::CheckpointStreamEncoder::begin(source);
    let records = [1_u64, 2].map(|segment| {
        encoder.encode_dirty_basis(worth_store_physical_format::CheckpointDirtyFrameBasis::new(
            worth_store_physical_format::RecordFrameCoordinate::new(
                worth_store_physical_format::RecordArtifactFile::Segment {
                    segment,
                    generation: 1,
                },
                0,
                64,
            )
            .unwrap(),
            segment,
        ))
    });
    let cutover =
        worth_store_physical_format::CheckpointBindingCompactionHeader::new(1, 2).unwrap();
    let (compaction, cutover_record) = encoder.begin_binding_compaction(cutover);
    let (_, footer) = compaction.finish();
    let mut bytes = Vec::new();
    bytes.extend_from_slice(&header);
    for record in records {
        bytes.extend_from_slice(&record);
    }
    bytes.extend_from_slice(&cutover_record);
    bytes.extend_from_slice(&footer);
    std::fs::write(path, bytes).unwrap();
}
