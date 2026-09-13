use std::path::Path;
use std::process::Command;

use worth_proof::TransitionOutcome;
use worth_store::physical_runtime::{
    PhysicalCheckpointDeadline, PhysicalCheckpointIdempotencyKey, PhysicalCheckpointOutcome,
    PhysicalCheckpointRequest,
};
use worth_store_test_support::harness::physical_residency::{
    canonical_physical_mutation_acknowledgment, PhysicalResidencyStoreWorld,
};

#[test]
fn production_entry_admits_an_existing_store_in_a_fresh_process() {
    let world = initialized_recovery_world("production-entry");
    let retained_root = world.retained_root();
    let root = retained_root.path().to_path_buf();
    let store_identity = world.serving().store_identity();
    drop(world);

    let first = run_entry(&root);
    let second = run_entry(&root);

    assert!(
        first.status.success(),
        "production entry failed: {}",
        String::from_utf8_lossy(&first.stderr)
    );
    assert!(second.status.success());
    let first_stderr = String::from_utf8_lossy(&first.stderr);
    let second_stderr = String::from_utf8_lossy(&second.stderr);
    assert!(first_stderr.contains("recovered Store"));
    assert!(first_stderr.contains(&format!("{:?}", store_identity.bytes())));
    assert_ne!(runtime_text(&first_stderr), runtime_text(&second_stderr));

    let authority = worth_store_recovery_runtime::PhysicalRecoveryPlatformAuthority::acquire(
        root,
        worth_store_recovery_runtime::PhysicalRecoveryStaticConfiguration::current(),
        worth_store_recovery_runtime::PhysicalRecoveryLimits::admit(test_limits())
            .expect("test limits"),
    )
    .expect("production entry released its root lease");
    drop(authority);
}

#[test]
fn production_entry_refuses_absent_incomplete_and_contended_roots() {
    let parent = tempfile::tempdir().expect("test root parent");
    let absent = parent.path().join("absent");
    let absent_output = run_entry(&absent);
    assert!(!absent_output.status.success());
    assert!(!absent.exists());

    let incomplete = parent.path().join("incomplete");
    std::fs::create_dir(&incomplete).expect("incomplete root");
    let incomplete_output = run_entry(&incomplete);
    assert!(!incomplete_output.status.success());
    assert_eq!(
        std::fs::read_dir(&incomplete)
            .expect("unchanged root")
            .count(),
        0
    );

    let world = initialized_recovery_world("production-contention");
    let retained_root = world.retained_root();
    let root = retained_root.path().to_path_buf();
    drop(world);
    let limits = worth_store_recovery_runtime::PhysicalRecoveryLimits::admit(test_limits())
        .expect("test limits");
    let owner = worth_store_recovery_runtime::PhysicalRecoveryPlatformAuthority::acquire(
        root.clone(),
        worth_store_recovery_runtime::PhysicalRecoveryStaticConfiguration::current(),
        limits,
    )
    .expect("parent owns recovery root");
    let contended = run_entry(&root);
    assert!(!contended.status.success());
    assert!(String::from_utf8_lossy(&contended.stderr).contains("OwnershipContended"));
    drop(owner);
}

#[test]
fn production_entry_rejects_a_report_path_inside_the_store_root() {
    let world = initialized_recovery_world("production-report-safety");
    let retained_root = world.retained_root();
    let root = retained_root.path().to_path_buf();
    let before = std::fs::read_dir(&root)
        .expect("read store before report rejection")
        .map(|entry| entry.expect("store entry").file_name())
        .collect::<Vec<_>>();
    drop(world);

    let report = root.join("report-inside-store.bin");
    let output = Command::new(env!("CARGO_BIN_EXE_physical_store_recover"))
        .arg(&root)
        .arg("--bounded-profile=c8-phase2-admission-v1")
        .arg(format!("--report={}", report.display()))
        .output()
        .expect("run report safety entry");
    assert!(!output.status.success());
    assert!(!report.exists());
    let after = std::fs::read_dir(&root)
        .expect("read store after report rejection")
        .map(|entry| entry.expect("store entry").file_name())
        .collect::<Vec<_>>();
    assert_eq!(after, before);
}

fn run_entry(root: &Path) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_physical_store_recover"))
        .arg(root)
        .arg("--bounded-profile=c8-phase2-admission-v1")
        .output()
        .expect("run production recovery entry")
}

fn runtime_text(stderr: &str) -> &str {
    stderr
        .split_once(" runtime ")
        .expect("runtime prefix")
        .1
        .split_once(" at root generation")
        .expect("runtime suffix")
        .0
}

fn initialized_recovery_world(label: &str) -> PhysicalResidencyStoreWorld {
    let world = PhysicalResidencyStoreWorld::initialize_for_recovery(label).unwrap();
    canonical_physical_mutation_acknowledgment(&world, [0x41; 32], b"production-entry-redo");
    let request = PhysicalCheckpointRequest::fuzzy(
        PhysicalCheckpointIdempotencyKey::new([0x42; 32]),
        PhysicalCheckpointDeadline::after_milliseconds(5_000).unwrap(),
    );
    let TransitionOutcome::Success(handle) =
        world.serving().checkpoints().start(request).into_raw()
    else {
        panic!("production entry checkpoint admission must succeed")
    };
    assert!(matches!(
        handle.wait(),
        PhysicalCheckpointOutcome::Completed(_)
    ));
    world
}

fn test_limits() -> worth_store_recovery_runtime::PhysicalRecoveryLimitDeclaration {
    worth_store_recovery_runtime::PhysicalRecoveryLimitDeclaration {
        selector_candidates: 1,
        checkpoint_candidates: 1,
        manifest_bytes: 1,
        manifest_entries: 1,
        wal_segments: 1,
        wal_frames: 1,
        wal_bytes: 1,
        redo_targets: 1,
        redo_bytes: 1,
        distinct_pages_and_extents: 1,
        operation_bindings: 1,
        staging_bytes: 1,
        recovery_memory_bytes: 1,
        dirty_frames: 1,
        concurrent_commands: 1,
        publication_effects: 1,
        cleanup_candidates: 1,
        cleanup_bytes: 1,
        observation_bytes: 1,
    }
}
