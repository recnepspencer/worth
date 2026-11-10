//! The fixture's actual owner-bound feature reconciliation planning entry.
use worth_execution::ExecutionRequest;
use worth_relational::facade::{
    history::BranchId,
    merge::{MergeIntent, MergePlanningArtifactCore, MergePlanningRequest},
    runtime::RelationalRuntime,
};
pub(super) fn inspect_feature_reconciliation(
    request: &ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
) -> MergePlanningArtifactCore {
    let bound = runtime
        .bind_merge_planning_request(MergePlanningRequest::new(
            BranchId("main".into()),
            BranchId("feature".into()),
            MergeIntent::ReconcileIntoTarget,
        ))
        .expect("the fixture registered both actual branches");
    runtime
        .merge()
        .inspect_planning_scope(request, bound)
        .expect("planning artifact")
}
