pub(crate) use {worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::certified_baseline::certified_supply_chain_world, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scale::SupplyChainScale};
pub(crate) use {crate::relational_certification::ordinary_context::mvcc_branch_fork_fixture::fork_branch};
pub(crate) use {worth_relational::facade::branch::RelationalBranchBasisDenial, worth_relational::facade::branch::RelationalBranchLifecyclePosture, worth_relational::facade::branch::RelationalForkDenial};
pub(crate) use {worth_relational::facade::inspection::RelationalMvccCostScope};
pub(crate) use {worth_relational::facade::mvcc::RelationalBranchTransactionAdmissionDenial, worth_relational::facade::mvcc::RelationalPublicationDenial, worth_relational::facade::mvcc::RelationalPublicationOutcome, worth_relational::facade::mvcc::RelationalTransactionIntent};
pub(crate) use {worth_relational::facade::runtime::ProjectionAspectScope};

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
            worth_relational::facade::runtime::ProjectionAspectScope::empty(),
            |_| Some(()),
        )
        .expect("fixture read request remains live")
        .is_some()
}