use worth_relational::facade::history::BranchId;
use worth_relational::facade::inspection::{
    RelationalBranchSharingObservation, RelationalOwnerAllocationLedgerObservation,
    RelationalVisibilityCommitmentObservation,
};
use worth_relational::facade::mvcc::RelationalTransactionIntent;
use worth_relational::facade::runtime::RelationalRuntime;

fn sharing_cannot_fork(
    runtime: &RelationalRuntime,
    observation: RelationalBranchSharingObservation,
) {
    let _ = runtime.fork_branch(BranchId("forged".to_owned()), observation);
}

fn allocation_cannot_begin_mutation(
    runtime: &RelationalRuntime,
    observation: RelationalOwnerAllocationLedgerObservation,
) {
    let _ = runtime.begin_branch_transaction(&observation, RelationalTransactionIntent::ordinary());
}

fn visibility_cannot_retain(
    runtime: &RelationalRuntime,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    observation: RelationalVisibilityCommitmentObservation,
) {
    let _ = runtime
        .history_authority()
        .retain_version_for_replay(execution, observation);
}

fn main() {}
