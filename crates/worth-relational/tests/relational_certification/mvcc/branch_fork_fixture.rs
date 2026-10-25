use worth_relational::facade::branch::RelationalBranchIdentity;
use worth_relational::facade::history::BranchId;
use worth_relational::facade::runtime::RelationalRuntime;

pub(crate) fn fork_branch(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    target: &str,
) -> RelationalBranchIdentity {
    let (_, basis) = runtime
        .observe_fork_source(&BranchId("main".to_owned()))
        .unwrap();
    runtime
        .fork_branch(execution, BranchId(target.to_owned()), basis)
        .unwrap();
    runtime
        .branch_identity(&BranchId(target.to_owned()))
        .unwrap()
}
