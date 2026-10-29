pub(crate) use std::collections::BTreeSet;

pub(crate) use super::invariant_oracle_expectations::expected_supply_chain_branch;
pub(crate) use super::world::supply_chain::{
    assert_oracle_matches, certified_supply_chain_world, commit_branch_batch, compare,
    lower_supply_chain_production_delta, observe_supply_chain_snapshot,
    snapshot_for_supply_chain_identity, BranchLabel, DeltaId, EntityKey, EntityKind,
    ExpectedSupplyChainObservation, RelationKey, RelationKind, SupplyChainScale,
};
pub(crate) use worth_relational::facade::branch::RelationalBranchIdentity;
pub(crate) use worth_relational::facade::history::BranchId;
pub(crate) use worth_relational::facade::inspection::{
    RelationalAuthoritativeAllocationKind, RelationalBranchSharingCostCounters,
    RelationalBranchSharingObservation, RelationalMvccCostScope, RelationalStorageRegionLocator,
};
pub(crate) use worth_relational::facade::runtime::RelationalRuntime;

pub(crate) fn fork_from_main(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    branch: &str,
) {
    let (_, source) = runtime
        .observe_fork_source(&BranchId("main".to_owned()))
        .expect("main remains a fork source");
    runtime
        .fork_branch(execution, BranchId(branch.to_owned()), source)
        .expect("fork retains the immutable ancestor root");
}

pub(crate) fn branch_identity(
    runtime: &RelationalRuntime,
    branch: &str,
) -> RelationalBranchIdentity {
    runtime
        .branch_identity(&BranchId(branch.to_owned()))
        .expect("branch identity is owner-issued")
}

pub(crate) fn sharing(
    runtime: &RelationalRuntime,
    branches: &[RelationalBranchIdentity],
) -> RelationalBranchSharingObservation {
    runtime
        .observe_branch_sharing(branches)
        .expect("owner-bound sharing inspection succeeds")
}

pub(crate) fn region_set(
    observation: &RelationalBranchSharingObservation,
) -> BTreeSet<RelationalStorageRegionLocator> {
    observation.region_locators().iter().copied().collect()
}

pub(crate) fn expected_rewire_from_pure_oracle(
    world: &super::world::supply_chain::ProductionSeededSupplyChainWorld,
    delta: DeltaId,
) -> ExpectedSupplyChainObservation {
    expected_supply_chain_branch(&world.program, BranchLabel::Rewire, Some(delta))
}

pub(crate) fn assert_rewire_matches_oracle(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    world: &mut super::world::supply_chain::ProductionSeededSupplyChainWorld,
    rewire: &RelationalBranchIdentity,
    expected: ExpectedSupplyChainObservation,
) {
    let snapshot = snapshot_for_supply_chain_identity(&world.runtime, rewire);
    let observed = observe_supply_chain_snapshot(
        execution,
        &world.program,
        &world.handles.for_snapshot(snapshot.clone()),
        &world.runtime,
        &snapshot,
    )
    .expect("production rewire remains observable");
    compare(&expected, &observed).expect("production state matches the independent pure oracle");
}
