#[allow(unused_imports)]
pub(crate) use {std::collections::BTreeSet, worth_relational::facade::history::BranchId, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scale::SupplyChainScale, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::certified_baseline::certified_supply_chain_world, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::runtime_driver::branch_operations::commit_branch_batch, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::runtime_driver::branch_operations::fork_supply_chain_branch_from_main, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::runtime_driver::lowering::lower_supply_chain_production_delta, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::observation::observe_observation as observe_supply_chain_observation};

#[test]
fn admitted_supply_chain_observation_is_repeatable_after_branch_moves() {
    let host_request =
        worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let (world, _) = certified_supply_chain_world(execution, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scale::SupplyChainScale::court());
    let branch_id = BranchId("storm".to_owned());
    fork_supply_chain_branch_from_main(execution, &world.runtime, branch_id.clone());
    let identity = world.runtime.branch_identity(&branch_id).unwrap();
    let (_, basis) = world.runtime.observe_branch(&identity).unwrap();
    let observation = basis.observation();
    let selected_root = observation.selected_root_identity();
    let selected_version = observation.version_id();
    let before = observe_supply_chain_observation(
        execution,
        &world.program,
        &world.handles,
        &world.runtime,
        &observation,
    )
    .expect("the admitted storm observation is readable");

    let batch = lower_supply_chain_production_delta(
        execution,
        &world.runtime,
        &world.program,
        &world.handles,
        &branch_id,
        &std::collections::BTreeSet::new(),
        worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId::StormRerouteAurora,
    )
    .expect("the real Supply Chain delta lowers from the admitted storm basis");
    commit_branch_batch(execution, &world.runtime, branch_id.clone(), batch);

    let repeated = observe_supply_chain_observation(
        execution,
        &world.program,
        &world.handles,
        &world.runtime,
        &observation,
    )
    .expect("the original admitted observation remains readable");
    assert_eq!(repeated, before, "the old observation remains exact");

    let (_, current) = world.runtime.observe_branch(&identity).unwrap();
    assert_ne!(
        current.observation().selected_root_identity(),
        selected_root
    );
    assert_ne!(current.observation().version_id(), selected_version);
}
