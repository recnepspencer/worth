use worth_relational::facade::branch::AdmittedRelationalForkSourceBasis;
use worth_relational::facade::runtime::RelationalRuntime;

fn cannot_retain_fork_basis(
    runtime: &RelationalRuntime,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    basis: AdmittedRelationalForkSourceBasis,
) {
    let _ = runtime
        .history_authority()
        .retain_version_for_replay(execution, basis);
}

fn main() {}
