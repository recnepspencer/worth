#[allow(dead_code)]
mod phase_three_support;

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use phase_three_support::{admitted_recovery_with_limits, limit_declaration};
use worth_proof::TransitionOutcome;
use worth_store::physical_runtime::{
    PhysicalCheckpointDeadline, PhysicalCheckpointIdempotencyKey, PhysicalCheckpointOutcome,
    PhysicalCheckpointRequest,
};
use worth_store_physical_format::RecordArtifactFile;
use worth_store_recovery_runtime::PhysicalRecoveryLimits;
use worth_store_test_support::harness::physical_residency::{
    canonical_durable_wal_attempt_without_execution, canonical_physical_mutation_acknowledgment,
    canonical_rooted_mutation_without_acknowledgment, PhysicalResidencyStoreWorld,
};

const CHILD_ROOT: &str = "WORTH_C8_PHASE5_CHILD_ROOT";
const WRITER_TEST: &str = "phase_five_writer_process";
const STAGER_TEST: &str = "phase_five_stager_process";
const CONVERGENCE_TEST: &str = "phase_five_convergence_process";

#[test]
fn ordinary_store_state_crosses_process_death_into_closed_convergent_staging() {
    let parent = tempfile::tempdir().expect("process-boundary parent");
    let marker = parent.path().join("persisted-root");
    assert_child_succeeded("writer", &run_child(WRITER_TEST, &marker, parent.path()));
    let root = PathBuf::from(std::fs::read_to_string(&marker).expect("writer root marker"));
    assert_child_succeeded("stager", &run_child(STAGER_TEST, &root, parent.path()));
    assert_child_succeeded(
        "convergence",
        &run_child(CONVERGENCE_TEST, &root, parent.path()),
    );
}

#[test]
fn conflicting_noncurrent_artifact_blocks_without_counterfeit_performed_evidence() {
    let parent = tempfile::tempdir().expect("conflict parent");
    let marker = parent.path().join("persisted-root");
    assert_child_succeeded("writer", &run_child(WRITER_TEST, &marker, parent.path()));
    let root = PathBuf::from(std::fs::read_to_string(&marker).unwrap());
    let artifact = RecordArtifactFile::Segment {
        segment: 1,
        generation: 3,
    };
    std::fs::write(record_artifact_path(&root, artifact), vec![0_u8; 16_384]).unwrap();

    let outcome = match plan(&root).stage() {
        Ok(_) => panic!("conflicting bytes cannot converge"),
        Err(outcome) => outcome,
    };
    let worth_store_recovery_runtime::PhysicalRecoveryOutcome::Blocked(blocked) = outcome else {
        panic!("staging media conflict is Blocked")
    };
    assert_eq!(
        blocked.kind,
        worth_store_recovery_runtime::PhysicalRecoveryBlockKind::Staging
    );
    let counters = blocked.evidence().staging_counters.unwrap();
    assert_eq!(counters.commands_submitted, 1);
    assert_eq!(counters.commands_settled, 1);
    assert_eq!(counters.scheduler_settlements, 1);
    assert_eq!(counters.artifacts_created, 0);
    assert_eq!(counters.performed_effects, 0);
    assert!(matches!(
        blocked.evidence().staging_denial,
        Some(worth_store_recovery_runtime::PhysicalRecoveryStagingDenial::CommandFailed {
            ordinal: 0,
            stage: worth_store::physical_runtime::PhysicalRecoveryStagingCommandStage::Materialization,
        })
    ));
    assert!(matches!(
        blocked.evidence().staging_settlements.as_ref().unwrap().entries(),
        [worth_store_recovery_runtime::PhysicalRecoveryStagingSettlement::DeniedBeforeEffect(denial)]
            if matches!(denial.denial(), worth_store::physical_runtime::PhysicalRecoveryStagingCommandDenialKind::Media(_))
                && denial.scheduler_posture() == Some(worth_store::physical_runtime::PhysicalWorkSchedulerPosture::Executed)
    ));
    assert_eq!(blocked.recovery_effects(), 0);
}

#[test]
#[ignore = "launched by the process-boundary parent"]
fn phase_five_writer_process() {
    let marker = required_child_path();
    let world = PhysicalResidencyStoreWorld::initialize_for_recovery("c8-phase5-process").unwrap();
    let retained_root = world.retained_root();
    canonical_physical_mutation_acknowledgment(&world, [0x51; 32], b"ordinary-c8-stage");
    let request = PhysicalCheckpointRequest::fuzzy(
        PhysicalCheckpointIdempotencyKey::new([0x52; 32]),
        PhysicalCheckpointDeadline::after_milliseconds(5_000).unwrap(),
    );
    let TransitionOutcome::Success(handle) =
        world.serving().checkpoints().start(request).into_raw()
    else {
        panic!("ordinary checkpoint admission must succeed")
    };
    assert!(matches!(
        handle.wait(),
        PhysicalCheckpointOutcome::Completed(_)
    ));
    canonical_rooted_mutation_without_acknowledgment(&world, [0x53; 32], b"rooted-c8-stage");
    canonical_durable_wal_attempt_without_execution(&world, [0x54; 32], b"wal-only-c8-stage");
    drop(world);
    let root = retained_root.persist();
    std::fs::write(marker, root.to_string_lossy().as_bytes()).expect("persisted root marker");
}

#[test]
#[ignore = "launched by the process-boundary parent"]
fn phase_five_stager_process() {
    let root = required_child_path();
    let current_selector =
        std::fs::read(root.join("families/records/root-current.selector")).unwrap();
    let previous_selector =
        std::fs::read(root.join("families/records/root-previous.selector")).unwrap();
    let planned = plan(&root);
    let action = &planned.staging_layout().actions()[0];
    let artifact = action.source().artifact();
    let expected = planned
        .staging_layout()
        .commands()
        .iter()
        .find(|command| command.artifact() == artifact)
        .expect("target artifact command")
        .bytes()
        .to_vec();
    let artifact_path = record_artifact_path(&root, artifact);
    let expected_generation = planned.staging_layout().staging_generation();
    let staged = planned.stage().expect("planned recovery stages");
    assert_eq!(staged.closed_generation().generation(), expected_generation);
    assert_eq!(staged.closed_generation().artifact_count(), 1);
    assert_eq!(
        staged.closed_generation().byte_count(),
        expected.len() as u64
    );
    assert_ne!(staged.closed_generation().content_identity(), [0; 32]);
    assert_eq!(staged.staging_counters().planned_scheduler_commands, 2);
    assert_eq!(staged.staging_counters().commands_submitted, 2);
    assert_eq!(staged.staging_counters().commands_settled, 2);
    assert_eq!(staged.staging_counters().scheduler_settlements, 2);
    assert_eq!(staged.staging_counters().artifacts_created, 1);
    assert_eq!(staged.staging_counters().artifacts_synchronized, 1);
    assert_eq!(staged.staging_counters().performed_effects, 2);
    assert_eq!(staged.staging_settlements().completed(), 1);
    assert!(staged.is_quiescent());
    assert_eq!(std::fs::read(&artifact_path).unwrap(), expected);
    assert_eq!(
        std::fs::read(root.join("families/records/root-current.selector")).unwrap(),
        current_selector
    );
    assert_eq!(
        std::fs::read(root.join("families/records/root-previous.selector")).unwrap(),
        previous_selector
    );
    let cancelled = staged.cancel_before_publication();
    let worth_store_recovery_runtime::PhysicalRecoveryOutcome::Blocked(cancelled) = cancelled
    else {
        panic!("post-staging cancellation retains escaped effects")
    };
    assert!(matches!(
        cancelled.evidence().staging_denial,
        Some(worth_store_recovery_runtime::PhysicalRecoveryStagingDenial::CancelledAfterClosedStaging)
    ));
    assert_eq!(
        cancelled
            .evidence()
            .staging_settlements
            .as_ref()
            .unwrap()
            .completed(),
        1
    );
    assert!(cancelled.recovery_effects() >= 2);
}

#[test]
#[ignore = "launched by the process-boundary parent"]
fn phase_five_convergence_process() {
    let root = required_child_path();
    let planned = plan(&root);
    let expected_bytes: u64 = planned
        .staging_layout()
        .commands()
        .iter()
        .map(|command| command.byte_count())
        .sum();
    let converged = planned.stage().expect("identical staging converges");
    assert_eq!(converged.staging_counters().artifacts_created, 0);
    assert_eq!(converged.staging_counters().artifacts_converged, 1);
    assert_eq!(converged.staging_counters().artifacts_synchronized, 1);
    assert_eq!(converged.staging_counters().performed_effects, 1);
    assert_eq!(converged.staging_counters().bytes_verified, expected_bytes);
    assert!(converged.is_quiescent());
    assert!(matches!(
        converged.cancel_before_publication(),
        worth_store_recovery_runtime::PhysicalRecoveryOutcome::Blocked(_)
    ));
}

fn plan(root: &Path) -> worth_store_recovery_runtime::PlannedPhysicalRecovery {
    admitted_recovery_with_limits(root, ordinary_limits())
        .discover()
        .unwrap()
        .select()
        .unwrap()
        .plan()
        .unwrap()
}

fn ordinary_limits() -> PhysicalRecoveryLimits {
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
    PhysicalRecoveryLimits::admit(declaration).unwrap()
}

fn record_artifact_path(root: &Path, artifact: RecordArtifactFile) -> PathBuf {
    let family = match artifact {
        RecordArtifactFile::Segment { .. } => "segments",
        RecordArtifactFile::Extent { .. } => "extents",
        _ => panic!("Phase 5 test stages a data frame"),
    };
    root.join("families/records")
        .join(family)
        .join(artifact.file_name())
}

fn run_child(test: &str, path: &Path, temporary_root: &Path) -> Output {
    Command::new(std::env::current_exe().unwrap())
        .args(["--exact", test, "--ignored", "--nocapture"])
        .env(CHILD_ROOT, path)
        .env("TMP", temporary_root)
        .env("TEMP", temporary_root)
        .env("TMPDIR", temporary_root)
        .output()
        .expect("launch Phase 5 child process")
}

fn required_child_path() -> PathBuf {
    std::env::var_os(CHILD_ROOT)
        .map(PathBuf::from)
        .expect("Phase 5 child root")
}

fn assert_child_succeeded(name: &str, output: &Output) {
    assert!(
        output.status.success(),
        "{name} child failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}
