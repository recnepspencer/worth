use worth_relational_branch::internal_port::history::data::canonical_commit_envelope::CanonicalCommitEnvelopeBodyFixturePort as _;
use worth_relational_branch::internal_port::history::data::canonical_commit_envelope::lineage_rebuild_test::CanonicalCommitEnvelopeBodyFixturePort as _;
use super::*;
use crate::durability::canonical_commit_artifact::checkpoint_body::mutable_positioned_fixture_body;
use crate::durability::data::RecoveryVerificationMode as RecoveryMode;
use worth_relational_contracts::internal_port::lineage::data::artifacts::checkpoint_artifact::LineageCheckpointArtifactOwnerPort as _;
use worth_relational_contracts::internal_port::lineage::data::artifacts::published_artifact::PublishedLineageArtifactFixturePort as _;
use worth_relational_contracts::internal_port::lineage::data::events::LineageEventRecordFieldsPort as _;
use worth_relational_contracts::internal_port::lineage::data::graph::LineageNodeFieldsPort as _;

#[test]
fn checkpoint_duplicate_lineage_event_ids_deny() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);
    let medium = OwnedTestDurableMedium::new();
    let runtime = durable_runtime_for_medium(&medium);
    create_entity_outcome(execution, &runtime, "checkpoint-duplicate-first");
    create_entity_outcome(execution, &runtime, "checkpoint-duplicate-second");
    medium.file_embedded_checkpoint(execution, &runtime);
    let mut plan = medium
        .recovery_plan(
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
        .map(|commit| mutable_positioned_fixture_body(commit, execution))
        .flat_map(|envelope| {
            envelope
                .published_lineage_mut_for_test()
                .lineage_events_mut_for_test()
        })
        .collect::<Vec<_>>();
    let first_id = events[0].event_id();
    *events[1].event_id_mut_for_test() = first_id;
    drop(events);
    for envelope in &mut plan.checkpoint.as_mut().expect("checkpoint").envelopes {
        mutable_positioned_fixture_body(envelope, execution).rebuild_lineage_basis_for_test();
    }
    let error = recover_error(execution, plan);
    assert_eq!(error.class, RecoveryFailureClass::CorruptCheckpoint);
    assert!(error.detail.contains("reused"));
}

#[test]
fn tail_duplicate_lineage_event_ids_deny() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);
    let medium = OwnedTestDurableMedium::new();
    let runtime = durable_runtime_for_medium(&medium);
    create_entity_outcome(execution, &runtime, "tail-duplicate-checkpoint");
    medium.file_embedded_checkpoint(execution, &runtime);
    create_entity_outcome(execution, &runtime, "tail-duplicate-first");
    create_entity_outcome(execution, &runtime, "tail-duplicate-second");
    let mut plan = medium
        .recovery_plan(
            execution,
            &runtime,
            RecoveryMode::NormalRecoveryVerification,
        )
        .unwrap();
    let mut events = plan
        .tail_log
        .iter_mut()
        .map(|commit| commit.readmitted_envelope_mut_for_test(execution))
        .flat_map(|envelope| {
            envelope
                .published_lineage_mut_for_test()
                .lineage_events_mut_for_test()
        })
        .collect::<Vec<_>>();
    let first_id = events[0].event_id();
    *events[1].event_id_mut_for_test() = first_id;
    drop(events);
    for envelope in &mut plan.tail_log {
        envelope
            .readmitted_envelope_mut_for_test(execution)
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
    let medium = OwnedTestDurableMedium::new();
    let runtime = durable_runtime_for_medium(&medium);
    create_entity_outcome(execution, &runtime, "cross-checkpoint-head");
    medium.file_embedded_checkpoint(execution, &runtime);
    create_entity_outcome(execution, &runtime, "cross-checkpoint-tail");
    let mut plan = medium
        .recovery_plan(
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
    *plan.tail_log[0]
        .readmitted_envelope_mut_for_test(execution)
        .published_lineage_mut_for_test()
        .lineage_events_mut_for_test()[0]
        .event_id_mut_for_test() = checkpoint_id;
    plan.tail_log[0]
        .readmitted_envelope_mut_for_test(execution)
        .rebuild_lineage_basis_for_test();

    let error = recover_error(execution, plan);
    assert_eq!(error.class, RecoveryFailureClass::CorruptCanonicalArtifact);
    assert!(error.detail.contains("reused"));
}

#[test]
fn tail_lineage_commit_and_branch_cross_splices_deny() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);
    let medium = OwnedTestDurableMedium::new();
    let runtime = durable_runtime_for_medium(&medium);
    create_entity_outcome(execution, &runtime, "cross-splice-checkpoint");
    medium.file_embedded_checkpoint(execution, &runtime);
    create_entity_outcome(execution, &runtime, "cross-splice-tail");
    let mut plan = medium
        .recovery_plan(
            execution,
            &runtime,
            RecoveryMode::NormalRecoveryVerification,
        )
        .unwrap();
    let event = plan.tail_log[0]
        .readmitted_envelope_mut_for_test(execution)
        .published_lineage_mut_for_test()
        .lineage_events_mut_for_test()
        .first_mut()
        .expect("tail event");
    event.owner_commit_mut().branch_id = BranchId("spliced-sibling".to_owned());
    *event.lineage_event_branch_mut_for_test() = BranchId("spliced-sibling".to_owned());
    plan.tail_log[0]
        .readmitted_envelope_mut_for_test(execution)
        .rebuild_lineage_basis_for_test();

    let error = recover_error(execution, plan);
    assert_eq!(error.class, RecoveryFailureClass::CorruptCanonicalArtifact);
    assert!(error.detail.contains("cross-spliced"));
}

#[test]
fn checkpoint_lineage_commit_cross_splice_denies() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let medium = OwnedTestDurableMedium::new();
    let runtime = durable_runtime_for_medium(&medium);
    create_entity_outcome(execution, &runtime, "checkpoint-cross-splice");
    medium.file_embedded_checkpoint(execution, &runtime);
    let mut plan = medium
        .recovery_plan(
            execution,
            &runtime,
            RecoveryMode::NormalRecoveryVerification,
        )
        .unwrap();
    let positioned = &mut plan.checkpoint.as_mut().expect("checkpoint").envelopes[0];
    let event = mutable_positioned_fixture_body(positioned, execution)
        .published_lineage_mut_for_test()
        .lineage_events_mut_for_test()
        .first_mut()
        .expect("checkpoint event");
    event.owner_commit_mut().commit_id =
        crate::history::data::CommitId(event.commit().commit_id.0 + 10);
    mutable_positioned_fixture_body(positioned, execution).rebuild_lineage_basis_for_test();

    let error = recover_error(execution, plan);
    assert_eq!(error.class, RecoveryFailureClass::CorruptCheckpoint);
    assert!(error.detail.contains("cross-spliced"));
}

#[test]
fn replay_and_durable_lineage_payload_conflict_denies() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let medium = OwnedTestDurableMedium::new();
    let runtime = durable_runtime_for_medium(&medium);
    create_entity_outcome(execution, &runtime, "payload-conflict-checkpoint");
    medium.file_embedded_checkpoint(execution, &runtime);
    let first = create_entity_outcome(execution, &runtime, "payload-conflict-first");
    let second = create_entity_outcome(execution, &runtime, "payload-conflict-second");
    let first_lineage = runtime
        .lineage_access()
        .for_record(execution, changed_entities(&first)[0])
        .unwrap()
        .expect("first tail lineage")
        .lineage_id();
    let mut plan = medium
        .recovery_plan(
            execution,
            &runtime,
            RecoveryMode::NormalRecoveryVerification,
        )
        .unwrap();
    let second_event = plan
        .tail_log
        .iter_mut()
        .map(|commit| commit.readmitted_envelope_mut_for_test(execution))
        .find(|envelope| envelope.commit.commit_id == second.commit.commit_id)
        .expect("second tail envelope")
        .published_lineage_mut_for_test()
        .lineage_events_mut_for_test()
        .first_mut()
        .expect("second tail event");
    *second_event.owner_targets_mut() = vec![first_lineage];
    plan.tail_log
        .iter_mut()
        .map(|commit| commit.readmitted_envelope_mut_for_test(execution))
        .find(|envelope| envelope.commit.commit_id == second.commit.commit_id)
        .expect("second tail envelope")
        .rebuild_lineage_basis_for_test();

    let error = recover_error(execution, plan);
    assert_eq!(error.class, RecoveryFailureClass::CorruptCanonicalArtifact);
    assert!(error.detail.contains("allocated lineage id"), "{error:?}");
}

#[test]
fn checkpoint_lineage_node_deletion_denies_before_installation() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let medium = OwnedTestDurableMedium::new();
    let runtime = durable_runtime_for_medium(&medium);
    create_entity_outcome(execution, &runtime, "checkpoint-node-deletion");
    medium.file_embedded_checkpoint(execution, &runtime);
    let mut plan = medium
        .recovery_plan(
            execution,
            &runtime,
            RecoveryMode::NormalRecoveryVerification,
        )
        .unwrap();
    plan.checkpoint
        .as_mut()
        .expect("checkpoint")
        .lineage
        .checkpoint_nodes_mut_for_test()
        .clear();

    let error = recover_error(execution, plan);
    assert_eq!(error.class, RecoveryFailureClass::CorruptCheckpoint);
    assert!(error.detail.contains("lineage counters"));
}

#[test]
fn checkpoint_lineage_node_remap_denies_before_installation() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let medium = OwnedTestDurableMedium::new();
    let runtime = durable_runtime_for_medium(&medium);
    create_entity_outcome(execution, &runtime, "checkpoint-node-remap");
    medium.file_embedded_checkpoint(execution, &runtime);
    let mut plan = medium
        .recovery_plan(
            execution,
            &runtime,
            RecoveryMode::NormalRecoveryVerification,
        )
        .unwrap();
    let mut wrong_generation = plan.clone();
    let checkpoint = wrong_generation.checkpoint.as_mut().unwrap();
    checkpoint.lineage.checkpoint_nodes_mut_for_test()[0]
        .lineage_node_entity_mut_for_test()
        .generation
        .0 += 1;
    let error = recover_error(execution, wrong_generation);
    assert_eq!(error.class, RecoveryFailureClass::CorruptCheckpoint);
    assert!(error.detail.contains("does not name its exact entity"));

    *plan
        .checkpoint
        .as_mut()
        .expect("checkpoint")
        .lineage
        .checkpoint_nodes_mut_for_test()[0]
        .lineage_node_entity_mut_for_test() =
        crate::identity::data::EntityId::new(crate::identity::data::PartitionId::main(), 99_999, 1);

    let error = recover_error(execution, plan);
    assert_eq!(error.class, RecoveryFailureClass::CorruptCheckpoint);
    assert!(error.detail.contains("does not name its exact entity"));
}

#[test]
fn checkpoint_lineage_node_swap_denies_before_installation() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut plan = two_entity_checkpoint_plan(execution, "checkpoint-swap");
    let nodes = plan
        .checkpoint
        .as_mut()
        .expect("checkpoint")
        .lineage
        .checkpoint_nodes_mut_for_test();
    assert_eq!(nodes.len(), 2);
    let first_entity = nodes[0].entity_id();
    *nodes[0].lineage_node_entity_mut_for_test() = nodes[1].entity_id();
    *nodes[1].lineage_node_entity_mut_for_test() = first_entity;

    let error = recover_error(execution, plan);
    assert_eq!(error.class, RecoveryFailureClass::CorruptCheckpoint);
    assert!(error.detail.contains("does not name its exact entity"));
}

#[test]
fn checkpoint_duplicate_lineage_entity_mapping_denies_before_installation() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut plan = two_entity_checkpoint_plan(execution, "checkpoint-duplicate-node");
    let nodes = plan
        .checkpoint
        .as_mut()
        .expect("checkpoint")
        .lineage
        .checkpoint_nodes_mut_for_test();
    assert_eq!(nodes.len(), 2);
    *nodes[1].lineage_node_entity_mut_for_test() = nodes[0].entity_id();

    let error = recover_error(execution, plan);
    assert_eq!(error.class, RecoveryFailureClass::CorruptCheckpoint);
    assert!(error.detail.contains("does not name its exact entity"));
}

fn two_entity_checkpoint_plan(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    prefix: &str,
) -> RecoveryPlan {
    let medium = OwnedTestDurableMedium::new();
    let runtime = durable_runtime_for_medium(&medium);
    let mut transaction = test_owner_begin_transaction_for_main(&runtime);
    transaction
        .push_batch(
            batch_create(&format!("{prefix}-first")),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("test staging stays within configured resource budgets");
    transaction
        .push_batch(
            batch_create(&format!("{prefix}-second")),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("test staging stays within configured resource budgets");
    transaction
        .commit(execution, &runtime)
        .expect("two-entity commit");
    medium.file_embedded_checkpoint(execution, &runtime);
    medium
        .recovery_plan(
            execution,
            &runtime,
            RecoveryMode::NormalRecoveryVerification,
        )
        .unwrap()
}

fn recover_error(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    plan: RecoveryPlan,
) -> crate::durability::data::DurabilityError {
    let medium = OwnedTestDurableMedium::new();
    let mut recovered = durable_runtime_for_medium(&medium);
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
