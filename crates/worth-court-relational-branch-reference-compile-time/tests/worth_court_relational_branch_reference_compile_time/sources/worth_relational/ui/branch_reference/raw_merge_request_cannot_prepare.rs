use worth_relational::facade::history::BranchId;
use worth_relational::facade::merge::{MergeExecutionRequest, MergeIntent};
use worth_relational::facade::runtime::RelationalRuntime;

pub fn attempt(runtime: &RelationalRuntime, execution: &worth_execution::ExecutionRequest<'_, '_>) {
    let request = MergeExecutionRequest::new(
        BranchId("main".to_owned()),
        BranchId("feature".to_owned()),
        MergeIntent::ReconcileIntoTarget,
    );
    let _ = runtime.prepare_merge_execution(execution, request);
}

fn main() {}
