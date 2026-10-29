pub(crate) use super::world::supply_chain::{certified_supply_chain_world, SupplyChainScale};
pub(crate) use crate::fast::relational_certification::mvcc_branch_fork_fixture::fork_branch;
pub(crate) use worth_relational::facade::branch::{
    RelationalBranchBasisDenial, RelationalBranchLifecyclePosture, RelationalForkDenial,
};
pub(crate) use worth_relational::facade::inspection::RelationalMvccCostScope;
pub(crate) use worth_relational::facade::mvcc::{
    RelationalBranchTransactionAdmissionDenial, RelationalPublicationDenial,
    RelationalPublicationOutcome, RelationalTransactionIntent,
};
pub(crate) use worth_relational::facade::runtime::ProjectionAspectScope;

pub(crate) fn snapshot_contains_entity(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    entity: worth_relational::facade::identity::EntityId,
) -> bool {
    runtime
        .read_truth()
        .project_snapshot(snapshot)
        .unwrap()
        .entity_record_with_projection_scope(
            execution,
            entity,
            ProjectionAspectScope::empty(),
            |_| Some(()),
        )
        .expect("fixture read request remains live")
        .is_some()
}
