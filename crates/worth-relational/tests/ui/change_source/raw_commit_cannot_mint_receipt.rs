use worth_relational::facade::history::CommitId;
use worth_relational::facade::runtime::RelationalRuntime;

fn mint(execution: &worth_execution::ExecutionRequest<'_, '_>, runtime: &RelationalRuntime) {
    let _ = runtime.mint_change_receipt(execution, CommitId(1), None);
}

fn main() {
    let _ = mint;
}
