use worth_relational::facade::branch::{
    RelationalBranchReferenceObservation, RelationalForkOutcome,
};
use worth_relational::facade::history::BranchId;
use worth_relational::facade::runtime::RelationalRuntime;

fn attempt(
    runtime: &RelationalRuntime,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    outcome: RelationalForkOutcome,
) {
    let provenance: &RelationalBranchReferenceObservation = outcome.fork_provenance();
    let _ = runtime.fork_branch(execution, BranchId("target".to_owned()), provenance);
}

fn main() {}
