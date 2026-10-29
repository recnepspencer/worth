#[allow(unused_imports)]
use crate::phase_five_generation_axes::*;
use {std::path::Path, std::path::PathBuf};
use {std::process::Command, std::process::Output};

use worth_proof::TransitionOutcome;
use worth_store_physical_format::RecordArtifactFile;
use worth_store_recovery_runtime::PhysicalRecoveryLimits;
use {
    worth_store::physical_runtime::PhysicalCheckpointDeadline,
    worth_store::physical_runtime::PhysicalCheckpointIdempotencyKey,
    worth_store::physical_runtime::PhysicalCheckpointOutcome,
    worth_store::physical_runtime::PhysicalCheckpointRequest,
};
use {
    worth_store_recovery_physics::PhysicalRedoDecisionKind,
    worth_store_recovery_physics::PhysicalRedoTargetIdentity,
};
use {
    worth_store_test_support::harness::physical_residency::canonical_durable_wal_attempt_without_execution,
    worth_store_test_support::harness::physical_residency::canonical_physical_batch_acknowledgment,
    worth_store_test_support::harness::physical_residency::PhysicalResidencyStoreWorld,
};
use {
    worth_store_testkit_recovery_runtime::phase_three_support::admitted_recovery_with_limits,
    worth_store_testkit_recovery_runtime::phase_three_support::limit_declaration,
};

#[test]
fn new_page_in_reused_segment_crosses_process_death_and_stages_exactly() {
    let parent = tempfile::tempdir().expect("generation-axis parent");
    let marker = parent.path().join("persisted-root");
    assert_child_succeeded("writer", &run_child(WRITER_TEST, &marker, parent.path()));
    let root =
        std::path::PathBuf::from(std::fs::read_to_string(&marker).expect("writer root marker"));
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
            "process::recovery_runtime::phase_five_generation_axes::repeated_page_reuse_writer",
            &marker,
            parent.path(),
        ),
    );
    let root =
        std::path::PathBuf::from(std::fs::read_to_string(&marker).expect("writer root marker"));
    assert_child_succeeded(
        "repeated page reuse planner",
        &run_child(
            "process::recovery_runtime::phase_five_generation_axes::repeated_page_reuse_planner",
            &root,
            parent.path(),
        ),
    );
}

#[test]
#[ignore = "launched by the repeated page reuse parent"]
fn repeated_page_reuse_writer() {
    let marker = required_child_path();
    let world = worth_store_test_support::harness::physical_residency::PhysicalResidencyStoreWorld::initialize_for_recovery_with_segment_pages(
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
    let request = worth_store::physical_runtime::PhysicalCheckpointRequest::fuzzy(
        worth_store::physical_runtime::PhysicalCheckpointIdempotencyKey::new([0x82; 32]),
        worth_store::physical_runtime::PhysicalCheckpointDeadline::after_milliseconds(5_000)
            .unwrap(),
    );
    let worth_proof::TransitionOutcome::Success(handle) =
        world.serving().checkpoints().start(request).into_raw()
    else {
        panic!("repeated page reuse checkpoint admission")
    };
    assert!(matches!(
        handle.wait(),
        worth_store::physical_runtime::PhysicalCheckpointOutcome::Completed(_)
    ));
    canonical_physical_batch_acknowledgment(&world, [0x83; 32], [&vec![5_u8; 3_000][..]]);
    canonical_physical_batch_acknowledgment(&world, [0x84; 32], [&vec![6_u8; 3_000][..]]);
    drop(world);
    let root = retained_root.persist();
    std::fs::write(marker, root.to_string_lossy().as_bytes()).expect("persisted root marker");
}

#[test]
#[ignore = "launched by the repeated page reuse parent"]
fn repeated_page_reuse_planner() {
    let planned = plan(&required_child_path());
    let decisions = planned.redo_plan().resolved_decisions().collect::<Vec<_>>();
    assert_eq!(decisions.len(), 2);
    assert!(decisions.iter().all(|decision| {
        decision.kind() == worth_store_recovery_physics::PhysicalRedoDecisionKind::SkipPageAlreadyAtOrBeyondLsn
    }));
    let worth_store_recovery_physics::PhysicalRedoTargetIdentity::InlinePage {
        segment,
        page,
        generation: 1,
    } = decisions[0].target().identity()
    else {
        panic!("the first append creates a new inline page")
    };
    assert_eq!(
        decisions[1].target().identity(),
        worth_store_recovery_physics::PhysicalRedoTargetIdentity::InlinePage {
            segment,
            page,
            generation: 2,
        }
    );
}

#[test]
#[ignore = "launched by the generation-axis parent"]
fn generation_axes_writer() {
    let marker = required_child_path();
    let world = worth_store_test_support::harness::physical_residency::PhysicalResidencyStoreWorld::initialize_for_recovery_with_segment_pages(
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
    let request = worth_store::physical_runtime::PhysicalCheckpointRequest::fuzzy(
        worth_store::physical_runtime::PhysicalCheckpointIdempotencyKey::new([0x72; 32]),
        worth_store::physical_runtime::PhysicalCheckpointDeadline::after_milliseconds(5_000)
            .unwrap(),
    );
    let worth_proof::TransitionOutcome::Success(handle) =
        world.serving().checkpoints().start(request).into_raw()
    else {
        panic!("generation-axis checkpoint admission")
    };
    assert!(matches!(
        handle.wait(),
        worth_store::physical_runtime::PhysicalCheckpointOutcome::Completed(_)
    ));
    canonical_durable_wal_attempt_without_execution(&world, [0x73; 32], &vec![5_u8; 3_000]);
    drop(world);
    let root = retained_root.persist();
    std::fs::write(marker, root.to_string_lossy().as_bytes()).expect("persisted root marker");
}

#[test]
#[ignore = "launched by the generation-axis parent"]
fn generation_axes_stager() {
    let root = required_child_path();
    let planned = plan(&root);
    let decision = planned
        .redo_plan()
        .resolved_decisions()
        .find(|decision| {
            decision.kind() == worth_store_recovery_physics::PhysicalRedoDecisionKind::Apply
        })
        .expect("the WAL-only append requires one physical apply");
    let worth_store_recovery_physics::PhysicalRedoTargetIdentity::InlinePage {
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
        worth_store_physical_format::RecordArtifactFile::Segment {
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
fn generation_axes_convergence() {
    let root = required_child_path();
    let converged = plan(&root).stage().expect("identical staging converges");
    assert_eq!(converged.staging_counters().artifacts_created, 0);
    assert_eq!(converged.staging_counters().artifacts_converged, 1);
    assert_eq!(converged.staging_counters().artifacts_synchronized, 1);
    assert!(converged.is_quiescent());
}
