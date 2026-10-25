use super::*;
use crate::durability::data::RecoveryVerificationMode as RecoveryMode;
use crate::publication::patch::data::PatchStreamPosition;

#[test]
fn tail_lineage_event_allocator_exhaustion_denies_before_replay() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = durable_runtime_with_test_schema();
    create_entity_outcome(execution, &runtime, "checkpoint-before-tail-overflow");
    crate::tests::support::checkpoint_durably(execution, &runtime).expect("checkpoint");
    let replay_invalid = create_entity_outcome(execution, &runtime, "tail-before-overflow");
    let allocator_exhausted = create_entity_outcome(execution, &runtime, "tail-event-overflow");
    let mut plan = crate::tests::support::recovery_plan_for(
        execution,
        &runtime,
        RecoveryMode::NormalRecoveryVerification,
    )
    .unwrap();
    let invalid_envelope = plan
        .tail_log
        .iter_mut()
        .map(|commit| commit.envelope_mut_for_test())
        .find(|envelope| envelope.commit.commit_id == replay_invalid.commit.commit_id)
        .expect("earlier tail envelope");
    let checkpoint = invalid_envelope
        .branch_cell_checkpoint
        .as_mut()
        .expect("tail envelope branch-cell checkpoint");
    checkpoint.truth_version = crate::branch::RelationalBranchVersion::new(
        checkpoint.truth_version.as_u64().saturating_add(1),
    );
    let event = plan
        .tail_log
        .iter_mut()
        .map(|commit| commit.envelope_mut_for_test())
        .find(|envelope| envelope.commit.commit_id == allocator_exhausted.commit.commit_id)
        .expect("later tail envelope")
        .published_lineage_mut_for_test()
        .lineage_events_mut()
        .first_mut()
        .expect("later tail lineage event");
    event.event_id = u64::MAX - 1;
    let exhausted_envelope = plan
        .tail_log
        .iter_mut()
        .map(|commit| commit.envelope_mut_for_test())
        .find(|envelope| envelope.commit.commit_id == allocator_exhausted.commit.commit_id)
        .expect("later tail envelope");
    exhausted_envelope.rebuild_lineage_basis_for_test();

    let mut recovered = durable_runtime_with_test_schema();
    let error = recovered
        .durability_recovery()
        .recover(execution, plan)
        .expect_err("tail allocator exhaustion must deny");
    let error = crate::tests::support::expect_durability_failure(error);

    assert_eq!(error.class, RecoveryFailureClass::CorruptCanonicalArtifact);
    assert!(error.detail.contains("tail lineage event id exhausted"));
    assert_eq!(recovered.history().immutable_commit_count(), 0);
}

#[test]
fn unresolved_durable_publication_blocks_descendants_and_recovers_last_checkpoint() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = durable_runtime_with_test_schema();
    let checkpointed = create_entity_outcome(execution, &runtime, "lineage-gap-checkpoint");
    crate::tests::support::checkpoint_durably(execution, &runtime).unwrap();
    let frontiers_at_checkpoint = runtime.lineage.identity_frontiers();
    let published_node_count = runtime.lineage.node_count();
    let published_event_count = runtime.lineage.event_count();
    let medium = attached_test_medium(&runtime);
    medium.script_next_publication(ScriptedPublication::IndeterminateWithoutEffect);
    let mut unresolved = test_owner_begin_transaction_for_main(&runtime);
    unresolved
        .push_batch(
            batch_create("lineage-gap-unresolved"),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("test staging stays within configured resource budgets");
    let fate = unresolved
        .commit(execution, &runtime)
        .expect_err("a publication of unknown durable fate is typed")
        .durability_unresolved_fate()
        .expect("the medium could not say whether the commit is durable")
        .clone();
    let crate::durability::medium::RelationalDurabilityUnresolvedSubject::CanonicalCommit {
        position: unresolved_position,
        ..
    } = fate.subject()
    else {
        panic!("a commit's unresolved fate names the commit")
    };
    assert_eq!(unresolved_position, PatchStreamPosition(2));
    assert_eq!(runtime.lineage.node_count(), published_node_count);
    assert_eq!(runtime.lineage.event_count(), published_event_count);
    assert_eq!(
        runtime.history().branch_head(&BranchId("main".to_owned())),
        Some(checkpointed.commit.clone())
    );

    let mut child = test_owner_begin_transaction_for_main(&runtime);
    child
        .push_batch(
            batch_create("lineage-gap-tail"),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("test staging stays within configured resource budgets");
    let child_refusal = child
        .commit(execution, &runtime)
        .expect_err("an unresolved earlier position refuses every later publication");
    assert_eq!(child_refusal.durability_unresolved_fate(), Some(&fate));

    let (recovery, recovered) = recover_with(execution, &runtime, durable_runtime_with_test_schema);
    assert_eq!(recovery.recovered_commits, 1);
    assert_eq!(recovery.coverage.replayed_tail_commits, 0);
    assert_eq!(
        recovered
            .history()
            .branch_head(&BranchId("main".to_owned())),
        Some(checkpointed.commit.clone())
    );
    assert_eq!(
        recovered.lineage.identity_frontiers(),
        frontiers_at_checkpoint
    );
    let resumed = create_entity_outcome(execution, &recovered, "lineage-gap-resumed");
    assert_eq!(resumed.patch_position(), unresolved_position);
    assert_eq!(resumed.commit.parents, vec![checkpointed.commit.commit_id]);
    release_test_commit_snapshot(&recovered, &resumed);
}

#[test]
fn multi_event_reservation_exhaustion_denies_before_public_effects() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = durable_runtime_with_test_schema();
    let baseline = create_entity_outcome(execution, &runtime, "reservation-exhaustion-baseline");
    let baseline_head = baseline.commit.clone();
    let baseline_entities = runtime.storage_access().storage_stats().live_entities;
    let (lineage_frontier, _) = runtime.lineage.identity_frontiers();
    runtime
        .lineage
        .set_identity_frontiers(lineage_frontier, u64::MAX - 1);

    let mut transaction = test_owner_begin_transaction_for_main(&runtime);
    transaction
        .push_batch(
            batch_create("reservation-exhaustion-first"),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("test staging stays within configured resource budgets");
    transaction
        .push_batch(
            batch_create("reservation-exhaustion-second"),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("test staging stays within configured resource budgets");
    let error = transaction
        .commit(execution, &runtime)
        .expect_err("two-event reservation must deny at the allocator boundary");

    assert!(format!("{error:?}").contains("lineage event id allocator exhausted"));
    assert_eq!(
        runtime.lineage.identity_frontiers(),
        (lineage_frontier, u64::MAX - 1)
    );
    assert_eq!(
        runtime
            .history()
            .branch_head(&BranchId("main".to_owned()))
            .expect("baseline head"),
        baseline_head
    );
    assert_eq!(
        runtime.storage_access().storage_stats().live_entities,
        baseline_entities
    );
}

#[test]
fn branch_local_sparse_slot_create_publishes_and_recovers_exact_lineage() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = durable_runtime_with_test_schema();
    create_entity_outcome(execution, &runtime, "sparse-slot-shared-root");
    let feature = create_branch_from_main(&runtime, "sparse-slot-feature");
    create_entity_outcome(execution, &runtime, "sparse-slot-main-divergence");
    let feature_create = create_entity_outcome_on_branch(
        execution,
        &runtime,
        "sparse-slot-feature-create",
        feature.clone(),
    );
    let feature_entity = changed_entities(&feature_create)[0];
    assert!(feature_entity.local_slot.0 > 1);
    let lineage_id = runtime
        .lineage
        .nodes_snapshot()
        .into_iter()
        .find(|node| node.entity_id() == feature_entity)
        .expect("sparse logical slot has lineage")
        .lineage_id();
    let event_targets = runtime
        .history()
        .commit_envelope(feature_create.commit.commit_id)
        .expect("feature create envelope")
        .lineage_events()[0]
        .targets()
        .to_vec();
    assert_eq!(event_targets, vec![lineage_id]);

    let plan = crate::tests::support::recovery_plan_for(
        execution,
        &runtime,
        RecoveryMode::NormalRecoveryVerification,
    )
    .unwrap();
    let mut recovered = durable_runtime_with_test_schema();
    recovered
        .durability_recovery()
        .recover(execution, plan)
        .expect("sparse branch-local lineage recovers");
    assert_eq!(
        recovered
            .lineage
            .nodes_snapshot()
            .into_iter()
            .find(|node| node.entity_id() == feature_entity)
            .expect("recovered sparse lineage")
            .lineage_id(),
        lineage_id
    );
}
