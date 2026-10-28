use worth_relational::facade::branch::RelationalForkSourceDescriptor;
use worth_relational::facade::history::BranchId;
use worth_relational::facade::runtime::RelationalRuntime;

fn attempt(
    runtime: &RelationalRuntime,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    descriptor: RelationalForkSourceDescriptor,
) {
    let _ = runtime.fork_branch(execution, BranchId("storm".to_owned()), descriptor);
}

fn main() {}
