use worth_relational::facade::change_source::RelationalChangeReceiptOutcome;
use worth_relational::facade::history::CommitId;
use worth_relational::facade::mvcc::RelationalBranchObservation;
use worth_relational::facade::runtime::RelationalRuntime;

fn mint(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    observation: &RelationalBranchObservation,
    commit_id: CommitId,
) -> Option<RelationalChangeReceiptOutcome> {
    let selected = runtime
        .select_reachable_commit(observation, commit_id)
        .into_outcome()
        .ok()?;
    Some(runtime.mint_change_receipt(execution, selected, None).ok()?)
}

fn main() {
    let _ = mint;
}
