use crate::world::supply_chain::{
    hazard_v2_transition, CompiledSupplyChainProgram, DeltaId, SchemaVersion,
};
use worth_relational::facade::branch::RelationalBranchIdentity;
use worth_relational::facade::history::{BranchId, RelationalCommitReceipt};
use worth_relational::facade::runtime::RelationalRuntime;
use worth_relational::facade::snapshots::SnapshotHandle;
use worth_relational::facade::transactions::{CommitResult, WorkerIntentBatch};

pub(crate) fn commit_main_batch(runtime: &RelationalRuntime, batch: WorkerIntentBatch) {
    commit_branch_batch(runtime, BranchId("main".to_owned()), batch);
}

pub(crate) fn fork_supply_chain_branch_from_main(runtime: &RelationalRuntime, branch_id: BranchId) {
    let (_, source) = runtime
        .observe_fork_source(&BranchId("main".to_owned()))
        .expect("main remains an admitted fork source");
    runtime
        .fork_branch(branch_id, source)
        .expect("the Supply Chain branch forks from the admitted main basis");
}

pub(crate) fn snapshot_for_supply_chain_identity(
    runtime: &RelationalRuntime,
    identity: &RelationalBranchIdentity,
) -> SnapshotHandle {
    let (_, basis) = runtime
        .observe_branch(identity)
        .expect("branch basis is owner-admitted");
    runtime
        .snapshots()
        .snapshot_for_observation(&basis.observation())
        .expect("admitted branch observation opens its exact snapshot")
}

pub(crate) fn head_for_supply_chain_identity(
    runtime: &RelationalRuntime,
    identity: &RelationalBranchIdentity,
) -> RelationalCommitReceipt {
    let (_, basis) = runtime
        .observe_branch(identity)
        .expect("branch basis is owner-admitted");
    runtime
        .history()
        .branch_head_for_observation(&basis.observation())
        .expect("observation belongs to this runtime")
        .expect("observed Supply Chain branch has a canonical head")
}

pub(crate) fn head_for_supply_chain_branch(
    runtime: &RelationalRuntime,
    branch_id: &BranchId,
) -> RelationalCommitReceipt {
    let identity = runtime
        .branch_identity(branch_id)
        .expect("branch identity is owner-issued");
    head_for_supply_chain_identity(runtime, &identity)
}

pub(crate) fn commit_branch_batch(
    runtime: &RelationalRuntime,
    branch_id: BranchId,
    batch: WorkerIntentBatch,
) {
    let _ = commit_branch_batch_with_result(runtime, branch_id, batch);
}

pub(crate) fn commit_branch_batch_with_result(
    runtime: &RelationalRuntime,
    branch_id: BranchId,
    batch: WorkerIntentBatch,
) -> CommitResult {
    commit_batch_with_intent(
        runtime,
        branch_id,
        batch,
        worth_relational::facade::mvcc::RelationalTransactionIntent::ordinary(),
    )
}

pub(crate) fn commit_supply_chain_delta(
    runtime: &RelationalRuntime,
    program: &CompiledSupplyChainProgram,
    branch_id: BranchId,
    delta: DeltaId,
    batch: WorkerIntentBatch,
) -> CommitResult {
    let identity = runtime
        .branch_identity(&branch_id)
        .expect("branch identity is owner-issued");
    let basis = runtime
        .admit_branch_basis(&identity)
        .expect("transaction authority is owner-issued");
    let mut transaction = if delta == DeltaId::AdoptHazardClassificationV2 {
        let transition = hazard_v2_transition();
        let target_registry = program
            .schema_registry_for_version(SchemaVersion::V2)
            .expect("the immutable Supply Chain V2 target registry compiles");
        runtime
            .begin_branch_schema_transition(&basis, transition, None, target_registry)
            .expect("the declared Supply Chain V1-to-V2 transaction is owner-admitted")
    } else {
        runtime
            .begin_branch_transaction(
                &basis,
                worth_relational::facade::mvcc::RelationalTransactionIntent::ordinary(),
            )
            .expect("owner-admitted transaction context")
    };
    transaction
        .push_batch(
            batch,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("Supply Chain mutation staging fits its configured transaction budget");
    transaction
        .commit(
            runtime,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("Supply Chain mutation commits through production publication")
}

fn commit_batch_with_intent(
    runtime: &RelationalRuntime,
    branch_id: BranchId,
    batch: WorkerIntentBatch,
    intent: worth_relational::facade::mvcc::RelationalTransactionIntent,
) -> CommitResult {
    let identity = runtime
        .branch_identity(&branch_id)
        .expect("branch identity is owner-issued");
    let basis = runtime
        .admit_branch_basis(&identity)
        .expect("transaction authority is owner-issued");
    let mut transaction = runtime
        .begin_branch_transaction(&basis, intent)
        .expect("owner-admitted transaction context");
    transaction
        .push_batch(
            batch,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("Supply Chain mutation staging fits its configured transaction budget");
    transaction
        .commit(
            runtime,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("Supply Chain mutation commits through production publication")
}
