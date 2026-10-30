use worth_proof::{AuthorityMarker, AuthorityWitness};
use worth_relational::facade::branch::AdmittedRelationalForkSourceBasis;
use worth_relational::facade::history::BranchId;
use worth_relational::facade::runtime::RelationalRuntime;

fn generic_door<Auth: AuthorityMarker>(
    runtime: &RelationalRuntime,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    target: BranchId,
    authority: AuthorityWitness<Auth>,
) {
    let _basis: AdmittedRelationalForkSourceBasis = authority;
    let _ = runtime.fork_branch(execution, target, _basis);
}

fn main() {}
