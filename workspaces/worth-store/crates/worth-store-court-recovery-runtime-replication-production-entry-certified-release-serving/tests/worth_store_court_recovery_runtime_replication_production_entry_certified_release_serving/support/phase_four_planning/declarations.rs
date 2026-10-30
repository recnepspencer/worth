pub(crate) use std::num::NonZeroU64;
pub(crate) use {std::path::Path, std::path::PathBuf};

pub(crate) use worth_proof::TransitionOutcome;
pub(crate) use {
    worth_store::physical_runtime::PhysicalCheckpointDeadline,
    worth_store::physical_runtime::PhysicalCheckpointIdempotencyKey,
    worth_store::physical_runtime::PhysicalCheckpointOutcome,
    worth_store::physical_runtime::PhysicalCheckpointRequest,
    worth_store::physical_runtime::StoreRecoveryBindingSampleDenial,
};
pub(crate) use {
    worth_store_physical_format::encode_data_frame_page_lsn,
    worth_store_physical_format::CheckpointBindingCompactionHeader,
    worth_store_physical_format::CheckpointRootBasis,
    worth_store_physical_format::CheckpointStreamEncoder,
    worth_store_physical_format::CheckpointWalSourceRange,
    worth_store_physical_format::DurableFrameKind,
    worth_store_physical_format::PhysicalCheckpointIdentity,
    worth_store_physical_format::PhysicalCheckpointSource,
    worth_store_physical_format::PhysicalPageLsn, worth_store_physical_format::RecordArtifactFile,
};
pub(crate) use {
    worth_store_recovery_physics::PhysicalRedoDecisionKind,
    worth_store_recovery_physics::PhysicalRedoPlanningDenial,
    worth_store_recovery_physics::PhysicalRedoTargetIdentity,
    worth_store_recovery_physics::RecoveryOperationFate,
    worth_store_recovery_physics::RecoveryPlanCostDenial,
};
pub(crate) use {
    worth_store_recovery_runtime::PhysicalRecoveryLimitDeclaration,
    worth_store_recovery_runtime::PhysicalRecoveryLimitDimension,
    worth_store_recovery_runtime::PhysicalRecoveryOutcome,
    worth_store_recovery_runtime::PhysicalRecoveryPlanningDenial,
};
pub(crate) use {
    worth_store_recovery_runtime::PhysicalRecoveryLimits,
    worth_store_testkit_recovery_runtime::phase_three_support::admitted_recovery,
    worth_store_testkit_recovery_runtime::phase_three_support::admitted_recovery_with_limits,
    worth_store_testkit_recovery_runtime::phase_three_support::expect_blocked,
    worth_store_testkit_recovery_runtime::phase_three_support::initialize_store,
    worth_store_testkit_recovery_runtime::phase_three_support::limit_declaration,
    worth_store_testkit_recovery_runtime::phase_three_support::publish_synthetic_checkpoint,
    worth_store_testkit_recovery_runtime::phase_three_support::publish_synthetic_covered_wal,
    worth_store_testkit_recovery_runtime::phase_three_support::publish_synthetic_genesis,
};
pub(crate) use {
    worth_store_test_support::harness::physical_residency::canonical_durable_wal_attempt_without_execution,
    worth_store_test_support::harness::physical_residency::canonical_physical_mutation_acknowledgment,
    worth_store_test_support::harness::physical_residency::canonical_rooted_mutation_without_acknowledgment,
    worth_store_test_support::harness::physical_residency::PhysicalResidencyStoreWorld,
};

pub(crate) fn publish_secured_synthetic_checkpoint(
    root: &Path,
    store: worth_store_physical_format::store_namespace::StableStoreIdentity,
) -> PhysicalCheckpointIdentity {
    let checkpoint = worth_store_physical_format::PhysicalCheckpointIdentity::new(
        store,
        std::num::NonZeroU64::new(1).unwrap(),
    );
    let source = worth_store_physical_format::PhysicalCheckpointSource::secured_concurrent(
        checkpoint,
        worth_store_physical_format::CheckpointWalSourceRange::new(1, 2).unwrap(),
        worth_store_physical_format::CheckpointRootBasis::new(1, 7),
        1,
        [7; 32],
        8,
    )
    .unwrap();
    let (encoder, header) = worth_store_physical_format::CheckpointStreamEncoder::begin(source);
    let cutover =
        worth_store_physical_format::CheckpointBindingCompactionHeader::new(1, 2).unwrap();
    let (compaction, cutover_record) = encoder.begin_binding_compaction(cutover);
    let (_, footer) = compaction.finish();
    let mut bytes = Vec::new();
    bytes.extend_from_slice(&header);
    bytes.extend_from_slice(&cutover_record);
    bytes.extend_from_slice(&footer);
    std::fs::write(root.join("families").join("checkpoint.current"), bytes).unwrap();
    checkpoint
}

pub(crate) fn prepare_ordinary_recovery_root(
    name: &str,
) -> worth_store_test_support::TemporaryDirectory {
    let world = worth_store_test_support::harness::physical_residency::PhysicalResidencyStoreWorld::initialize_for_recovery(name).unwrap();
    let retained_root = world.retained_root();
    let acknowledgment =
        canonical_physical_mutation_acknowledgment(&world, [0x41; 32], b"ordinary-c8-redo");
    assert_ne!(acknowledgment.request_fingerprint().bytes(), [0; 32]);
    let checkpoint_request = worth_store::physical_runtime::PhysicalCheckpointRequest::fuzzy(
        worth_store::physical_runtime::PhysicalCheckpointIdempotencyKey::new([0x42; 32]),
        worth_store::physical_runtime::PhysicalCheckpointDeadline::after_milliseconds(5_000)
            .unwrap(),
    );
    let worth_proof::TransitionOutcome::Success(handle) = world
        .serving()
        .checkpoints()
        .start(checkpoint_request)
        .into_raw()
    else {
        panic!("ordinary checkpoint admission must succeed")
    };
    let checkpoint = match handle.wait() {
        worth_store::physical_runtime::PhysicalCheckpointOutcome::Completed(checkpoint) => {
            checkpoint
        }
        other => panic!("ordinary checkpoint publication must complete: {other:?}"),
    };
    assert!(checkpoint.retained_wal_tail().segment_count().get() > 0);
    canonical_rooted_mutation_without_acknowledgment(&world, [0x43; 32], b"rooted-c8-redo");
    canonical_durable_wal_attempt_without_execution(&world, [0x44; 32], b"post-checkpoint-c8-redo");
    drop(world);
    retained_root
}

pub(crate) fn ordinary_recovery_declaration(
    manifest_entries: u64,
) -> PhysicalRecoveryLimitDeclaration {
    let mut declaration = limit_declaration(2, 8, 2 * 1024 * 1024);
    declaration.manifest_entries = manifest_entries;
    declaration.wal_bytes = 2 * 1024 * 1024;
    declaration.redo_targets = 4_096;
    declaration.redo_bytes = 4 * 1024 * 1024;
    declaration.distinct_pages_and_extents = 4_096;
    declaration.operation_bindings = 4_096;
    declaration.staging_bytes = 32 * 1024 * 1024;
    declaration.recovery_memory_bytes = 32 * 1024 * 1024;
    declaration.dirty_frames = 4_096;
    declaration.publication_effects = 64;
    declaration.observation_bytes = 32 * 1024 * 1024;
    declaration
}

pub(crate) fn ordinary_recovery_limits(
    manifest_entries: u64,
) -> worth_store_recovery_runtime::PhysicalRecoveryLimits {
    worth_store_recovery_runtime::PhysicalRecoveryLimits::admit(ordinary_recovery_declaration(
        manifest_entries,
    ))
    .unwrap()
}

pub(crate) fn selected_ordinary_recovery(
    root: &Path,
) -> worth_store_recovery_runtime::SelectedPhysicalRecovery {
    admitted_recovery_with_limits(root, ordinary_recovery_limits(4_096))
        .discover()
        .unwrap()
        .select()
        .unwrap()
}

pub(crate) fn only_artifact(directory: PathBuf, predicate: impl Fn(&str) -> bool) -> PathBuf {
    let matches = std::fs::read_dir(directory)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .is_some_and(&predicate)
        })
        .collect::<Vec<_>>();
    assert_eq!(
        matches.len(),
        1,
        "fixture must identify one governed artifact: {matches:?}"
    );
    matches.into_iter().next().unwrap()
}
