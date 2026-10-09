use super::*;

#[test]
fn tail_lineage_event_allocator_exhaustion_denies_before_replay() {
    let runtime = persisted_runtime_with_test_schema();
    create_entity_outcome(&runtime, "checkpoint-before-tail-overflow");
    runtime
        .durability_authority()
        .checkpoint()
        .expect("checkpoint");
    let replay_invalid = create_entity_outcome(&runtime, "tail-before-overflow");
    let allocator_exhausted = create_entity_outcome(&runtime, "tail-event-overflow");
    let mut plan = runtime.durability().recovery_plan(
        crate::durability::data::RecoveryVerificationMode::NormalRecoveryVerification,
    );
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

    let mut recovered = persisted_runtime_with_test_schema();
    let error = recovered
        .durability_recovery()
        .recover(plan)
        .expect_err("tail allocator exhaustion must deny");

    assert_eq!(error.class, RecoveryFailureClass::CorruptSegment);
    assert!(error.detail.contains("tail lineage event id exhausted"));
    assert_eq!(recovered.history().immutable_commit_count(), 0);
}

#[test]
fn failed_durable_append_blocks_descendants_and_recovers_last_checkpoint() {
    let runtime = persisted_runtime_with_test_schema();
    create_entity_outcome(&runtime, "lineage-gap-checkpoint");
    runtime.durability_authority().checkpoint().unwrap();
    let (abandoned_lineage_id, abandoned_event_id) = runtime.lineage.identity_frontiers();
    let published_node_count = runtime.lineage.node_count();
    let published_event_count = runtime.lineage.event_count();
    runtime.durability.arm_append_failure();
    let mut failed = test_owner_begin_transaction_for_main(&runtime);
    failed
        .push_batch(
            batch_create("lineage-gap-abandoned"),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("test staging stays within configured resource budgets");
    let durability_deferred = failed
        .commit(
            &runtime,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect_err("performed movement without durable acknowledgement is typed");
    assert!(matches!(
        durability_deferred,
        TransactionCommitError::PerformedButDurabilityDeferred { .. }
    ));
    let performed_commit = durability_deferred
        .performed_commit()
        .expect("deferred error carries performed receipt")
        .clone();
    assert_eq!(
        runtime.lineage.identity_frontiers(),
        (abandoned_lineage_id + 1, abandoned_event_id + 1)
    );
    assert_eq!(runtime.lineage.node_count(), published_node_count + 1);
    assert_eq!(runtime.lineage.event_count(), published_event_count + 1);
    assert_eq!(
        runtime
            .history()
            .branch_head(&BranchId("main".to_owned()))
            .expect("performed append-fault commit is current")
            .commit_id,
        performed_commit.commit_id
    );

    let mut child = test_owner_begin_transaction_for_main(&runtime);
    child
        .push_batch(
            batch_create("lineage-gap-tail"),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("test staging stays within configured resource budgets");
    let child_denial = child
        .commit(
            &runtime,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect_err("an unsettled performed parent denies descendants");
    assert!(child_denial
        .detail()
        .contains("requires explicit owner settlement"));
    assert_eq!(
        runtime
            .fork()
            .expect_err("an unsettled performed parent denies runtime fork"),
        crate::runtime::RelationalRuntimeForkDenial::PerformedPublicationRequiresSettlement {
            commit_id: performed_commit.commit_id,
        }
    );

    let plan = runtime.durability().recovery_plan(
        crate::durability::data::RecoveryVerificationMode::NormalRecoveryVerification,
    );
    let mut recovered = persisted_runtime_with_test_schema();
    let recovery = recovered
        .durability_recovery()
        .recover(plan)
        .expect("recovery stops at the last acknowledged checkpoint");
    assert_eq!(recovery.recovered_commits, 1);
    assert_eq!(
        recovered
            .history()
            .branch_head(&BranchId("main".to_owned()))
            .expect("checkpoint head recovers")
            .commit_id,
        crate::history::data::CommitId(1)
    );
}

#[test]
fn multi_event_reservation_exhaustion_denies_before_public_effects() {
    let runtime = persisted_runtime_with_test_schema();
    let baseline = create_entity_outcome(&runtime, "reservation-exhaustion-baseline");
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
        .commit(
            &runtime,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
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
    let runtime = persisted_runtime_with_test_schema();
    create_entity_outcome(&runtime, "sparse-slot-shared-root");
    let feature = create_branch_from_main(&runtime, "sparse-slot-feature");
    create_entity_outcome(&runtime, "sparse-slot-main-divergence");
    let feature_create =
        create_entity_outcome_on_branch(&runtime, "sparse-slot-feature-create", feature.clone());
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

    let plan = runtime.durability().recovery_plan(
        crate::durability::data::RecoveryVerificationMode::NormalRecoveryVerification,
    );
    let mut recovered = persisted_runtime_with_test_schema();
    recovered
        .durability_recovery()
        .recover(plan)
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
