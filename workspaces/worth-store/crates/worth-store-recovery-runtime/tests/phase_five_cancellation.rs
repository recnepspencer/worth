#[allow(dead_code)]
mod phase_three_support;

use std::path::{Path, PathBuf};

use phase_three_support::{admitted_recovery_with_limits, limit_declaration};
use worth_proof::TransitionOutcome;
use worth_store::physical_runtime::{
    PhysicalCheckpointDeadline, PhysicalCheckpointIdempotencyKey, PhysicalCheckpointOutcome,
    PhysicalCheckpointRequest,
};
use worth_store_recovery_runtime::{
    PhysicalRecoveryLimits, PhysicalRecoveryOutcome, PhysicalRecoveryStagingDenial,
};
use worth_store_test_support::harness::physical_residency::{
    canonical_durable_wal_attempt_without_execution, canonical_physical_mutation_acknowledgment,
    PhysicalResidencyStoreWorld,
};

#[test]
fn cancellation_between_staging_commands_retains_exact_settled_effects() {
    let (_parent, root) = multi_artifact_recovery_world("phase5-partial-cancellation");
    let planned = plan(&root);
    assert!(
        planned.staging_layout().commands().len() >= 2,
        "the causal fixture must stage more than one artifact"
    );
    let cancellation = planned
        .cancellation_after_command(0)
        .expect("the first command is a declared cancellation safe point");

    let Err(PhysicalRecoveryOutcome::Blocked(blocked)) =
        planned.stage_with_cancellation(cancellation)
    else {
        panic!("partial-effect cancellation terminates Blocked")
    };
    assert!(matches!(
        blocked.evidence().staging_denial,
        Some(
            PhysicalRecoveryStagingDenial::CancelledAfterPartialStaging {
                settled_commands: 2,
            }
        )
    ));
    let counters = blocked.evidence().staging_counters.unwrap();
    assert_eq!(counters.commands_submitted, 2);
    assert_eq!(counters.commands_settled, 2);
    assert_eq!(counters.scheduler_settlements, 2);
    assert_eq!(counters.performed_effects, 2);
    assert_eq!(
        blocked
            .evidence()
            .staging_settlements
            .as_ref()
            .unwrap()
            .completed(),
        1
    );
    assert_eq!(blocked.recovery_effects(), 2);
}

#[test]
fn cancellation_authority_from_another_plan_has_no_effect() {
    let (_left_parent, left_root) = multi_artifact_recovery_world("phase5-cancel-left");
    let (_right_parent, right_root) = multi_artifact_recovery_world("phase5-cancel-right");
    let left = plan(&left_root);
    let cancellation = left.cancellation_after_command(0).unwrap();
    let right = plan(&right_root);

    let Err(PhysicalRecoveryOutcome::Blocked(blocked)) =
        right.stage_with_cancellation(cancellation)
    else {
        panic!("a foreign cancellation request cannot enter staging")
    };
    assert_eq!(
        blocked.evidence().staging_denial,
        Some(PhysicalRecoveryStagingDenial::InvalidPlan)
    );
    let counters = blocked.evidence().staging_counters.unwrap();
    assert_eq!(counters.commands_submitted, 0);
    assert_eq!(counters.performed_effects, 0);
    assert_eq!(blocked.recovery_effects(), 0);
    assert!(matches!(
        left.cancel_before_execution(),
        PhysicalRecoveryOutcome::Refused(_)
    ));
}

fn multi_artifact_recovery_world(label: &str) -> (tempfile::TempDir, PathBuf) {
    let parent = tempfile::tempdir().expect("cancellation world parent");
    let world = PhysicalResidencyStoreWorld::initialize_for_recovery(label).unwrap();
    let retained_root = world.retained_root();
    canonical_physical_mutation_acknowledgment(&world, [0x71; 32], b"cancellation-base");
    checkpoint(&world);
    canonical_durable_wal_attempt_without_execution(&world, [0x72; 32], b"inline-pending");
    canonical_durable_wal_attempt_without_execution(&world, [0x73; 32], &vec![0x74; 32 * 1024]);
    drop(world);
    let copied = parent.path().join("retained-root");
    copy_directory(&retained_root.persist(), &copied);
    (parent, copied)
}

fn checkpoint(world: &PhysicalResidencyStoreWorld) {
    let request = PhysicalCheckpointRequest::fuzzy(
        PhysicalCheckpointIdempotencyKey::new([0x75; 32]),
        PhysicalCheckpointDeadline::after_milliseconds(5_000).unwrap(),
    );
    let TransitionOutcome::Success(handle) =
        world.serving().checkpoints().start(request).into_raw()
    else {
        panic!("cancellation checkpoint admission")
    };
    assert!(matches!(
        handle.wait(),
        PhysicalCheckpointOutcome::Completed(_)
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
    let mut declaration = limit_declaration(2, 8, 4 * 1024 * 1024);
    declaration.manifest_entries = 4_096;
    declaration.wal_bytes = 4 * 1024 * 1024;
    declaration.redo_targets = 4_096;
    declaration.redo_bytes = 8 * 1024 * 1024;
    declaration.distinct_pages_and_extents = 4_096;
    declaration.operation_bindings = 4_096;
    declaration.staging_bytes = 64 * 1024 * 1024;
    declaration.recovery_memory_bytes = 32 * 1024 * 1024;
    declaration.dirty_frames = 4_096;
    declaration.observation_bytes = 64 * 1024 * 1024;
    declaration.publication_effects = 64;
    PhysicalRecoveryLimits::admit(declaration).unwrap()
}

fn copy_directory(source: &Path, destination: &Path) {
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
