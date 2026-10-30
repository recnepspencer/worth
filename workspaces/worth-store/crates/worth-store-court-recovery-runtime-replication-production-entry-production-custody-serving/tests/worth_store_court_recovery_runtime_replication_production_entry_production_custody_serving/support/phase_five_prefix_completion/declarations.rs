#![cfg(all())]

pub(crate) use {std::path::Path, std::path::PathBuf};

pub(crate) use worth_proof::TransitionOutcome;
pub(crate) use {
    worth_store::physical_runtime::certification::MediaFaultDirective,
    worth_store::physical_runtime::certification::MediaFaultSchedule,
    worth_store::physical_runtime::certification::MediaOperationRole,
};
pub(crate) use {
    worth_store::physical_runtime::ArtifactTreeFailureKind,
    worth_store::physical_runtime::PhysicalCheckpointDeadline,
    worth_store::physical_runtime::PhysicalCheckpointIdempotencyKey,
    worth_store::physical_runtime::PhysicalCheckpointOutcome,
    worth_store::physical_runtime::PhysicalCheckpointRequest,
    worth_store::physical_runtime::PhysicalRecoveryStagingCommandIndeterminate,
    worth_store::physical_runtime::RecoveryStagingIndeterminatePhysical,
};
pub(crate) use {
    worth_store_recovery_runtime::PhysicalRecoveryLimitDeclaration,
    worth_store_recovery_runtime::PhysicalRecoveryLimits,
    worth_store_recovery_runtime::PhysicalRecoveryOpenRequest,
    worth_store_recovery_runtime::PhysicalRecoveryPlatformAuthority,
    worth_store_recovery_runtime::PhysicalRecoveryStagingSettlement,
    worth_store_recovery_runtime::PhysicalRecoveryStaticConfiguration,
};
pub(crate) use {
    worth_store_test_support::harness::physical_residency::canonical_durable_wal_attempt_without_execution,
    worth_store_test_support::harness::physical_residency::canonical_physical_mutation_acknowledgment,
    worth_store_test_support::harness::physical_residency::PhysicalResidencyStoreWorld,
};

pub(crate) fn assert_existing_artifact_is_damaged(
    label: &str,
    bytes: impl FnOnce(&[u8]) -> Vec<u8>,
) {
    let (parent, root) = ordinary_persisted_world(label);
    let planned = plan_with_schedule(&root, empty_fault_schedule());
    let (artifact, expected) = only_command(&planned);
    std::fs::write(record_artifact_path(&root, artifact), bytes(&expected)).unwrap();

    let Err(worth_store_recovery_runtime::PhysicalRecoveryOutcome::Blocked(blocked)) =
        planned.stage()
    else {
        panic!("conflicting existing artifact must block")
    };
    assert!(matches!(
        blocked.evidence().staging_settlements.as_ref().unwrap().entries(),
        [worth_store_recovery_runtime::PhysicalRecoveryStagingSettlement::DeniedBeforeEffect(denial)]
            if matches!(
                denial.denial(),
                worth_store::physical_runtime::PhysicalRecoveryStagingCommandDenialKind::Media(
                    failure
                ) if failure.kind() == worth_store::physical_runtime::ArtifactTreeFailureKind::Damaged
            )
    ));
    assert_eq!(blocked.recovery_effects(), 0);
    drop(parent);
}

pub(crate) fn ordinary_persisted_world(label: &str) -> (tempfile::TempDir, PathBuf) {
    let parent = tempfile::tempdir().expect("prefix world parent");
    let world = worth_store_test_support::harness::physical_residency::PhysicalResidencyStoreWorld::initialize_for_recovery(label).unwrap();
    let retained_root = world.retained_root();
    canonical_physical_mutation_acknowledgment(&world, [0x81; 32], b"prefix-base");
    let request = worth_store::physical_runtime::PhysicalCheckpointRequest::fuzzy(
        worth_store::physical_runtime::PhysicalCheckpointIdempotencyKey::new([0x83; 32]),
        worth_store::physical_runtime::PhysicalCheckpointDeadline::after_milliseconds(5_000)
            .unwrap(),
    );
    let worth_proof::TransitionOutcome::Success(handle) =
        world.serving().checkpoints().start(request).into_raw()
    else {
        panic!("prefix proof checkpoint admission")
    };
    assert!(matches!(
        handle.wait(),
        worth_store::physical_runtime::PhysicalCheckpointOutcome::Completed(_)
    ));
    canonical_durable_wal_attempt_without_execution(&world, [0x82; 32], b"prefix-redo");
    drop(world);
    let copied = parent.path().join("retained-root");
    copy_directory(&retained_root.persist(), &copied);
    (parent, copied)
}

pub(crate) fn plan_with_schedule(
    root: &Path,
    schedule: MediaFaultSchedule,
) -> worth_store_recovery_runtime::PlannedPhysicalRecovery {
    let limits = ordinary_limits();
    let configuration =
        worth_store_recovery_runtime::PhysicalRecoveryStaticConfiguration::current();
    let authority =
        worth_store_recovery_runtime::PhysicalRecoveryPlatformAuthority::acquire_for_certification(
            root.to_path_buf(),
            configuration.clone(),
            limits,
            schedule,
        )
        .expect("certification recovery authority");
    let profile = authority.qualified_backend_profile().clone();
    worth_store_recovery_runtime::PhysicalRecoveryOpenRequest::declare(
        root.to_path_buf(),
        configuration,
        profile,
        limits,
        authority,
    )
    .admit()
    .unwrap()
    .discover()
    .unwrap()
    .select()
    .unwrap()
    .plan()
    .unwrap()
}

pub(crate) fn only_command(
    planned: &worth_store_recovery_runtime::PlannedPhysicalRecovery,
) -> (worth_store_physical_format::RecordArtifactFile, Vec<u8>) {
    assert_eq!(planned.staging_layout().commands().len(), 1);
    let command = &planned.staging_layout().commands()[0];
    (command.artifact(), command.bytes().to_vec())
}

pub(crate) fn empty_fault_schedule() -> MediaFaultSchedule {
    let admission = worth_store::physical_runtime::FilesystemMediaAdmission::certification(
        worth_store::physical_runtime::FilesystemAccessPosture::CoordinatedServiceAccount,
    );
    admission
        .fault_schedule_authority()
        .schedule(Vec::new())
        .unwrap()
}

pub(crate) fn recovery_fault_schedule(
    role: MediaOperationRole,
    directive: MediaFaultDirective,
) -> MediaFaultSchedule {
    let admission = worth_store::physical_runtime::FilesystemMediaAdmission::certification(
        worth_store::physical_runtime::FilesystemAccessPosture::CoordinatedServiceAccount,
    );
    let authority = admission.fault_schedule_authority();
    authority
        .schedule(vec![authority.rule(role, 1, directive)])
        .unwrap()
}

pub(crate) fn ordinary_limits() -> PhysicalRecoveryLimits {
    worth_store_recovery_runtime::PhysicalRecoveryLimits::admit(PhysicalRecoveryLimitDeclaration {
        selector_candidates: 2,
        checkpoint_candidates: 8,
        manifest_bytes: 2 * 1024 * 1024,
        manifest_entries: 4_096,
        wal_segments: 8,
        wal_frames: 4_096,
        wal_bytes: 2 * 1024 * 1024,
        redo_targets: 4_096,
        redo_bytes: 4 * 1024 * 1024,
        distinct_pages_and_extents: 4_096,
        operation_bindings: 4_096,
        staging_bytes: 32 * 1024 * 1024,
        recovery_memory_bytes: 32 * 1024 * 1024,
        dirty_frames: 4_096,
        concurrent_commands: 8,
        publication_effects: 64,
        cleanup_candidates: 4_096,
        cleanup_bytes: 2 * 1024 * 1024,
        observation_bytes: 32 * 1024 * 1024,
    })
    .unwrap()
}

pub(crate) fn record_artifact_path(
    root: &Path,
    artifact: worth_store_physical_format::RecordArtifactFile,
) -> PathBuf {
    assert!(
        matches!(
            artifact,
            worth_store_physical_format::RecordArtifactFile::Segment { .. }
        ),
        "prefix proof stages a segment data artifact"
    );
    root.join("families/records")
        .join("segments")
        .join(artifact.file_name())
}

pub(crate) fn copy_directory(source: &Path, destination: &Path) {
    std::fs::create_dir_all(destination).unwrap();
    for entry in std::fs::read_dir(source).unwrap() {
        let entry = entry.unwrap();
        let target = destination.join(entry.file_name());
        if entry.file_type().unwrap().is_dir() {
            copy_directory(&entry.path(), &target);
        } else {
            std::fs::copy(entry.path(), target).unwrap();
        }
    }
}
