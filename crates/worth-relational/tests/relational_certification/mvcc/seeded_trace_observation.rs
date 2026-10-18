use super::{trace_panic, ProductionModelTrace};
use crate::relational_certification::world::supply_chain::{
    compare, observe_supply_chain_snapshot, ExpectedSupplyChainObservation, OracleBranch,
    ProductionSeededSupplyChainWorld,
};
use worth_relational::facade::snapshots::SnapshotHandle;

pub(super) fn observe_snapshot_against_oracle(
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
    let mut expected = ExpectedSupplyChainObservation::from_branch(oracle);
    if let Some(parent) = expected.ancestry.parent {
        expected.ancestry.lineage = vec![parent, expected.ancestry.branch];
    }
    expected.ancestry.accepted.clear();
    expected.ancestry.history.clear();
    compare(&expected, &observed)
        .unwrap_or_else(|error| trace_panic(trace, scenario_index, operation, error));
}
