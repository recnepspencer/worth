use std::collections::BTreeMap;
use std::path::PathBuf;

use worth_store_offline_verifier::RecoveryObserverReport;

use super::super::super::{child_lifecycle, history};
use super::super::evidence::snapshot_directory;
use super::super::process::{spawn_writer, wait_for_writer_ready};
use super::super::CheckpointCrashScenario;

pub(super) struct CheckpointFixture {
    pub(super) scenario_index: usize,
    pub(super) scenario: CheckpointCrashScenario,
    pub(super) stage: super::super::CheckpointCrashStage,
    pub(super) parent: tempfile::TempDir,
    pub(super) root: PathBuf,
    pub(super) operation_program: history::SubmittedOperationProgram,
    pub(super) child: child_lifecycle::ProcessChildGuard,
    pub(super) baseline_snapshot: BTreeMap<String, (u64, [u8; 32])>,
    pub(super) baseline_observer: RecoveryObserverReport,
}

pub(super) fn prepare(
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
    let operation_program = history::create_checkpoint_operation_program(
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
    wait_for_writer_ready(&mut child, &start_marker);
    let mut operation_program = operation_program;
    operation_program
        .expected
        .bind_persisted_operation_identities(&root)
        .expect("bind submitted operations from persisted authority");
    let baseline_snapshot = snapshot_directory(&root);
    let baseline_observer =
        super::super::process::fresh_observer(&parent, &root, "baseline-observer");

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
