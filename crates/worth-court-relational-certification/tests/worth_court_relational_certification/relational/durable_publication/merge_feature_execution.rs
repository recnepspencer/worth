//! One feature-into-main prepared merge executed over an explicit durable medium.
use super::branch_history_publications::{create_entity_outcome_on_branch, fork};
use super::branch_history_recovery::current_head;
use std::sync::Arc;
use worth_execution::ExecutionRequest;
use worth_relational::facade::{
    history::{BranchId, CommitId},
    merge::{MergeExecutionOutcome, MergeExecutionRequest, MergeIntent},
    runtime::RelationalRuntime,
};
use worth_testkit_relational::{
    durable_medium::TestDurableMedium,
    durable_publication::{create_entity_outcome, durable_runtime},
};

/// The medium, its runtime, the published merge, the main head commit the
/// merge was prepared against and the feature head commit it adopted.
pub(super) fn execute_feature_into_main_merge(
    execution: &ExecutionRequest<'_, '_>,
) -> (
    Arc<TestDurableMedium>,
    RelationalRuntime,
    MergeExecutionOutcome,
    CommitId,
    CommitId,
) {
    let medium = TestDurableMedium::new();
    let runtime = durable_runtime(&medium);
    create_entity_outcome(execution, &runtime, "root");
    fork(execution, &runtime, "feature");
    let feature_head = create_entity_outcome_on_branch(
        execution,
        &runtime,
        "feature-only",
        BranchId("feature".to_string()),
    );
    let prepared = runtime
        .prepare_merge_execution(
            execution,
            runtime
                .bind_merge_execution_request(MergeExecutionRequest::new(
                    BranchId("main".to_string()),
                    BranchId("feature".to_string()),
                    MergeIntent::ReconcileIntoTarget,
                ))
                .unwrap(),
        )
        .expect("prepared merge execution");
    let main_head_commit_id = current_head(&runtime, &BranchId("main".to_string()))
        .expect("main head before merge")
        .commit_id;
    let merge = runtime
        .execute_prepared_merge(execution, prepared)
        .expect("executed prepared merge");
    (
        medium,
        runtime,
        merge,
        main_head_commit_id,
        feature_head.commit.commit_id,
    )
}
