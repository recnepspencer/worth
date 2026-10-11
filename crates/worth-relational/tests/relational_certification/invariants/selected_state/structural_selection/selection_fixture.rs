use super::*;

pub(crate) fn observe_branch_snapshot(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    program: &CompiledSupplyChainProgram,
    runtime: &RelationalRuntime,
    handles: &super::super::world::supply_chain::SupplyChainSemanticHandles,
    branch: &str,
) -> super::super::world::supply_chain::ObservedSupplyChainState {
    let identity = runtime
        .branch_identity(&BranchId(branch.to_owned()))
        .expect("branch identity is owner-issued");
    let snapshot = snapshot_for_supply_chain_identity(runtime, &identity);
    observe_supply_chain_snapshot(
        execution,
        program,
        &handles.for_snapshot(snapshot.clone()),
        runtime,
        &snapshot,
    )
    .expect("branch snapshot remains observable")
}

pub(crate) fn commit_branch_batch_result(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    branch_id: BranchId,
    batch: WorkerIntentBatch,
) -> worth_relational::facade::transactions::CommitResult {
    let identity = runtime
        .branch_identity(&branch_id)
        .expect("branch identity is owner-issued");
    let options = runtime
        .admit_branch_basis(&identity)
        .expect("transaction options are owner-issued");
    let mut transaction = runtime
        .begin_branch_transaction(
            &options,
            worth_relational::facade::mvcc::RelationalTransactionIntent::ordinary(),
        )
        .expect("owner-admitted transaction context");
    transaction
        .push_batch(
            batch,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    transaction
        .commit(
            execution,
            runtime,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("branch batch commits through production publication")
}
