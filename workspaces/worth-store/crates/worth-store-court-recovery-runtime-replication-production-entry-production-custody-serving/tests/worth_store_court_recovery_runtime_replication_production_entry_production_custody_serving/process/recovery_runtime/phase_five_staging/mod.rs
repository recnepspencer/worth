#[allow(unused_imports)]
use crate::phase_five_staging::*;
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
    worth_store_test_support::harness::physical_residency::canonical_durable_wal_attempt_without_execution,
    worth_store_test_support::harness::physical_residency::canonical_physical_mutation_acknowledgment,
    worth_store_test_support::harness::physical_residency::canonical_rooted_mutation_without_acknowledgment,
    worth_store_test_support::harness::physical_residency::PhysicalResidencyStoreWorld,
};
use {
    worth_store_testkit_recovery_runtime::phase_three_support::admitted_recovery_with_limits,
    worth_store_testkit_recovery_runtime::phase_three_support::limit_declaration,
};

#[test]
fn ordinary_store_state_crosses_process_death_into_closed_convergent_staging() {
    let parent = tempfile::tempdir().expect("process-boundary parent");
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
fn conflicting_noncurrent_artifact_blocks_without_counterfeit_performed_evidence() {
    let parent = tempfile::tempdir().expect("conflict parent");
    let marker = parent.path().join("persisted-root");
    assert_child_succeeded("writer", &run_child(WRITER_TEST, &marker, parent.path()));
    let root = std::path::PathBuf::from(std::fs::read_to_string(&marker).unwrap());
    let artifact = worth_store_physical_format::RecordArtifactFile::Segment {
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
        blocked.cause().damage(),
        Some(worth_store_recovery_runtime::PhysicalRecoveryBlockKind::Staging)
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
fn staging_writer_process() {
    let marker = required_child_path();
    let world = worth_store_test_support::harness::physical_residency::PhysicalResidencyStoreWorld::initialize_for_recovery("c8-phase5-process").unwrap();
    let retained_root = world.retained_root();
    canonical_physical_mutation_acknowledgment(&world, [0x51; 32], b"ordinary-c8-stage");
    let request = worth_store::physical_runtime::PhysicalCheckpointRequest::fuzzy(
        worth_store::physical_runtime::PhysicalCheckpointIdempotencyKey::new([0x52; 32]),
        worth_store::physical_runtime::PhysicalCheckpointDeadline::after_milliseconds(5_000)
            .unwrap(),
    );
    let worth_proof::TransitionOutcome::Success(handle) =
        world.serving().checkpoints().start(request).into_raw()
    else {
        panic!("ordinary checkpoint admission must succeed")
    };
    assert!(matches!(
        handle.wait(),
        worth_store::physical_runtime::PhysicalCheckpointOutcome::Completed(_)
    ));
    canonical_rooted_mutation_without_acknowledgment(&world, [0x53; 32], b"rooted-c8-stage");
    canonical_durable_wal_attempt_without_execution(&world, [0x54; 32], b"wal-only-c8-stage");
    drop(world);
    let root = retained_root.persist();
    std::fs::write(marker, root.to_string_lossy().as_bytes()).expect("persisted root marker");
}

#[test]
#[ignore = "launched by the process-boundary parent"]
fn stager_process() {
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
fn staging_convergence_process() {
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
