use super::*;

#[test]
fn checkpoint_duplicate_lineage_event_ids_deny() {
    let runtime = persisted_runtime_with_test_schema();
    create_entity_outcome(&runtime, "checkpoint-duplicate-first");
    create_entity_outcome(&runtime, "checkpoint-duplicate-second");
    runtime.durability_authority().checkpoint().unwrap();
    let mut plan = runtime.durability().recovery_plan(
        crate::durability::data::RecoveryVerificationMode::NormalRecoveryVerification,
    );
    let mut events = plan
        .checkpoint
        .as_mut()
        .expect("checkpoint")
        .envelopes
        .iter_mut()
        .map(|commit| commit.envelope_mut_for_test())
        .flat_map(|envelope| {
            envelope
                .published_lineage_mut_for_test()
                .lineage_events_mut()
        })
        .collect::<Vec<_>>();
    let first_id = events[0].event_id;
    events[1].event_id = first_id;
    drop(events);
    for envelope in &mut plan.checkpoint.as_mut().expect("checkpoint").envelopes {
        envelope
            .envelope_mut_for_test()
            .rebuild_lineage_basis_for_test();
    }

    let error = recover_error(plan);
    assert_eq!(error.class, RecoveryFailureClass::CorruptCheckpoint);
    assert!(error.detail.contains("reused"));
}

#[test]
fn tail_duplicate_lineage_event_ids_deny() {
    let runtime = persisted_runtime_with_test_schema();
    create_entity_outcome(&runtime, "tail-duplicate-checkpoint");
    runtime.durability_authority().checkpoint().unwrap();
    create_entity_outcome(&runtime, "tail-duplicate-first");
    create_entity_outcome(&runtime, "tail-duplicate-second");
    let mut plan = runtime.durability().recovery_plan(
        crate::durability::data::RecoveryVerificationMode::NormalRecoveryVerification,
    );
    let mut events = plan
        .tail_log
        .iter_mut()
        .map(|commit| commit.envelope_mut_for_test())
        .flat_map(|envelope| {
            envelope
                .published_lineage_mut_for_test()
                .lineage_events_mut()
        })
        .collect::<Vec<_>>();
    let first_id = events[0].event_id;
    events[1].event_id = first_id;
    drop(events);
    for envelope in &mut plan.tail_log {
        envelope
            .envelope_mut_for_test()
            .rebuild_lineage_basis_for_test();
    }

    let error = recover_error(plan);
    assert_eq!(error.class, RecoveryFailureClass::CorruptSegment);
    assert!(error.detail.contains("reused"));
}

#[test]
fn checkpoint_and_tail_lineage_event_id_collision_denies() {
    let runtime = persisted_runtime_with_test_schema();
    create_entity_outcome(&runtime, "cross-segment-checkpoint");
    runtime.durability_authority().checkpoint().unwrap();
    create_entity_outcome(&runtime, "cross-segment-tail");
    let mut plan = runtime.durability().recovery_plan(
        crate::durability::data::RecoveryVerificationMode::NormalRecoveryVerification,
    );
    let checkpoint_id = plan
        .checkpoint
        .as_ref()
        .expect("checkpoint")
        .envelopes
        .iter()
        .flat_map(|envelope| envelope.lineage_events())
        .next()
        .expect("checkpoint event")
        .event_id();
    plan.tail_log[0]
        .envelope_mut_for_test()
        .published_lineage_mut_for_test()
        .lineage_events_mut()[0]
        .event_id = checkpoint_id;
    plan.tail_log[0]
        .envelope_mut_for_test()
        .rebuild_lineage_basis_for_test();

    let error = recover_error(plan);
    assert_eq!(error.class, RecoveryFailureClass::CorruptSegment);
    assert!(error.detail.contains("reused"));
}

#[test]
fn tail_lineage_commit_and_branch_cross_splices_deny() {
    let runtime = persisted_runtime_with_test_schema();
    create_entity_outcome(&runtime, "cross-splice-checkpoint");
    runtime.durability_authority().checkpoint().unwrap();
    create_entity_outcome(&runtime, "cross-splice-tail");
    let mut plan = runtime.durability().recovery_plan(
        crate::durability::data::RecoveryVerificationMode::NormalRecoveryVerification,
    );
    let event = plan.tail_log[0]
        .envelope_mut_for_test()
        .published_lineage_mut_for_test()
        .lineage_events_mut()
        .first_mut()
        .expect("tail event");
    event.commit.branch_id = BranchId("spliced-sibling".to_owned());
    event.branch_id = BranchId("spliced-sibling".to_owned());
    plan.tail_log[0]
        .envelope_mut_for_test()
        .rebuild_lineage_basis_for_test();

    let error = recover_error(plan);
    assert_eq!(error.class, RecoveryFailureClass::CorruptSegment);
    assert!(error.detail.contains("cross-spliced"));
}

#[test]
fn checkpoint_lineage_commit_cross_splice_denies() {
    let runtime = persisted_runtime_with_test_schema();
    create_entity_outcome(&runtime, "checkpoint-cross-splice");
    runtime.durability_authority().checkpoint().unwrap();
    let mut plan = runtime.durability().recovery_plan(
        crate::durability::data::RecoveryVerificationMode::NormalRecoveryVerification,
    );
    let event = plan.checkpoint.as_mut().expect("checkpoint").envelopes[0]
        .envelope_mut_for_test()
        .published_lineage_mut_for_test()
        .lineage_events_mut()
        .first_mut()
        .expect("checkpoint event");
    event.commit.commit_id = crate::history::data::CommitId(event.commit.commit_id.0 + 10);
    plan.checkpoint.as_mut().expect("checkpoint").envelopes[0]
        .envelope_mut_for_test()
        .rebuild_lineage_basis_for_test();

    let error = recover_error(plan);
    assert_eq!(error.class, RecoveryFailureClass::CorruptCheckpoint);
    assert!(error.detail.contains("cross-spliced"));
}

#[test]
fn replay_and_durable_lineage_payload_conflict_denies() {
    let runtime = persisted_runtime_with_test_schema();
    create_entity_outcome(&runtime, "payload-conflict-checkpoint");
    runtime.durability_authority().checkpoint().unwrap();
    let first = create_entity_outcome(&runtime, "payload-conflict-first");
    let second = create_entity_outcome(&runtime, "payload-conflict-second");
    let first_lineage = runtime
        .lineage_access()
        .for_record(changed_entities(&first)[0])
        .expect("first tail lineage")
        .lineage_id();
    let mut plan = runtime.durability().recovery_plan(
        crate::durability::data::RecoveryVerificationMode::NormalRecoveryVerification,
    );
    let second_event = plan
        .tail_log
        .iter_mut()
        .map(|commit| commit.envelope_mut_for_test())
        .find(|envelope| envelope.commit.commit_id == second.commit.commit_id)
        .expect("second tail envelope")
        .published_lineage_mut_for_test()
        .lineage_events_mut()
        .first_mut()
        .expect("second tail event");
    second_event.targets = vec![first_lineage];
    let second_envelope = plan
        .tail_log
        .iter_mut()
        .map(|commit| commit.envelope_mut_for_test())
        .find(|envelope| envelope.commit.commit_id == second.commit.commit_id)
        .expect("second tail envelope");
    second_envelope.rebuild_lineage_basis_for_test();

    let error = recover_error(plan);
    assert_eq!(error.class, RecoveryFailureClass::CorruptSegment);
    assert!(error.detail.contains("create lineage id"));
}

#[test]
fn checkpoint_lineage_node_deletion_denies_before_installation() {
    let runtime = persisted_runtime_with_test_schema();
    create_entity_outcome(&runtime, "checkpoint-node-deletion");
    runtime.durability_authority().checkpoint().unwrap();
    let mut plan = runtime.durability().recovery_plan(
        crate::durability::data::RecoveryVerificationMode::NormalRecoveryVerification,
    );
    plan.checkpoint
        .as_mut()
        .expect("checkpoint")
        .lineage
        .nodes_mut()
        .clear();

    let error = recover_error(plan);
    assert_eq!(error.class, RecoveryFailureClass::CorruptCheckpoint);
    assert!(error.detail.contains("lineage counters"));
}

#[test]
fn checkpoint_lineage_node_remap_denies_before_installation() {
    let runtime = persisted_runtime_with_test_schema();
    create_entity_outcome(&runtime, "checkpoint-node-remap");
    runtime.durability_authority().checkpoint().unwrap();
    let mut plan = runtime.durability().recovery_plan(
        crate::durability::data::RecoveryVerificationMode::NormalRecoveryVerification,
    );
    plan.checkpoint
        .as_mut()
        .expect("checkpoint")
        .lineage
        .nodes_mut()[0]
        .entity_id =
        crate::identity::data::EntityId::new(crate::identity::data::PartitionId::main(), 99_999, 1);

    let error = recover_error(plan);
    assert_eq!(error.class, RecoveryFailureClass::CorruptCheckpoint);
    assert!(error.detail.contains("does not name its exact entity"));
}

#[test]
fn checkpoint_lineage_node_swap_denies_before_installation() {
    let runtime = persisted_runtime_with_test_schema();
    let mut transaction = test_owner_begin_transaction_for_main(&runtime);
    transaction
        .push_batch(
            batch_create("checkpoint-swap-first"),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("test staging stays within configured resource budgets");
    transaction
        .push_batch(
            batch_create("checkpoint-swap-second"),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("test staging stays within configured resource budgets");
    transaction
        .commit(
            &runtime,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("two-entity commit");
    runtime.durability_authority().checkpoint().unwrap();
    let mut plan = runtime.durability().recovery_plan(
        crate::durability::data::RecoveryVerificationMode::NormalRecoveryVerification,
    );
    let nodes = plan
        .checkpoint
        .as_mut()
        .expect("checkpoint")
        .lineage
        .nodes_mut();
    assert_eq!(nodes.len(), 2);
    let first_entity = nodes[0].entity_id;
    nodes[0].entity_id = nodes[1].entity_id;
    nodes[1].entity_id = first_entity;

    let error = recover_error(plan);
    assert_eq!(error.class, RecoveryFailureClass::CorruptCheckpoint);
    assert!(error.detail.contains("does not name its exact entity"));
}

#[test]
fn checkpoint_duplicate_lineage_entity_mapping_denies_before_installation() {
    let runtime = persisted_runtime_with_test_schema();
    let mut transaction = test_owner_begin_transaction_for_main(&runtime);
    transaction
        .push_batch(
            batch_create("checkpoint-duplicate-node-first"),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("test staging stays within configured resource budgets");
    transaction
        .push_batch(
            batch_create("checkpoint-duplicate-node-second"),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("test staging stays within configured resource budgets");
    transaction
        .commit(
            &runtime,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("two-entity commit");
    runtime.durability_authority().checkpoint().unwrap();
    let mut plan = runtime.durability().recovery_plan(
        crate::durability::data::RecoveryVerificationMode::NormalRecoveryVerification,
    );
    let nodes = plan
        .checkpoint
        .as_mut()
        .expect("checkpoint")
        .lineage
        .nodes_mut();
    assert_eq!(nodes.len(), 2);
    nodes[1].entity_id = nodes[0].entity_id;

    let error = recover_error(plan);
    assert_eq!(error.class, RecoveryFailureClass::CorruptCheckpoint);
    assert!(error.detail.contains("does not name its exact entity"));
}

#[test]
fn tail_lineage_event_regression_below_checkpoint_high_water_denies() {
    let runtime = persisted_runtime_with_test_schema();
    create_entity_outcome(&runtime, "event-regression-checkpoint");
    runtime.durability_authority().checkpoint().unwrap();
    create_entity_outcome(&runtime, "event-regression-tail");
    let mut plan = runtime.durability().recovery_plan(
        crate::durability::data::RecoveryVerificationMode::NormalRecoveryVerification,
    );
    plan.tail_log[0]
        .envelope_mut_for_test()
        .published_lineage_mut_for_test()
        .lineage_events_mut()[0]
        .event_id = 0;
    plan.tail_log[0]
        .envelope_mut_for_test()
        .rebuild_lineage_basis_for_test();

    let error = recover_error(plan);
    assert_eq!(error.class, RecoveryFailureClass::CorruptSegment);
    assert!(error.detail.contains("lineage event id 0 does not advance"));
}

#[test]
fn tail_created_lineage_regression_below_checkpoint_high_water_denies() {
    let runtime = persisted_runtime_with_test_schema();
    create_entity_outcome(&runtime, "lineage-regression-checkpoint");
    runtime.durability_authority().checkpoint().unwrap();
    create_entity_outcome(&runtime, "lineage-regression-tail");
    let mut plan = runtime.durability().recovery_plan(
        crate::durability::data::RecoveryVerificationMode::NormalRecoveryVerification,
    );
    plan.tail_log[0]
        .envelope_mut_for_test()
        .published_lineage_mut_for_test()
        .lineage_events_mut()[0]
        .targets[0] = crate::identity::data::LineageId(0);
    plan.tail_log[0]
        .envelope_mut_for_test()
        .rebuild_lineage_basis_for_test();

    let error = recover_error(plan);
    assert_eq!(error.class, RecoveryFailureClass::CorruptSegment);
    assert!(error
        .detail
        .contains("create lineage id 0 does not advance"));
}

fn recover_error(
    plan: crate::durability::data::RecoveryPlan,
) -> crate::durability::data::DurabilityError {
    let mut recovered = persisted_runtime_with_test_schema();
    let error = recovered
        .durability_recovery()
        .recover(plan)
        .expect_err("corrupt lineage artifact must deny recovery");
    assert_eq!(recovered.history().immutable_commit_count(), 0);
    error
}
