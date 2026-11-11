use super::checkpoint_index_generations::{
    build, fork_branch, generation, name_index, scoped_name_index,
};
use super::checkpoint_source_capture::capture_durably;
use worth_relational::facade::{
    durability::RecoveryVerificationMode as RecoveryMode, indexes::DerivedIndexBuildRequest,
};
use worth_testkit_relational::{
    aspect_storage::publications::release_test_commit_snapshot,
    durable_medium::TestDurableMedium,
    durable_publication::{create_entity_outcome, durable_runtime},
};
#[test]
fn live_fork_scoped_generation_readmits_from_shared_basis() {
    let host_request = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let medium = TestDurableMedium::new();
    let runtime = durable_runtime(&medium);
    let anchor = create_entity_outcome(execution, &runtime, "scoped-fork-checkpoint-anchor");
    let branch = fork_branch(execution, &runtime, "scoped-fork-checkpoint");
    let identity = runtime.branch_identity(&branch).unwrap();
    let (_, basis) = runtime.observe_branch(&identity).unwrap();
    let index = scoped_name_index(&runtime);
    let built = runtime.index_authority().build_for_basis(
        execution,
        DerivedIndexBuildRequest {
            source_commit_id: anchor.commit.commit_id,
            branch_id: branch.clone(),
            index_ids: vec![index.index_id],
        },
        &basis,
    );
    assert!(built.failed_indexes.is_empty());
    let generation = built.generations[0].generation_id;
    drop(basis);
    release_test_commit_snapshot(&runtime, &anchor);
    capture_durably(execution, &runtime, &medium).unwrap();
    let plan = runtime
        .durability()
        .recovery_plan_from_source(
            execution,
            medium.recovery_source(),
            RecoveryMode::NormalRecoveryVerification,
        )
        .unwrap();
    let mut recovered = durable_runtime(&medium);
    recovered
        .durability_recovery()
        .recover(execution, plan)
        .unwrap();
    assert!(
        super::checkpoint_index_generations::generation(execution, &recovered, generation)
            .is_some()
    );
}

#[test]
fn checkpoint_index_artifact_survives_tail_replay_without_envelope_caches() {
    let host_request = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let medium = TestDurableMedium::new();
    let runtime = durable_runtime(&medium);
    let first = create_entity_outcome(execution, &runtime, "checkpoint-index-first");
    let index = name_index(&runtime);
    let first_generation = build(execution, &runtime, index.index_id, &first);
    let _checkpoint = capture_durably(execution, &runtime, &medium).unwrap();
    let second = create_entity_outcome(execution, &runtime, "checkpoint-index-tail");
    let unjournaled_cache = build(execution, &runtime, index.index_id, &second);
    let plan = runtime
        .durability()
        .recovery_plan_from_source(
            execution,
            medium.recovery_source(),
            RecoveryMode::NormalRecoveryVerification,
        )
        .unwrap();
    let mut recovered = durable_runtime(&medium);
    recovered
        .durability_recovery()
        .recover(execution, plan)
        .unwrap();
    assert!(generation(execution, &recovered, first_generation).is_some());
    assert!(generation(execution, &recovered, unjournaled_cache).is_none());
    let rebuilt = build(execution, &recovered, index.index_id, &second);
    assert!(generation(execution, &recovered, rebuilt).is_some());
}

#[test]
fn checkpoint_and_new_tail_index_definitions_preserve_predecessor_generation_identity() {
    let host_request = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);
    let medium = TestDurableMedium::new();
    let runtime = durable_runtime(&medium);
    let first = create_entity_outcome(execution, &runtime, "mixed-index-checkpoint");
    let retained = name_index(&runtime);
    let first_generation = build(execution, &runtime, retained.index_id, &first);
    capture_durably(execution, &runtime, &medium).unwrap();

    let introduced = scoped_name_index(&runtime);
    build(execution, &runtime, introduced.index_id, &first);
    let second = create_entity_outcome(execution, &runtime, "mixed-index-tail");
    let expected = [retained.index_id, introduced.index_id].map(|index| {
        runtime
            .index_access()
            .read_published_generation_for_commit(execution, index, &second.commit)
            .unwrap()
            .unwrap()
    });
    let unjournaled = build(execution, &runtime, retained.index_id, &second);
    let plan = runtime
        .durability()
        .recovery_plan_from_source(
            execution,
            medium.recovery_source(),
            RecoveryMode::NormalRecoveryVerification,
        )
        .unwrap();
    let mut recovered = durable_runtime(&medium);
    recovered
        .durability_recovery()
        .recover(execution, plan)
        .unwrap();

    assert!(generation(execution, &recovered, first_generation).is_some());
    assert!(generation(execution, &recovered, unjournaled).is_none());
    for expected in expected {
        let actual = recovered
            .index_access()
            .read_published_generation_for_commit(execution, expected.index_id, &second.commit)
            .unwrap()
            .unwrap();
        assert_eq!(actual.as_ref(), expected.as_ref());
    }
}
