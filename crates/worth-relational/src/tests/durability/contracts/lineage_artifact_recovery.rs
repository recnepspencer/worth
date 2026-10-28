use super::*;
use crate::durability::data::RecoveryVerificationMode as RecoveryMode;

#[test]
fn checkpoint_duplicate_lineage_event_ids_deny() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = durable_runtime_with_test_schema();
    create_entity_outcome(execution, &runtime, "checkpoint-duplicate-first");
    create_entity_outcome(execution, &runtime, "checkpoint-duplicate-second");
    crate::tests::support::checkpoint_durably(execution, &runtime).unwrap();
    let mut plan = crate::tests::support::recovery_plan_for(
        execution,
        &runtime,
        RecoveryMode::NormalRecoveryVerification,
    )
    .unwrap();
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

    let error = recover_error(execution, plan);
    assert_eq!(error.class, RecoveryFailureClass::CorruptCheckpoint);
    assert!(error.detail.contains("reused"));
}

#[test]
fn tail_duplicate_lineage_event_ids_deny() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = durable_runtime_with_test_schema();
    create_entity_outcome(execution, &runtime, "tail-duplicate-checkpoint");
    crate::tests::support::checkpoint_durably(execution, &runtime).unwrap();
    create_entity_outcome(execution, &runtime, "tail-duplicate-first");
    create_entity_outcome(execution, &runtime, "tail-duplicate-second");
    let mut plan = crate::tests::support::recovery_plan_for(
        execution,
        &runtime,
        RecoveryMode::NormalRecoveryVerification,
    )
    .unwrap();
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

    let error = recover_error(execution, plan);
    assert_eq!(error.class, RecoveryFailureClass::CorruptCanonicalArtifact);
    assert!(error.detail.contains("reused"));
}

#[test]
fn checkpoint_and_tail_lineage_event_id_collision_denies() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = durable_runtime_with_test_schema();
    create_entity_outcome(execution, &runtime, "cross-checkpoint-head");
    crate::tests::support::checkpoint_durably(execution, &runtime).unwrap();
    create_entity_outcome(execution, &runtime, "cross-checkpoint-tail");
    let mut plan = crate::tests::support::recovery_plan_for(
        execution,
        &runtime,
        RecoveryMode::NormalRecoveryVerification,
    )
    .unwrap();
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

    let error = recover_error(execution, plan);
    assert_eq!(error.class, RecoveryFailureClass::CorruptCanonicalArtifact);
    assert!(error.detail.contains("reused"));
}

#[test]
fn tail_lineage_commit_and_branch_cross_splices_deny() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = durable_runtime_with_test_schema();
    create_entity_outcome(execution, &runtime, "cross-splice-checkpoint");
    crate::tests::support::checkpoint_durably(execution, &runtime).unwrap();
    create_entity_outcome(execution, &runtime, "cross-splice-tail");
    let mut plan = crate::tests::support::recovery_plan_for(
        execution,
        &runtime,
        RecoveryMode::NormalRecoveryVerification,
    )
    .unwrap();
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

    let error = recover_error(execution, plan);
    assert_eq!(error.class, RecoveryFailureClass::CorruptCanonicalArtifact);
    assert!(error.detail.contains("cross-spliced"));
}

#[test]
fn checkpoint_lineage_commit_cross_splice_denies() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = durable_runtime_with_test_schema();
    create_entity_outcome(execution, &runtime, "checkpoint-cross-splice");
    crate::tests::support::checkpoint_durably(execution, &runtime).unwrap();
    let mut plan = crate::tests::support::recovery_plan_for(
        execution,
        &runtime,
        RecoveryMode::NormalRecoveryVerification,
    )
    .unwrap();
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

    let error = recover_error(execution, plan);
    assert_eq!(error.class, RecoveryFailureClass::CorruptCheckpoint);
    assert!(error.detail.contains("cross-spliced"));
}

#[test]
fn replay_and_durable_lineage_payload_conflict_denies() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = durable_runtime_with_test_schema();
    create_entity_outcome(execution, &runtime, "payload-conflict-checkpoint");
    crate::tests::support::checkpoint_durably(execution, &runtime).unwrap();
    let first = create_entity_outcome(execution, &runtime, "payload-conflict-first");
    let second = create_entity_outcome(execution, &runtime, "payload-conflict-second");
    let first_lineage = runtime
        .lineage_access()
        .for_record(execution, changed_entities(&first)[0])
        .unwrap()
        .expect("first tail lineage")
        .lineage_id();
    let mut plan = crate::tests::support::recovery_plan_for(
        execution,
        &runtime,
        RecoveryMode::NormalRecoveryVerification,
    )
    .unwrap();
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

    let error = recover_error(execution, plan);
    assert_eq!(error.class, RecoveryFailureClass::CorruptCanonicalArtifact);
    assert!(error.detail.contains("create lineage id"));
}

#[test]
fn checkpoint_lineage_node_deletion_denies_before_installation() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = durable_runtime_with_test_schema();
    create_entity_outcome(execution, &runtime, "checkpoint-node-deletion");
    crate::tests::support::checkpoint_durably(execution, &runtime).unwrap();
    let mut plan = crate::tests::support::recovery_plan_for(
        execution,
        &runtime,
        RecoveryMode::NormalRecoveryVerification,
    )
    .unwrap();
    plan.checkpoint
        .as_mut()
        .expect("checkpoint")
        .lineage
        .nodes_mut()
        .clear();

    let error = recover_error(execution, plan);
    assert_eq!(error.class, RecoveryFailureClass::CorruptCheckpoint);
    assert!(error.detail.contains("lineage counters"));
}

#[test]
fn checkpoint_lineage_node_remap_denies_before_installation() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = durable_runtime_with_test_schema();
    create_entity_outcome(execution, &runtime, "checkpoint-node-remap");
    crate::tests::support::checkpoint_durably(execution, &runtime).unwrap();
    let mut plan = crate::tests::support::recovery_plan_for(
        execution,
        &runtime,
        RecoveryMode::NormalRecoveryVerification,
    )
    .unwrap();
    plan.checkpoint
        .as_mut()
        .expect("checkpoint")
        .lineage
        .nodes_mut()[0]
        .entity_id =
        crate::identity::data::EntityId::new(crate::identity::data::PartitionId::main(), 99_999, 1);

    let error = recover_error(execution, plan);
    assert_eq!(error.class, RecoveryFailureClass::CorruptCheckpoint);
    assert!(error.detail.contains("does not name its exact entity"));
}

#[test]
fn checkpoint_lineage_node_swap_denies_before_installation() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = durable_runtime_with_test_schema();
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
        .commit(execution, &runtime)
        .expect("two-entity commit");
    crate::tests::support::checkpoint_durably(execution, &runtime).unwrap();
    let mut plan = crate::tests::support::recovery_plan_for(
        execution,
        &runtime,
        RecoveryMode::NormalRecoveryVerification,
    )
    .unwrap();
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

    let error = recover_error(execution, plan);
    assert_eq!(error.class, RecoveryFailureClass::CorruptCheckpoint);
    assert!(error.detail.contains("does not name its exact entity"));
}

#[test]
fn checkpoint_duplicate_lineage_entity_mapping_denies_before_installation() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = durable_runtime_with_test_schema();
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
        .commit(execution, &runtime)
        .expect("two-entity commit");
    crate::tests::support::checkpoint_durably(execution, &runtime).unwrap();
    let mut plan = crate::tests::support::recovery_plan_for(
        execution,
        &runtime,
        RecoveryMode::NormalRecoveryVerification,
    )
    .unwrap();
    let nodes = plan
        .checkpoint
        .as_mut()
        .expect("checkpoint")
        .lineage
        .nodes_mut();
    assert_eq!(nodes.len(), 2);
    nodes[1].entity_id = nodes[0].entity_id;

    let error = recover_error(execution, plan);
    assert_eq!(error.class, RecoveryFailureClass::CorruptCheckpoint);
    assert!(error.detail.contains("does not name its exact entity"));
}

fn recover_error(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    plan: crate::durability::data::RecoveryPlan,
) -> crate::durability::data::DurabilityError {
    let mut recovered = durable_runtime_with_test_schema();
    let error = recovered
        .durability_recovery()
        .recover(execution, plan)
        .expect_err("corrupt lineage artifact must deny recovery");
    let error = crate::tests::support::expect_durability_failure(error);
    assert_eq!(recovered.history().immutable_commit_count(), 0);
    error
}

#[path = "lineage_artifact_recovery/lineage_regression.rs"]
mod lineage_regression;
