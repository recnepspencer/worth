use crate::facade::history::{BranchId, CommitId};
use crate::facade::merge::{MergeExecutionOutcome, MergeExecutionRequest, MergeIntent};
use crate::tests::support::{
    create_branch_from_main, create_entity_outcome, create_entity_outcome_on_branch,
    durable_runtime_with_test_schema,
};

pub(super) fn execute_feature_into_main_merge(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
) -> (
    crate::facade::runtime::RelationalRuntime,
    MergeExecutionOutcome,
    CommitId,
    CommitId,
) {
    let runtime = durable_runtime_with_test_schema();
    create_entity_outcome(execution, &runtime, "root");
    create_branch_from_main(&runtime, "feature");
    let feature_head = create_entity_outcome_on_branch(
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
    let main_head_commit_id = runtime
        .history()
        .branch_head(&BranchId("main".to_string()))
        .expect("main head before merge")
        .commit_id;
    let merge = runtime
        .execute_prepared_merge(execution, prepared)
        .expect("executed prepared merge");
    (
        runtime,
        merge,
        main_head_commit_id,
        feature_head.commit.commit_id,
    )
}
