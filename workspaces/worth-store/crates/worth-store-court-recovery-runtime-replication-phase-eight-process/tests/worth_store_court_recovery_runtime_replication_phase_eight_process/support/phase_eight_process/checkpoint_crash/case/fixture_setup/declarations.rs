pub(crate) use {std::collections::BTreeMap};
pub(crate) use {std::path::PathBuf};

pub(crate) use {worth_store_offline_verifier::RecoveryObserverReport};

pub(crate) use {worth_store_testkit_recovery_runtime::child_lifecycle, crate::history};
pub(crate) use {crate::checkpoint_crash::evidence::snapshot::snapshot_directory};
pub(crate) use {crate::phase_eight_process::checkpoint_crash::process::writer_process::spawn_writer, crate::phase_eight_process::checkpoint_crash::process::writer_process::wait_for_writer_ready};
pub(crate) use {crate::checkpoint_crash::CheckpointCrashScenario};

pub(crate) struct CheckpointFixture {
    pub(crate) scenario_index: usize,
    pub(crate) scenario: CheckpointCrashScenario,
    pub(crate) stage: crate::checkpoint_crash::CheckpointCrashStage,
    pub(crate) parent: tempfile::TempDir,
    pub(crate) root: PathBuf,
    pub(crate) operation_program: crate::phase_eight_process::history::writer_history::submitted_program::SubmittedOperationProgram,
    pub(crate) child: worth_store_testkit_recovery_runtime::child_lifecycle::ProcessChildGuard,
    pub(crate) baseline_snapshot: BTreeMap<String, (u64, [u8; 32])>,
    pub(crate) baseline_observer: RecoveryObserverReport,
}

pub(crate) fn prepare(
    scenario_index: usize,
    scenario: CheckpointCrashScenario,
    schedule_seed: u64,
    perturbation_seed: u64,
) -> CheckpointFixture {
    let stage = scenario.stage;
    let parent = tempfile::tempdir().expect("checkpoint matrix parent");
    let root = parent.path().join(format!(
        "checkpoint-root-{scenario_index}-{}-{schedule_seed}",
        scenario.id
    ));
    let operation_program = crate::phase_eight_process::history::schedule::create_checkpoint_operation_program(
        parent.path(),
        schedule_seed,
        perturbation_seed,
    )
    .expect("write C8 submitted operation program");
    let start_marker = parent.path().join("checkpoint-start");
    let reached_marker = parent.path().join("checkpoint-reached");
    let mut child = spawn_writer(
        stage,
        schedule_seed,
        perturbation_seed,
        &root,
        &operation_program,
        &start_marker,
        &reached_marker,
    );
    wait_for_writer_ready(
        &mut child,
        &start_marker,
        operation_program.expected.payloads().len(),
    );
    let mut operation_program = operation_program;
    operation_program
        .expected
        .bind_persisted_operation_identities(&root)
        .expect("bind submitted operations from persisted authority");
    let baseline_snapshot = snapshot_directory(&root);
    let baseline_observer =
        crate::phase_eight_process::checkpoint_crash::process::observer_report::fresh_observer(&parent, &root, "baseline-observer");

    CheckpointFixture {
        scenario_index,
        scenario,
        stage,
        parent,
        root,
        operation_program,
        child,
        baseline_snapshot,
        baseline_observer,
    }
}