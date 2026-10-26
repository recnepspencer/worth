use worth_relational::facade::branch::AdmittedRelationalForkSourceBasis;
use worth_relational::facade::history::BranchId;
use worth_relational::facade::runtime::RelationalRuntime;

fn attempt(
    runtime: &RelationalRuntime,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    basis: AdmittedRelationalForkSourceBasis,
) {
    let _ = runtime.fork_branch(execution, BranchId("first".to_owned()), basis);
    let _ = runtime.fork_branch(execution, BranchId("second".to_owned()), basis);
}

fn main() {}
