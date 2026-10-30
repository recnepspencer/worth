#[allow(unused_imports)]
use crate::phase_four_process::*;
use {std::path::Path, std::path::PathBuf};
use {std::process::Command, std::process::Output};

use worth_proof::TransitionOutcome;
use worth_store_physical_format::RecordArtifactFile;
use {
    worth_store::physical_runtime::recovery_wal::WalSegmentArtifactIdentity,
    worth_store::physical_runtime::PhysicalCheckpointDeadline,
    worth_store::physical_runtime::PhysicalCheckpointIdempotencyKey,
    worth_store::physical_runtime::PhysicalCheckpointOutcome,
    worth_store::physical_runtime::PhysicalCheckpointRequest,
};
use {
    worth_store_recovery_physics::PhysicalRedoDecisionKind,
    worth_store_recovery_physics::PhysicalRedoDecisionPrior,
    worth_store_recovery_physics::PhysicalRedoTargetIdentity,
    worth_store_recovery_physics::RecoveryOperationFate,
    worth_store_recovery_physics::RecoveryPageSource,
};
use {
    worth_store_recovery_runtime::PhysicalRecoveryLimits,
    worth_store_recovery_runtime::PhysicalRecoveryOutcome,
    worth_store_recovery_runtime::RecoveryPublicationAction,
    worth_store_recovery_runtime::WorthStoreRecovery,
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
    worth_store_testkit_recovery_runtime::phase_three_support::recovery_request_with_limits,
};

#[test]
fn ordinary_store_state_crosses_process_death_into_one_recovered_handoff() {
    let parent = tempfile::tempdir().expect("process-boundary parent");
    let marker = parent.path().join("persisted-root");
    let writer = run_child(WRITER_TEST, &marker, parent.path());
    assert_child_succeeded("writer", &writer);

    let root =
        std::path::PathBuf::from(std::fs::read_to_string(&marker).expect("writer root marker"));
    assert!(root.starts_with(parent.path()));
    assert!(root.is_dir());
    append_torn_terminal_wal_segment(&root);
    let planner = run_child(PLANNER_TEST, &root, parent.path());
    assert_child_succeeded("planner", &planner);
}

#[test]
fn production_facade_replaces_a_dead_writer_in_a_distinct_process() {
    let parent = tempfile::tempdir().expect("facade process-boundary parent");
    let marker = parent.path().join("persisted-root");
    let writer = run_child(WRITER_TEST, &marker, parent.path());
    assert_child_succeeded("writer", &writer);

    let root =
        std::path::PathBuf::from(std::fs::read_to_string(&marker).expect("writer root marker"));
    let facade = run_child(FACADE_TEST, &root, parent.path());
    assert_child_succeeded("facade", &facade);
}

#[test]
#[ignore = "launched by the process-boundary parent"]
fn planning_writer_process() {
    let marker = required_child_path();
    let world = worth_store_test_support::harness::physical_residency::PhysicalResidencyStoreWorld::initialize_for_recovery("c8-phase4-process").unwrap();
    let retained_root = world.retained_root();
    canonical_physical_mutation_acknowledgment(&world, [0x41; 32], b"ordinary-c8-redo");
    let request = worth_store::physical_runtime::PhysicalCheckpointRequest::fuzzy(
        worth_store::physical_runtime::PhysicalCheckpointIdempotencyKey::new([0x42; 32]),
        worth_store::physical_runtime::PhysicalCheckpointDeadline::after_milliseconds(5_000)
            .unwrap(),
    );
    let worth_proof::TransitionOutcome::Success(handle) =
        world.serving().checkpoints().start(request).into_raw()
    else {
        panic!("ordinary checkpoint admission must succeed")
    };
    let checkpoint = match handle.wait() {
        worth_store::physical_runtime::PhysicalCheckpointOutcome::Completed(checkpoint) => {
            checkpoint
        }
        other => panic!("ordinary checkpoint publication must complete: {other:?}"),
    };
    assert!(checkpoint.retained_wal_tail().segment_count().get() > 0);
    canonical_rooted_mutation_without_acknowledgment(&world, [0x43; 32], b"rooted-c8-redo");
    canonical_durable_wal_attempt_without_execution(&world, [0x44; 32], b"post-checkpoint-c8-redo");
    drop(world);
    let root = retained_root.persist();
    std::fs::write(marker, root.to_string_lossy().as_bytes()).expect("persisted root marker");
}

#[test]
#[ignore = "launched by the process-boundary parent"]
fn planner_process() {
    let root = required_child_path();
    let planned = admitted_recovery_with_limits(&root, ordinary_limits())
        .discover()
        .unwrap()
        .select()
        .unwrap()
        .plan()
        .unwrap();

    let fates = planned.operation_fates().operations();
    // The parent added a poisoned terminal frame after writer process death.
    // C.8 still selects its admissible prefix, with no raw WAL owner decoder entry.
    assert_eq!(planned.discovery_counters().wal_owner_decoder_entries, 0);
    assert!(planned.discovery_counters().wal_integrity_rejections > 0);
    assert_eq!(fates.len(), 3);
    assert_eq!(
        fates[0].fate(),
        worth_store_recovery_physics::RecoveryOperationFate::AcknowledgedDurable
    );
    assert_eq!(
        fates[1].fate(),
        worth_store_recovery_physics::RecoveryOperationFate::DurableUnacknowledged
    );
    assert_eq!(
        fates[2].fate(),
        worth_store_recovery_physics::RecoveryOperationFate::Indeterminate
    );
    let decisions = planned.redo_plan().resolved_decisions().collect::<Vec<_>>();
    assert_eq!(decisions.len(), 2);
    assert_eq!(
        decisions[0].kind(),
        worth_store_recovery_physics::PhysicalRedoDecisionKind::SkipPageAlreadyAtOrBeyondLsn
    );
    assert_eq!(
        decisions[1].kind(),
        worth_store_recovery_physics::PhysicalRedoDecisionKind::Apply
    );
    assert_eq!(decisions[1].record().lsn().get(), 3);
    assert_eq!(
        decisions[1].target().identity(),
        worth_store_recovery_physics::PhysicalRedoTargetIdentity::InlinePage {
            segment: 1,
            page: 1,
            generation: 3,
        }
    );
    let worth_store_recovery_physics::PhysicalRedoDecisionPrior::Page(prior) = decisions[1].prior()
    else {
        panic!("an applicable redo step retains its exact selected-page proof")
    };
    let worth_store_recovery_physics::RecoveryPageSource::Materialized {
        coordinate,
        routing_identity,
    } = prior.source()
    else {
        panic!("the ordinary successor is based on the selected materialized page")
    };
    assert_eq!(
        coordinate.artifact().file_name(),
        "segment-0000000000000001-0000000000000002.pages"
    );
    assert_ne!(routing_identity, [0; 32]);

    let cost = planned.plan_cost();
    assert_eq!(cost.redo_targets(), 2);
    assert_eq!(cost.redo_bytes(), 34_315);
    assert_eq!(cost.distinct_targets(), 2);
    assert_eq!(cost.operation_bindings(), 3);
    assert_eq!(cost.observation_reads(), 7);
    // Includes the 368-byte source-root manifest the checkpoint binds.
    assert_eq!(cost.observation_bytes(), 74_030);
    assert_eq!(cost.staging_bytes(), 3_276_800);
    assert_eq!(cost.dirty_frames(), 1);
    assert_eq!(planned.staging_layout().actions().len(), 1);
    assert_eq!(planned.staging_layout().actions()[0].steps().len(), 1);
    assert!(planned
        .staging_layout()
        .commands()
        .iter()
        .any(|command| command.byte_count() == 16_384));
    let base = planned.staging_layout().base_image();
    assert_eq!(base.actions().len(), 3);
    assert!(base.actions().iter().all(|action| action.is_projected()));
    assert_eq!(base.segment_updates().len(), 1);
    let publication = planned.publication_plan();
    assert!(!publication.candidates().is_empty());
    assert_eq!(
        publication.actions().len(),
        publication.candidates().len() * 2 + 2
    );
    assert_eq!(
        &publication.actions()[publication.actions().len() - 2..],
        [
            worth_store_recovery_runtime::RecoveryPublicationAction::ReplaceRootProtocol,
            worth_store_recovery_runtime::RecoveryPublicationAction::SynchronizeStoreNamespace,
        ]
    );
    assert_ne!(planned.publication_plan().plan_identity(), [0; 32]);
    let protocol = planned.publication_plan().root_protocol();
    assert_eq!(
        protocol.catalog_candidate(),
        worth_store_physical_format::RecordArtifactFile::CatalogCandidate {
            publication: protocol.publication(),
        }
    );

    let expected_generation = planned.publication_plan().staging_generation();
    let expected_plan = planned.publication_plan().plan_identity();
    let reopened = planned
        .stage()
        .unwrap()
        .publish()
        .unwrap()
        .reopen()
        .unwrap();
    assert_eq!(reopened.recovered_root().generation(), expected_generation);
    assert_eq!(
        reopened.publication_expectation().plan_identity(),
        expected_plan
    );
    assert_eq!(reopened.reopen_counters().selector_reads_completed, 1);
    assert_eq!(reopened.reopen_counters().root_reads_completed, 1);
    let worth_store_recovery_runtime::PhysicalRecoveryOutcome::Recovered(handoff) =
        reopened.finish()
    else {
        panic!("process-death recovery must produce the recovered handoff")
    };
    assert_eq!(handoff.core().root().generation(), expected_generation);
    assert_ne!(
        handoff.core().runtime_identity(),
        handoff.core().recovery_runtime_identity()
    );
    assert_eq!(handoff.operation_fates().operations().len(), 3);
    assert_eq!(handoff.discovery_counters().wal_owner_decoder_entries, 0);
    assert_eq!(
        handoff.integrity_observation_count() as usize,
        handoff.integrity_observations().len()
    );
    assert!(handoff.integrity_observations().iter().any(|observation|
        observation.scope().artifact_family()
            == worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily::CheckpointFooter));
    assert!(handoff
        .wal_integrity_observations()
        .iter()
        .any(|observation| matches!(
            observation.outcome(),
            worth_store_recovery_runtime::PhysicalRecoveryWalIntegrityObservationOutcome::Admitted
        )));
    assert!(handoff
        .wal_integrity_observations()
        .iter()
        .any(|observation| matches!(
            observation.outcome(),
            worth_store_recovery_runtime::PhysicalRecoveryWalIntegrityObservationOutcome::Rejected(
                _
            )
        )));
    assert!(handoff.core().recovery_effect_count() > 0);
}

#[test]
#[ignore = "launched by the process-boundary parent"]
fn recovery_facade_process() {
    let root = required_child_path();
    let request = recovery_request_with_limits(&root, ordinary_limits());
    let worth_store_recovery_runtime::PhysicalRecoveryOutcome::Recovered(handoff) =
        worth_store_recovery_runtime::WorthStoreRecovery::recover(request)
    else {
        panic!("the production facade must recover the dead ordinary writer")
    };
    assert_ne!(
        handoff.core().runtime_identity(),
        handoff.core().recovery_runtime_identity()
    );
    assert_eq!(handoff.reopen_counters().selector_reads_completed, 1);
    assert_eq!(handoff.reopen_counters().root_reads_completed, 1);
    assert!(handoff.core().recovery_effect_count() > 0);
}
