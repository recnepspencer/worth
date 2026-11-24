mod aspect_field_reconciliation;
mod entity_replacement_reconciliation;
mod execution;
mod intent_reconciliation;
mod replica_convergence;

use worth_relational::facade::{
    runtime::RelationalRuntime, schema::DeclaredAspectContractBinding, snapshots::SnapshotHandle,
};

fn current_snapshot(runtime: &RelationalRuntime) -> SnapshotHandle {
    let (_, basis) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    runtime
        .snapshots()
        .snapshot_for_observation(&basis.observation())
        .unwrap()
}

fn lifecycle_aspect() -> DeclaredAspectContractBinding {
    let (binding, contract) =
        worth_testkit_relational::aspect_storage::definition::lifecycle_aspect();
    DeclaredAspectContractBinding { binding, contract }
}
