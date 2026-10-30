pub(crate) use {crate::relational_certification::reference_context::mvcc_seeded_model::trace_panic, crate::relational_certification::reference_context::mvcc_seeded_model::ProductionModelTrace};
pub(crate) use {worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::comparison::compare, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::observation::observe_snapshot as observe_supply_chain_snapshot, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::expected_observation::ExpectedSupplyChainObservation, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::oracle::ancestry::OracleBranch, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::production_world::ProductionSeededSupplyChainWorld};
pub(crate) use {worth_relational::facade::snapshots::SnapshotHandle};

pub(crate) fn observe_snapshot_against_oracle(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    world: &ProductionSeededSupplyChainWorld,
    trace: &ProductionModelTrace,
    scenario_index: usize,
    snapshot: &SnapshotHandle,
    oracle: &OracleBranch,
    operation: &'static str,
) {
    let observed = observe_supply_chain_snapshot(
        execution,
        &world.program,
        &world.handles.for_snapshot(snapshot.clone()),
        &world.runtime,
        snapshot,
    )
    .unwrap_or_else(|error| trace_panic(trace, scenario_index, operation, error));
    let mut expected = worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::expected_observation::ExpectedSupplyChainObservation::from_branch(oracle);
    if let Some(parent) = expected.ancestry.parent {
        expected.ancestry.lineage = vec![parent, expected.ancestry.branch];
    }
    expected.ancestry.accepted.clear();
    expected.ancestry.history.clear();
    compare(&expected, &observed)
        .unwrap_or_else(|error| trace_panic(trace, scenario_index, operation, error));
}
