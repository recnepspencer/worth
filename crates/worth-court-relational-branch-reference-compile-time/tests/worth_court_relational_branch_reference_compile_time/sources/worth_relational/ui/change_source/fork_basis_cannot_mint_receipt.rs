use worth_relational::facade::branch::AdmittedRelationalForkSourceBasis;
use worth_relational::facade::runtime::RelationalRuntime;

fn mint(execution: &worth_execution::ExecutionRequest<'_, '_>, runtime: &RelationalRuntime, basis: AdmittedRelationalForkSourceBasis) {
    let _ = runtime.mint_change_receipt(execution, basis, None);
}

fn main() {
    let _ = mint;
}
