#[allow(unused_imports)]
pub(crate) use {std::collections::BTreeSet, worth_relational::facade::history::BranchId, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKey, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKey, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKind, worth_relational::facade::inspection::RelationalAuthoritativeAllocationKind, worth_relational::facade::inspection::RelationalBranchSharingCostCounters, worth_relational::facade::inspection::RelationalMvccCostScope, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scale::SupplyChainScale, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::certified_baseline::assert_oracle_matches, crate::relational_certification::ordinary_context::root_branch_copy_on_write::assert_rewire_matches_oracle, crate::relational_certification::ordinary_context::root_branch_copy_on_write::branch_identity, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::certified_baseline::certified_supply_chain_world, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::runtime_driver::branch_operations::commit_branch_batch, crate::relational_certification::ordinary_context::root_branch_copy_on_write::expected_rewire_from_pure_oracle, crate::relational_certification::ordinary_context::root_branch_copy_on_write::fork_from_main, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::runtime_driver::lowering::lower_supply_chain_production_delta, crate::relational_certification::ordinary_context::root_branch_copy_on_write::region_set, crate::relational_certification::ordinary_context::root_branch_copy_on_write::sharing};

#[test]
fn forked_topology_write_preserves_ancestor_and_sibling_truth() {
    let host_request =
        worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let (mut world, baseline) = certified_supply_chain_world(execution, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scale::SupplyChainScale::court());
    assert_oracle_matches(execution, &world, &baseline);
    fork_from_main(execution, &world.runtime, "rewire");
    fork_from_main(execution, &world.runtime, "maintenance");

    let main = world.runtime.main_branch_identity();
    let rewire = branch_identity(&world.runtime, "rewire");
    let sibling = branch_identity(&world.runtime, "maintenance");
    let before_main = world.runtime.observe_branch(&main).unwrap().0;
    let before_sibling = world.runtime.observe_branch(&sibling).unwrap().0;
    let delta = worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId::RewireAuroraPortCall;
    let expected = expected_rewire_from_pure_oracle(&world, delta);
    let batch = lower_supply_chain_production_delta(
        execution,
        &world.runtime,
        &world.program,
        &world.handles,
        &BranchId("rewire".to_owned()),
        &std::collections::BTreeSet::new(),
        delta,
    )
    .expect("the actual rewire branch pre-state lowers to production intent");
    commit_branch_batch(
        execution,
        &world.runtime,
        BranchId("rewire".to_owned()),
        batch,
    );

    assert_rewire_matches_oracle(execution, &mut world, &rewire, expected);
    assert_eq!(world.runtime.observe_branch(&main).unwrap().0, before_main);
    assert_eq!(
        world.runtime.observe_branch(&sibling).unwrap().0,
        before_sibling
    );
}
