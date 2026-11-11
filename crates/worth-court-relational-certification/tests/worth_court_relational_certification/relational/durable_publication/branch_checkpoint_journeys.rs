//! Public adjacency, fork tails and replay across real checkpoint recovery.
use super::branch_history_publications::*;
use super::branch_history_recovery::{checkpoint, main_version, recover};
use worth_relational::facade::{
    history::BranchId,
    replay::{RelationalReplayRequest, ReplayExecutionMode, ReplayVerificationMode},
    runtime::RelationalRuntime,
};
use worth_testkit_relational::{
    durable_medium::TestDurableMedium,
    durable_publication::{create_entity_outcome, durable_runtime},
};
#[test]
fn checkpoint_rebuilds_current_and_historical_adjacency_kind_buckets() {
    let host_request = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let medium = TestDurableMedium::new();
    let runtime = durable_runtime(&medium);
    let source = create_entity(execution, &runtime, "source");
    let target = create_entity(execution, &runtime, "target");
    let created =
        create_relation_outcome(execution, &runtime, source, target, "checkpoint-relation");
    let relation = changed_relations(&created)[0];
    let historical_version = created.version_id;
    let _deleted =
        delete_relation_on_branch(execution, &runtime, relation, BranchId("main".to_string()));

    checkpoint(execution, &runtime, &medium);
    let (_, recovered) = recover(execution, &medium, durable_runtime(&medium));
    let relation_kind = worth_relational::facade::identity::KindId(2);
    let historical = recovered
        .read_truth()
        .bounded_outgoing_relations_of_kind_at_version(
            execution,
            source,
            relation_kind,
            historical_version,
            2,
        )
        .expect("read request remains live")
        .expect("recovery must rebuild retained historical kind membership");
    let current = recovered
        .read_truth()
        .bounded_outgoing_relations_of_kind_at_version(
            execution,
            source,
            relation_kind,
            main_version(&recovered),
            1,
        )
        .expect("read request remains live")
        .expect("deleted current adjacency must be empty without scanning history");

    assert_eq!(historical.into_records()[0].relation_id, relation);
    assert!(current.into_records().is_empty());
}

#[test]
fn a_fork_commits_after_a_checkpoint_taken_once_its_source_moved_on() {
    let host_request = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);
    let medium = TestDurableMedium::new();
    let runtime = durable_runtime(&medium);
    let storm = BranchId("storm".to_owned());
    create_entity_outcome(execution, &runtime, "source-a");
    let (_, basis) = runtime
        .observe_fork_source(&BranchId("main".to_owned()))
        .expect("committed main is forkable");
    runtime
        .fork_branch(execution, storm.clone(), basis)
        .expect("the fork is admitted");
    create_entity_outcome_on_branch(execution, &runtime, "storm-a", storm.clone());
    create_entity_outcome(execution, &runtime, "source-b");
    checkpoint(execution, &runtime, &medium);
    create_entity_outcome_on_branch(execution, &runtime, "storm-b", storm.clone());

    let (outcome, recovered) = recover(execution, &medium, durable_runtime(&medium));
    assert_eq!(outcome.recovered_commits, 4);
    assert!(recovered.branch_identity(&storm).is_ok());
    assert_eq!(
        recovered.history().historical_latest_commit(),
        runtime.history().historical_latest_commit()
    );
}

#[test]
fn partial_replay_includes_an_intermediate_fork_without_its_own_commit() {
    let request = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&request);
    let medium = TestDurableMedium::new();
    let runtime = durable_runtime(&medium);
    create_entity(execution, &runtime, "source");
    fork(execution, &runtime, "A");
    let (_, source) = runtime.observe_fork_source(&BranchId("A".into())).unwrap();
    runtime
        .fork_branch(execution, BranchId("B".into()), source)
        .unwrap();
    let target =
        create_entity_outcome_on_branch(execution, &runtime, "target", BranchId("B".into()));
    let verify = |runtime: &RelationalRuntime| {
        let replay = runtime
            .replay_authority()
            .replay_commit(
                RelationalReplayRequest {
                    commit_id: target.commit.commit_id,
                    branch_id: BranchId("B".into()),
                    execution_mode: ReplayExecutionMode::SerialDeterministic,
                    verification_mode: ReplayVerificationMode::AuditRecoveryVerification,
                },
                execution,
            )
            .unwrap();
        assert!(replay.mismatches.is_empty(), "{replay:?}");
    };
    verify(&runtime);
    checkpoint(execution, &runtime, &medium);
    assert!(runtime
        .durability()
        .branch_transition_artifacts(execution.allocation_policy())
        .next()
        .is_none());
    let (_, recovered) = recover(execution, &medium, durable_runtime(&medium));
    verify(&recovered);
}
