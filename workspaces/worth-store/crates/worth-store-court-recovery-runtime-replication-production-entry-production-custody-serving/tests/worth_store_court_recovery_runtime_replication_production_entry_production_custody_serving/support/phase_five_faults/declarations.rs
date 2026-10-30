#![cfg(all())]

pub(crate) use {std::path::Path, std::path::PathBuf};

pub(crate) use worth_proof::TransitionOutcome;
pub(crate) use {
    worth_store::physical_runtime::certification::MediaFaultDirective,
    worth_store::physical_runtime::certification::MediaFaultSchedule,
    worth_store::physical_runtime::certification::MediaOperationRole,
};
pub(crate) use {
    worth_store::physical_runtime::FilesystemAccessPosture,
    worth_store::physical_runtime::FilesystemMediaAdmission,
    worth_store::physical_runtime::PhysicalCheckpointDeadline,
    worth_store::physical_runtime::PhysicalCheckpointIdempotencyKey,
    worth_store::physical_runtime::PhysicalCheckpointOutcome,
    worth_store::physical_runtime::PhysicalCheckpointRequest,
    worth_store::physical_runtime::RecoveryStagingIndeterminatePhysical,
};
pub(crate) use {
    worth_store_recovery_runtime::PhysicalRecoveryLimitDeclaration,
    worth_store_recovery_runtime::PhysicalRecoveryLimits,
    worth_store_recovery_runtime::PhysicalRecoveryOpenRequest,
    worth_store_recovery_runtime::PhysicalRecoveryPlatformAuthority,
    worth_store_recovery_runtime::PhysicalRecoveryStaticConfiguration,
};
pub(crate) use {
    worth_store_test_support::harness::physical_residency::canonical_durable_wal_attempt_without_execution,
    worth_store_test_support::harness::physical_residency::canonical_physical_mutation_acknowledgment,
    worth_store_test_support::harness::physical_residency::PhysicalResidencyStoreWorld,
};

pub(crate) fn assert_signal_derivation_failure(
    expected_stage: worth_store::physical_runtime::PhysicalRecoveryStagingCommandStage,
) {
    let label = match expected_stage {
        worth_store::physical_runtime::PhysicalRecoveryStagingCommandStage::Materialization => {
            "signal-derivation-materialization"
        }
        worth_store::physical_runtime::PhysicalRecoveryStagingCommandStage::Synchronization => {
            "signal-derivation-synchronization"
        }
    };
    let (parent, root) = ordinary_persisted_world(label);
    let planned = plan_with_schedule(&root, empty_recovery_fault_schedule());
    planned.certification_fail_staging_signal_settlement_at(expected_stage);
    let blocked = expect_staging_block(planned.stage());
    let counters = blocked.evidence().staging_counters.unwrap();
    assert_eq!(
        counters.commands_submitted,
        match expected_stage {
            worth_store::physical_runtime::PhysicalRecoveryStagingCommandStage::Materialization =>
                1,
            worth_store::physical_runtime::PhysicalRecoveryStagingCommandStage::Synchronization =>
                2,
        },
        "the exact physical stages completed before Signal derivation failed",
    );
    assert_eq!(counters.commands_settled, counters.commands_submitted);
    assert_eq!(counters.scheduler_settlements, counters.commands_submitted);
    assert_eq!(
        counters.performed_effects,
        u64::from(
            expected_stage
                == worth_store::physical_runtime::PhysicalRecoveryStagingCommandStage::Synchronization
        ),
        "only the earlier terminal materialization may carry performed authority",
    );
    assert_eq!(
        (
            counters.pending_signal_reconciliations_after_close,
            counters.signal_reconciliation_overflow_after_close,
        ),
        (1, 0),
    );
    let [worth_store_recovery_runtime::PhysicalRecoveryStagingSettlement::Indeterminate(
        worth_store::physical_runtime::PhysicalRecoveryStagingCommandIndeterminate::Signal {
            stage,
            materialization,
            synchronization,
            outcome,
        },
    )] = blocked
        .evidence()
        .staging_settlements
        .as_ref()
        .unwrap()
        .entries()
    else {
        panic!("derived-state loss retains one exact Signal settlement")
    };
    assert_eq!(*stage, expected_stage);
    assert_eq!(
        *outcome,
        worth_store::physical_runtime::PhysicalSignalSettlementOutcome::DerivedStateUnavailable,
    );
    let materialization = materialization
        .as_ref()
        .expect("physical materialization truth is retained");
    assert_eq!(
        materialization.is_performed(),
        expected_stage
            == worth_store::physical_runtime::PhysicalRecoveryStagingCommandStage::Synchronization,
        "the failing stage cannot mint performed authority",
    );
    assert_eq!(
        synchronization.is_some(),
        expected_stage
            == worth_store::physical_runtime::PhysicalRecoveryStagingCommandStage::Synchronization,
    );
    assert!(matches!(
        blocked.evidence().staging_denial,
        Some(worth_store_recovery_runtime::PhysicalRecoveryStagingDenial::Indeterminate {
            stage,
            ..
        }) if stage == expected_stage
    ));
    drop(parent);
}

pub(crate) fn ordinary_persisted_world(label: &str) -> (tempfile::TempDir, PathBuf) {
    let parent = tempfile::tempdir().expect("fault world parent");
    let world = worth_store_test_support::harness::physical_residency::PhysicalResidencyStoreWorld::initialize_for_recovery(label).unwrap();
    let retained_root = world.retained_root();
    canonical_physical_mutation_acknowledgment(&world, [0x61; 32], b"fault-base");
    let request = worth_store::physical_runtime::PhysicalCheckpointRequest::fuzzy(
        worth_store::physical_runtime::PhysicalCheckpointIdempotencyKey::new([0x62; 32]),
        worth_store::physical_runtime::PhysicalCheckpointDeadline::after_milliseconds(5_000)
            .unwrap(),
    );
    let worth_proof::TransitionOutcome::Success(handle) =
        world.serving().checkpoints().start(request).into_raw()
    else {
        panic!("fault-world checkpoint admission")
    };
    assert!(matches!(
        handle.wait(),
        worth_store::physical_runtime::PhysicalCheckpointOutcome::Completed(_)
    ));
    canonical_durable_wal_attempt_without_execution(&world, [0x63; 32], b"fault-redo");
    drop(world);
    let copied = parent.path().join("retained-root");
    copy_directory(&retained_root.persist(), &copied);
    (parent, copied)
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
        .expect("valid recovery fault schedule")
}

pub(crate) fn empty_recovery_fault_schedule() -> MediaFaultSchedule {
    let admission = worth_store::physical_runtime::FilesystemMediaAdmission::certification(
        worth_store::physical_runtime::FilesystemAccessPosture::CoordinatedServiceAccount,
    );
    admission
        .fault_schedule_authority()
        .schedule(Vec::new())
        .expect("empty recovery fault schedule")
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

pub(crate) fn expect_staging_block(
    outcome: Result<
        worth_store_recovery_runtime::StagedPhysicalRecovery,
        worth_store_recovery_runtime::PhysicalRecoveryOutcome,
    >,
) -> worth_store_recovery_runtime::PhysicalRecoveryBlock {
    let Err(worth_store_recovery_runtime::PhysicalRecoveryOutcome::Blocked(blocked)) = outcome
    else {
        panic!("faulted staging terminates Blocked")
    };
    *blocked
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
