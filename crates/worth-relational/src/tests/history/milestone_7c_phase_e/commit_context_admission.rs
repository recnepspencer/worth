use crate::facade::history::BranchId;
use crate::facade::merge::{MergeExecutionRequest, MergeIntent};
use crate::tests::support::{
    create_branch_from_main, create_entity_outcome, create_entity_outcome_on_branch,
    durable_runtime_with_test_schema,
};

#[test]
fn merge_commit_context_rejects_mismatched_parent_branch_metadata() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = durable_runtime_with_test_schema();
    create_entity_outcome(execution, &runtime, "root");
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
        .expect("prepared merge execution");
    let _mutation_plan =
        prepared.bind_mutation_plan_for_test(crate::facade::transactions::TransactionId(999));

    let error = runtime
        .prepare_merge_execution(
            execution,
            MergeExecutionRequest {
                target_branch: BranchId("main".to_string()),
                source_branch: BranchId("wrong".to_string()),
                merge_intent: MergeIntent::ReconcileIntoTarget,
            },
        )
        .expect_err("mismatched merge context should be rejected");

    match error {
        crate::facade::merge::MergeExecutionPreparationError::Planning(
            crate::facade::merge::MergePlanningError::MissingSourceHead { branch_id },
        ) => assert_eq!(branch_id, BranchId("wrong".to_string())),
        other => panic!("expected conflict error, got {other:?}"),
    }
}
