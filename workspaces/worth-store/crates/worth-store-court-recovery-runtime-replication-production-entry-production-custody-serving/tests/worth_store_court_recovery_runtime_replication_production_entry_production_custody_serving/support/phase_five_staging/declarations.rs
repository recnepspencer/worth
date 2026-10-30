pub(crate) use {std::path::Path, std::path::PathBuf};
pub(crate) use {std::process::Command, std::process::Output};

pub(crate) use worth_proof::TransitionOutcome;
pub(crate) use worth_store_physical_format::RecordArtifactFile;
pub(crate) use worth_store_recovery_runtime::PhysicalRecoveryLimits;
pub(crate) use {
    worth_store::physical_runtime::PhysicalCheckpointDeadline,
    worth_store::physical_runtime::PhysicalCheckpointIdempotencyKey,
    worth_store::physical_runtime::PhysicalCheckpointOutcome,
    worth_store::physical_runtime::PhysicalCheckpointRequest,
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
};

pub(crate) const CHILD_ROOT: &str = "WORTH_C8_PHASE5_CHILD_ROOT";
pub(crate) const WRITER_TEST: &str =
    "process::recovery_runtime::phase_five_staging::staging_writer_process";
pub(crate) const STAGER_TEST: &str =
    "process::recovery_runtime::phase_five_staging::stager_process";
pub(crate) const CONVERGENCE_TEST: &str =
    "process::recovery_runtime::phase_five_staging::staging_convergence_process";

pub(crate) fn plan(root: &Path) -> worth_store_recovery_runtime::PlannedPhysicalRecovery {
    admitted_recovery_with_limits(root, ordinary_limits())
        .discover()
        .unwrap()
        .select()
        .unwrap()
        .plan()
        .unwrap()
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
    declaration.observation_bytes = 32 * 1024 * 1024;
    declaration.publication_effects = 64;
    worth_store_recovery_runtime::PhysicalRecoveryLimits::admit(declaration).unwrap()
}

pub(crate) fn record_artifact_path(root: &Path, artifact: RecordArtifactFile) -> PathBuf {
    assert!(
        matches!(
            artifact,
            worth_store_physical_format::RecordArtifactFile::Segment { .. }
        ),
        "Phase 5 test stages a segment data frame"
    );
    root.join("families/records")
        .join("segments")
        .join(artifact.file_name())
}

pub(crate) fn run_child(test: &str, path: &Path, temporary_root: &Path) -> Output {
    std::process::Command::new(std::env::current_exe().unwrap())
        .args(["--exact", test, "--ignored", "--nocapture"])
        .env(CHILD_ROOT, path)
        .env("TMP", temporary_root)
        .env("TEMP", temporary_root)
        .env("TMPDIR", temporary_root)
        .output()
        .expect("launch Phase 5 child process")
}

pub(crate) fn required_child_path() -> PathBuf {
    std::env::var_os(CHILD_ROOT)
        .map(std::path::PathBuf::from)
        .expect("Phase 5 child root")
}

pub(crate) fn assert_child_succeeded(name: &str, output: &Output) {
    assert!(
        output.status.success(),
        "{name} child failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}
