pub(crate) use {std::path::Path, std::path::PathBuf};
pub(crate) use {std::process::Command, std::process::Output};

pub(crate) use worth_proof::TransitionOutcome;
pub(crate) use worth_store_physical_format::RecordArtifactFile;
pub(crate) use {
    worth_store::physical_runtime::recovery_wal::WalSegmentArtifactIdentity,
    worth_store::physical_runtime::PhysicalCheckpointDeadline,
    worth_store::physical_runtime::PhysicalCheckpointIdempotencyKey,
    worth_store::physical_runtime::PhysicalCheckpointOutcome,
    worth_store::physical_runtime::PhysicalCheckpointRequest,
};
pub(crate) use {
    worth_store_recovery_physics::PhysicalRedoDecisionKind,
    worth_store_recovery_physics::PhysicalRedoDecisionPrior,
    worth_store_recovery_physics::PhysicalRedoTargetIdentity,
    worth_store_recovery_physics::RecoveryOperationFate,
    worth_store_recovery_physics::RecoveryPageSource,
};
pub(crate) use {
    worth_store_recovery_runtime::PhysicalRecoveryLimits,
    worth_store_recovery_runtime::PhysicalRecoveryOutcome,
    worth_store_recovery_runtime::RecoveryPublicationAction,
    worth_store_recovery_runtime::WorthStoreRecovery,
};
pub(crate) use {
    worth_store_test_support::harness::physical_residency::canonical_durable_wal_attempt_without_execution,
    worth_store_test_support::harness::physical_residency::canonical_physical_mutation_acknowledgment,
    worth_store_test_support::harness::physical_residency::canonical_rooted_mutation_without_acknowledgment,
    worth_store_test_support::harness::physical_residency::PhysicalResidencyStoreWorld,
};
pub(crate) use {
    worth_store_testkit_recovery_runtime::phase_three_support::admitted_recovery_with_limits,
    worth_store_testkit_recovery_runtime::phase_three_support::limit_declaration,
    worth_store_testkit_recovery_runtime::phase_three_support::recovery_request_with_limits,
};

pub(crate) const CHILD_ROOT: &str = "WORTH_C8_PHASE4_CHILD_ROOT";
pub(crate) const WRITER_TEST: &str =
    "process::recovery_runtime::phase_four_process::planning_writer_process";
pub(crate) const PLANNER_TEST: &str =
    "process::recovery_runtime::phase_four_process::planner_process";
pub(crate) const FACADE_TEST: &str =
    "process::recovery_runtime::phase_four_process::recovery_facade_process";

pub(crate) fn append_torn_terminal_wal_segment(root: &Path) {
    let wal = root.join("families").join("wal");
    let latest = std::fs::read_dir(&wal)
        .unwrap()
        .filter_map(Result::ok)
        .filter_map(|entry| {
            let name = entry.file_name().to_string_lossy().into_owned();
            worth_store::physical_runtime::recovery_wal::WalSegmentArtifactIdentity::parse(&name)
                .map(|identity| (identity, entry.path()))
        })
        .max_by_key(|(identity, _)| *identity)
        .unwrap();
    let next_segment = latest.0.segment().get() + 1;
    let (path, frame) =
        worth_store_test_support::harness::recovery::wal_tail::prepare_persisted_wal_frame(
            &root.join("families"),
            next_segment,
            0,
            1,
            "terminal-torn-observation",
            b"not-owner-visible",
        );
    std::fs::write(path, &frame[..37]).unwrap();
}

pub(crate) fn ordinary_limits() -> PhysicalRecoveryLimits {
    let mut declaration = limit_declaration(2, 8, 2 * 1024 * 1024);
    declaration.manifest_entries = 4_096;
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
    worth_store_recovery_runtime::PhysicalRecoveryLimits::admit(declaration).unwrap()
}

pub(crate) fn run_child(test: &str, path: &Path, temporary_root: &Path) -> Output {
    let mut command =
        std::process::Command::new(std::env::current_exe().expect("integration test executable"));
    command
        .args(["--exact", test, "--ignored", "--nocapture"])
        .env(CHILD_ROOT, path)
        .env("TMP", temporary_root)
        .env("TEMP", temporary_root)
        .env("TMPDIR", temporary_root);
    command.output().expect("launch Phase 4 child process")
}

pub(crate) fn required_child_path() -> PathBuf {
    std::env::var_os(CHILD_ROOT)
        .map(std::path::PathBuf::from)
        .expect("Phase 4 child root")
}

pub(crate) fn assert_child_succeeded(name: &str, output: &Output) {
    assert!(
        output.status.success(),
        "{name} child failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );
}
