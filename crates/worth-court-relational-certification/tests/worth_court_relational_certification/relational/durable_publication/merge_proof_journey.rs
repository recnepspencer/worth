use super::branch_history_publications::*;
use std::sync::Arc;
use worth_execution::ExecutionRequest;
use worth_relational::facade::{
    history::*, merge::*, runtime::RelationalRuntime,
    transactions::PublishedMergeExecutionAuthority,
};
use worth_testkit_relational::{
    durable_medium::TestDurableMedium, durable_publication::durable_runtime,
};
pub(super) fn merge_request() -> MergeExecutionRequest {
    MergeExecutionRequest::new(
        BranchId("main".to_string()),
        BranchId("feature".to_string()),
        MergeIntent::ReconcileIntoTarget,
    )
}
pub(super) fn execute_merge(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
) -> MergeExecutionOutcome {
    let prepared = runtime
        .merge()
        .prepare_merge_execution(
            execution,
            runtime
                .bind_merge_execution_request(merge_request())
                .unwrap(),
        )
        .expect("prepared merge");
    runtime
        .execute_prepared_merge(execution, prepared)
        .expect("executed merge")
}
pub(super) fn published_merge_authority(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    commit_id: worth_relational::facade::history::CommitId,
) -> PublishedMergeExecutionAuthority {
    runtime
        .replay()
        .canonical_commit_envelope(execution, commit_id)
        .unwrap()
        .and_then(|envelope| envelope.merge_execution_authority.clone())
        .expect("published merge authority")
}
pub(super) fn merge_ready_runtime(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    medium: &Arc<TestDurableMedium>,
) -> RelationalRuntime {
    let runtime = durable_runtime(medium);
    create_entity(execution, &runtime, "root");
    fork(execution, &runtime, "feature");
    create_entity_outcome_on_branch(
        execution,
        &runtime,
        "feature-only",
        BranchId("feature".to_string()),
    );
    runtime
}
pub(super) fn checkpoint_and_recover(
    request: &ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    medium: &Arc<TestDurableMedium>,
) -> (
    worth_relational::facade::durability::RecoveryOutcome,
    RelationalRuntime,
) {
    super::branch_history_recovery::checkpoint(request, runtime, medium);
    super::branch_history_recovery::recover(request, medium, durable_runtime(medium))
}
pub(super) fn actual_basis(
    runtime: &RelationalRuntime,
    source: &str,
    target: &str,
) -> RelationalMergeBranchBasis {
    let (_, source_basis) = runtime
        .observe_branch(&runtime.branch_identity(&BranchId(source.into())).unwrap())
        .unwrap();
    let (_, target_basis) = runtime
        .observe_branch(&runtime.branch_identity(&BranchId(target.into())).unwrap())
        .unwrap();
    runtime
        .history()
        .merge_branch_basis_for_observations(
            &source_basis.observation(),
            &target_basis.observation(),
        )
        .unwrap()
}
