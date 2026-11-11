//! Prepared merge verification accepts a fresh preparation and refuses instance or head drift.
use super::branch_history_publications::{create_entity, create_entity_outcome_on_branch};
use super::merge_proof_journey::{merge_ready_runtime, merge_request};
use worth_relational::facade::{
    history::BranchId,
    merge::{MergeExecutionError, PreparedMergeExecution},
    runtime::RelationalRuntime,
};
use worth_testkit_relational::durable_medium::TestDurableMedium;

#[test]
fn verify_prepared_merge_execution_accepts_fresh_prepared_merge() {
    let host_request = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let medium = TestDurableMedium::new();
    let runtime = merge_ready_runtime(execution, &medium);

    let prepared = prepare_feature_merge(execution, &runtime);

    runtime
        .merge()
        .verify_prepared_merge_execution(execution, &prepared)
        .expect("fresh prepared merge should verify");
}

#[test]
fn verify_prepared_merge_execution_rejects_runtime_instance_mismatch() {
    let host_request = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let medium = TestDurableMedium::new();
    let runtime = merge_ready_runtime(execution, &medium);

    let prepared = prepare_feature_merge(execution, &runtime);
    let forked = runtime.fork().expect("settled runtime forks");

    match forked
        .merge()
        .verify_prepared_merge_execution(execution, &prepared)
    {
        Err(MergeExecutionError::RuntimeInstanceMismatch { .. }) => {}
        other => panic!("expected runtime instance mismatch, got {other:?}"),
    }
}

#[test]
fn verify_prepared_merge_execution_rejects_target_head_drift() {
    let host_request = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let medium = TestDurableMedium::new();
    let runtime = merge_ready_runtime(execution, &medium);

    let prepared = prepare_feature_merge(execution, &runtime);

    create_entity(execution, &runtime, "main-advance");

    match runtime
        .merge()
        .verify_prepared_merge_execution(execution, &prepared)
    {
        Err(MergeExecutionError::StaleBranchHead { branch, .. }) => {
            assert_eq!(branch, BranchId("main".to_string()));
        }
        other => panic!("expected target stale-head rejection, got {other:?}"),
    }
}

#[test]
fn verify_prepared_merge_execution_rejects_source_head_drift() {
    let host_request = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let medium = TestDurableMedium::new();
    let runtime = merge_ready_runtime(execution, &medium);

    let prepared = prepare_feature_merge(execution, &runtime);

    create_entity_outcome_on_branch(
        execution,
        &runtime,
        "feature-advance",
        BranchId("feature".to_string()),
    );

    match runtime
        .merge()
        .verify_prepared_merge_execution(execution, &prepared)
    {
        Err(MergeExecutionError::StaleBranchHead { branch, .. }) => {
            assert_eq!(branch, BranchId("feature".to_string()));
        }
        other => panic!("expected source stale-head rejection, got {other:?}"),
    }
}

/// The runtime-surface preparation of the fixture's feature-into-main merge.
pub(super) fn prepare_feature_merge(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
) -> PreparedMergeExecution {
    runtime
        .prepare_merge_execution(
            execution,
            runtime
                .bind_merge_execution_request(merge_request())
                .unwrap(),
        )
        .expect("prepared merge")
}
