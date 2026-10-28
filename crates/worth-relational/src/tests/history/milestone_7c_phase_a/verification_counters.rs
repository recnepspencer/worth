use crate::facade::history::BranchId;
use crate::facade::merge::{MergeExecutionRequest, MergeIntent};
use crate::tests::support::{
    create_branch_from_main, create_entity, create_entity_outcome_on_branch,
    durable_runtime_with_test_schema,
};

#[test]
fn verify_prepared_merge_execution_does_not_increment_planning_counters() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = durable_runtime_with_test_schema();
    create_entity(execution, &runtime, "root");
    create_branch_from_main(&runtime, "feature");
    create_entity_outcome_on_branch(
        execution,
        &runtime,
        "feature-only",
        BranchId("feature".to_string()),
    );

    let prepared = runtime
        .prepare_merge_execution(
            execution,
            MergeExecutionRequest {
                target_branch: BranchId("main".to_string()),
                source_branch: BranchId("feature".to_string()),
                merge_intent: MergeIntent::ReconcileIntoTarget,
            },
        )
        .expect("prepared merge");
    let before = runtime.performance_access().counters();

    runtime
        .merge()
        .verify_prepared_merge_execution(execution, &prepared)
        .expect("verification should succeed");

    let after = runtime.performance_access().counters();
    assert_eq!(
        before.merge_planning_requests,
        after.merge_planning_requests
    );
    assert_eq!(
        before.merge_planning_schema_kinds_snapshotted,
        after.merge_planning_schema_kinds_snapshotted
    );
    assert_eq!(
        before.merge_planning_elapsed_nanos,
        after.merge_planning_elapsed_nanos
    );
    assert_eq!(
        after.merge_execution_verification_requests,
        before.merge_execution_verification_requests + 1
    );
    assert_eq!(
        after.merge_execution_branch_head_checks,
        before.merge_execution_branch_head_checks + 2
    );
    assert_eq!(
        after.merge_execution_merge_base_checks,
        before.merge_execution_merge_base_checks + 1
    );
    assert_eq!(
        after.merge_execution_compiled_plan_digest_checks,
        before.merge_execution_compiled_plan_digest_checks + 1
    );
    assert!(
        after.merge_execution_schema_kinds_snapshotted
            >= before.merge_execution_schema_kinds_snapshotted
    );
}

#[test]
fn verify_prepared_merge_execution_reports_verification_counters_without_planning_work() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = durable_runtime_with_test_schema();
    create_entity(execution, &runtime, "root");
    create_branch_from_main(&runtime, "feature");
    create_entity_outcome_on_branch(
        execution,
        &runtime,
        "feature-only",
        BranchId("feature".to_string()),
    );

    let prepared = runtime
        .prepare_merge_execution(
            execution,
            MergeExecutionRequest {
                target_branch: BranchId("main".to_string()),
                source_branch: BranchId("feature".to_string()),
                merge_intent: MergeIntent::ReconcileIntoTarget,
            },
        )
        .expect("prepared merge");
    runtime.performance_access().reset_counters();

    runtime
        .merge()
        .verify_prepared_merge_execution(execution, &prepared)
        .expect("verification should succeed");

    let counters = runtime.performance_access().counters();
    assert_eq!(counters.merge_planning_requests, 0);
    assert_eq!(counters.merge_planning_schema_kinds_snapshotted, 0);
    assert_eq!(counters.merge_planning_elapsed_nanos, 0);
    assert_eq!(counters.merge_execution_verification_requests, 1);
    assert_eq!(counters.merge_execution_branch_head_checks, 2);
    assert_eq!(counters.merge_execution_merge_base_checks, 1);
    assert_eq!(counters.merge_execution_compiled_plan_digest_checks, 1);
    assert!(counters.merge_execution_schema_kinds_snapshotted > 0);
}
