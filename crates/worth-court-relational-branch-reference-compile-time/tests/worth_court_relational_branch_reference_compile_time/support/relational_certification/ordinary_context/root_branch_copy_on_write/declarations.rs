pub(crate) use {std::collections::BTreeSet};

pub(crate) use {worth_testkit_relational::relational_certification::ordinary_context::invariant_oracle_expectations::expected_supply_chain_branch};
pub(crate) use {worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::certified_baseline::assert_oracle_matches, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::certified_baseline::certified_supply_chain_world, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::runtime_driver::branch_operations::commit_branch_batch, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::comparison::compare, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::runtime_driver::lowering::lower_supply_chain_production_delta, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::observation::observe_snapshot as observe_supply_chain_snapshot, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::runtime_driver::branch_operations::snapshot_for_supply_chain_identity, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKey, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::expected_observation::ExpectedSupplyChainObservation, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKey, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKind, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scale::SupplyChainScale};
pub(crate) use {worth_relational::facade::branch::RelationalBranchIdentity};
pub(crate) use {worth_relational::facade::history::BranchId};
pub(crate) use {worth_relational::facade::inspection::RelationalAuthoritativeAllocationKind, worth_relational::facade::inspection::RelationalBranchSharingCostCounters, worth_relational::facade::inspection::RelationalBranchSharingObservation, worth_relational::facade::inspection::RelationalMvccCostScope, worth_relational::facade::inspection::RelationalStorageRegionLocator};
pub(crate) use {worth_relational::facade::runtime::RelationalRuntime};

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
    world: &worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::production_world::ProductionSeededSupplyChainWorld,
    delta: DeltaId,
) -> ExpectedSupplyChainObservation {
    expected_supply_chain_branch(&world.program, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::Rewire, Some(delta))
}

pub(crate) fn assert_rewire_matches_oracle(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    world: &mut worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::production_world::ProductionSeededSupplyChainWorld,
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