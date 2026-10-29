pub(crate) use {std::collections::BTreeSet};

pub(crate) use {worth_testkit_relational::relational_certification::ordinary_context::invariant_oracle_expectations::expected_supply_chain_branch};
pub(crate) use {worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::certified_baseline::certified_supply_chain_world, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::comparison::compare, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::runtime_driver::lowering::lower_supply_chain_production_delta, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::observation::observe_observation as observe_supply_chain_observation, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::production_world::ProductionSeededSupplyChainWorld, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scale::SupplyChainScale};
pub(crate) use {worth_relational::facade::branch::AdmittedRelationalBranchBasis, worth_relational::facade::branch::RelationalBranchIdentity, worth_relational::facade::branch::RelationalOwnerServicePorts};
pub(crate) use {worth_relational::facade::history::BranchId};
pub(crate) use {worth_relational::facade::mvcc::RelationalBranchTransactionAdmissionDenial, worth_relational::facade::mvcc::RelationalPublicationOutcome, worth_relational::facade::mvcc::RelationalTransactionIntent};

pub(crate) fn fork_supply_chain_branch(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    services: &RelationalOwnerServicePorts,
    branch: &str,
) -> RelationalBranchIdentity {
    let (_, source) = services
        .fork_port()
        .observe_fork_source(&BranchId("main".to_owned()))
        .expect("the non-empty Supply Chain main branch is a fork source");
    services
        .fork_port()
        .fork_branch(execution, BranchId(branch.to_owned()), source)
        .expect("the fork port installs the exact Supply Chain branch")
        .target_identity()
        .clone()
}

pub(crate) fn observe_basis(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    world: &ProductionSeededSupplyChainWorld,
    basis: &AdmittedRelationalBranchBasis,
) -> worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::comparison_state::ObservedSupplyChainState {
    let observation = basis.observation();
    let handles = world.handles.for_observation(&observation);
    observe_supply_chain_observation(
        execution,
        &world.program,
        &handles,
        &world.runtime,
        &observation,
    )
    .expect("the owner-issued basis remains neutrally observable")
}