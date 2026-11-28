#[allow(unused_imports)]
pub(crate) use {std::collections::BTreeSet, worth_relational::facade::history::BranchId, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::schema::EntityRecord, worth_relational::facade::branch::RelationalBranchBasisDenial, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scale::SupplyChainScale, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::certified_baseline::certified_supply_chain_world, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::runtime_driver::branch_operations::commit_branch_batch, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::runtime_driver::branch_operations::fork_supply_chain_branch_from_main, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::runtime_driver::lowering::lower_supply_chain_production_delta, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::observation::observe_observation as observe_supply_chain_observation};

#[test]
fn history_and_visibility_read_the_observation_selected_root() {
    let host_request =
        worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let (world, _) = certified_supply_chain_world(execution, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scale::SupplyChainScale::court());
    let branch_id = BranchId("storm".to_owned());
    fork_supply_chain_branch_from_main(execution, &world.runtime, branch_id.clone());
    let identity = world.runtime.branch_identity(&branch_id).unwrap();
    let (_, basis) = world.runtime.observe_branch(&identity).unwrap();
    let observation = basis.observation();
    let voyage_status = |observed: &_| {
        let worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::schema::EntityRecord::Voyage(voyage) = &observe_supply_chain_observation(
            execution,
            &world.program,
            &world.handles,
            &world.runtime,
            observed,
        )
        .unwrap()
        .entities[&world.handles.aurora_voyage().semantic] else {
            panic!("Aurora remains a voyage in the admitted root");
        };
        voyage.status
    };
    let status_before = voyage_status(&observation);
    let original_head = world
        .runtime
        .history()
        .branch_head_for_observation(&observation)
        .unwrap()
        .expect("baseline has one canonical head");

    let batch = lower_supply_chain_production_delta(
        execution,
        &world.runtime,
        &world.program,
        &world.handles,
        &branch_id,
        &std::collections::BTreeSet::new(),
        worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId::StormRerouteAurora,
    )
    .unwrap();
    commit_branch_batch(execution, &world.runtime, branch_id, batch);

    let (_, current_basis) = world.runtime.observe_branch(&identity).unwrap();
    let current_observation = current_basis.observation();
    let current_head = world
        .runtime
        .history()
        .branch_head_for_observation(&current_observation)
        .unwrap()
        .unwrap();
    assert_ne!(current_head.commit_id, original_head.commit_id);
    assert_eq!(
        world
            .runtime
            .history()
            .branch_head_for_observation(&observation)
            .unwrap()
            .unwrap()
            .commit_id,
        original_head.commit_id
    );
    assert_eq!(voyage_status(&observation), status_before);
    assert_ne!(voyage_status(&current_observation), status_before);
}
