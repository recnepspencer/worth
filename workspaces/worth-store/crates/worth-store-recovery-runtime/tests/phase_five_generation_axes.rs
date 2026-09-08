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
use worth_store_recovery_physics::{PhysicalRedoDecisionKind, PhysicalRedoTargetIdentity};
use worth_store_recovery_runtime::PhysicalRecoveryLimits;
use worth_store_test_support::harness::physical_residency::{
    canonical_durable_wal_attempt_without_execution, canonical_physical_batch_acknowledgment,
    PhysicalResidencyStoreWorld,
};

const CHILD_ROOT: &str = "WORTH_C8_PHASE5_GENERATION_AXES_ROOT";
const WRITER_TEST: &str = "phase_five_generation_axes_writer";
const STAGER_TEST: &str = "phase_five_generation_axes_stager";
const CONVERGENCE_TEST: &str = "phase_five_generation_axes_convergence";

#[test]
fn new_page_in_reused_segment_crosses_process_death_and_stages_exactly() {
    let parent = tempfile::tempdir().expect("generation-axis parent");
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
fn repeated_post_checkpoint_page_reuse_crosses_process_death_without_redo() {
    let parent = tempfile::tempdir().expect("repeated page reuse parent");
    let marker = parent.path().join("persisted-root");
    assert_child_succeeded(
        "repeated page reuse writer",
        &run_child(
            "phase_five_repeated_page_reuse_writer",
            &marker,
            parent.path(),
        ),
    );
    let root = PathBuf::from(std::fs::read_to_string(&marker).expect("writer root marker"));
    assert_child_succeeded(
        "repeated page reuse planner",
        &run_child(
            "phase_five_repeated_page_reuse_planner",
            &root,
            parent.path(),
        ),
    );
}

#[test]
#[ignore = "launched by the repeated page reuse parent"]
fn phase_five_repeated_page_reuse_writer() {
    let marker = required_child_path();
    let world = PhysicalResidencyStoreWorld::initialize_for_recovery_with_segment_pages(
        "c8-phase5-repeated-page-reuse",
        4,
    )
    .unwrap();
    let retained_root = world.retained_root();
    let base = [
        vec![1_u8; 3_000],
        vec![2_u8; 3_000],
        vec![3_u8; 3_000],
        vec![4_u8; 3_000],
    ];
    canonical_physical_batch_acknowledgment(&world, [0x81; 32], base.iter().map(Vec::as_slice));
    let request = PhysicalCheckpointRequest::fuzzy(
        PhysicalCheckpointIdempotencyKey::new([0x82; 32]),
        PhysicalCheckpointDeadline::after_milliseconds(5_000).unwrap(),
    );
    let TransitionOutcome::Success(handle) =
        world.serving().checkpoints().start(request).into_raw()
    else {
        panic!("repeated page reuse checkpoint admission")
    };
    assert!(matches!(
        handle.wait(),
        PhysicalCheckpointOutcome::Completed(_)
    ));
    canonical_physical_batch_acknowledgment(&world, [0x83; 32], [&vec![5_u8; 3_000][..]]);
    canonical_physical_batch_acknowledgment(&world, [0x84; 32], [&vec![6_u8; 3_000][..]]);
    drop(world);
    let root = retained_root.persist();
    std::fs::write(marker, root.to_string_lossy().as_bytes()).expect("persisted root marker");
}

#[test]
#[ignore = "launched by the repeated page reuse parent"]
fn phase_five_repeated_page_reuse_planner() {
    let planned = plan(&required_child_path());
    let decisions = planned.redo_plan().resolved_decisions().collect::<Vec<_>>();
    assert_eq!(decisions.len(), 2);
    assert!(decisions.iter().all(|decision| {
        decision.kind() == PhysicalRedoDecisionKind::SkipPageAlreadyAtOrBeyondLsn
    }));
    let PhysicalRedoTargetIdentity::InlinePage {
        segment,
        page,
        generation: 1,
    } = decisions[0].target().identity()
    else {
        panic!("the first append creates a new inline page")
    };
    assert_eq!(
        decisions[1].target().identity(),
        PhysicalRedoTargetIdentity::InlinePage {
            segment,
            page,
            generation: 2,
        }
    );
}

#[test]
#[ignore = "launched by the generation-axis parent"]
fn phase_five_generation_axes_writer() {
    let marker = required_child_path();
    let world = PhysicalResidencyStoreWorld::initialize_for_recovery_with_segment_pages(
        "c8-phase5-generation-axes",
        4,
    )
    .unwrap();
    let retained_root = world.retained_root();
    let base = [
        vec![1_u8; 3_000],
        vec![2_u8; 3_000],
        vec![3_u8; 3_000],
        vec![4_u8; 3_000],
    ];
    canonical_physical_batch_acknowledgment(&world, [0x71; 32], base.iter().map(Vec::as_slice));
    let request = PhysicalCheckpointRequest::fuzzy(
        PhysicalCheckpointIdempotencyKey::new([0x72; 32]),
        PhysicalCheckpointDeadline::after_milliseconds(5_000).unwrap(),
    );
    let TransitionOutcome::Success(handle) =
        world.serving().checkpoints().start(request).into_raw()
    else {
        panic!("generation-axis checkpoint admission")
    };
    assert!(matches!(
        handle.wait(),
        PhysicalCheckpointOutcome::Completed(_)
    ));
    canonical_durable_wal_attempt_without_execution(&world, [0x73; 32], &vec![5_u8; 3_000]);
    drop(world);
    let root = retained_root.persist();
    std::fs::write(marker, root.to_string_lossy().as_bytes()).expect("persisted root marker");
}

#[test]
#[ignore = "launched by the generation-axis parent"]
fn phase_five_generation_axes_stager() {
    let root = required_child_path();
    let planned = plan(&root);
    let decision = planned
        .redo_plan()
        .resolved_decisions()
        .find(|decision| decision.kind() == PhysicalRedoDecisionKind::Apply)
        .expect("the WAL-only append requires one physical apply");
    let PhysicalRedoTargetIdentity::InlinePage {
        segment,
        page,
        generation,
    } = decision.target().identity()
    else {
        panic!("the full inline tail produces another inline page")
    };
    assert_eq!((segment, generation), (1, 1));
    assert!(
        page > 1,
        "the WAL-only append allocates beyond the full tail"
    );
    assert_eq!(
        decision.target().artifact(),
        RecordArtifactFile::Segment {
            segment: 1,
            generation: 2,
        }
    );
    assert_eq!(decision.target().artifact_offset(), 0);
    let command = planned
        .staging_layout()
        .commands()
        .iter()
        .find(|command| command.artifact() == decision.target().artifact())
        .expect("the exact segment artifact is planned");
    assert_eq!(command.byte_count(), 16_384);
    let staged = planned.stage().expect("independent generation axes stage");
    assert_eq!(staged.closed_generation().artifact_count(), 1);
    assert_eq!(staged.staging_counters().artifacts_created, 1);
    assert_eq!(staged.staging_counters().artifacts_synchronized, 1);
    assert!(staged.is_quiescent());
}

#[test]
#[ignore = "launched by the generation-axis parent"]
fn phase_five_generation_axes_convergence() {
    let root = required_child_path();
    let converged = plan(&root).stage().expect("identical staging converges");
    assert_eq!(converged.staging_counters().artifacts_created, 0);
    assert_eq!(converged.staging_counters().artifacts_converged, 1);
    assert_eq!(converged.staging_counters().artifacts_synchronized, 1);
    assert!(converged.is_quiescent());
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

fn run_child(test: &str, path: &Path, temporary_root: &Path) -> Output {
    Command::new(std::env::current_exe().unwrap())
        .args(["--exact", test, "--ignored", "--nocapture"])
        .env(CHILD_ROOT, path)
        .env("TMP", temporary_root)
        .env("TEMP", temporary_root)
        .env("TMPDIR", temporary_root)
        .output()
        .expect("launch generation-axis child")
}

fn required_child_path() -> PathBuf {
    std::env::var_os(CHILD_ROOT)
        .map(PathBuf::from)
        .expect("generation-axis child root")
}

fn assert_child_succeeded(name: &str, output: &Output) {
    assert!(
        output.status.success(),
        "{name} child failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );
}
