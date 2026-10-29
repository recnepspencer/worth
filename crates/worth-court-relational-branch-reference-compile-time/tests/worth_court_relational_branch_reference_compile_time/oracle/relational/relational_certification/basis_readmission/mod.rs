#[allow(unused_imports)]
pub(crate) use {std::collections::BTreeSet, worth_relational::facade::history::BranchId, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId, worth_relational::facade::branch::RelationalBranchBasisDenial, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scale::SupplyChainScale, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::certified_baseline::certified_supply_chain_world, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::runtime_driver::branch_operations::commit_branch_batch, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::runtime_driver::branch_operations::fork_supply_chain_branch_from_main, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::runtime_driver::lowering::lower_supply_chain_production_delta};

#[test]
fn transported_descriptor_requires_owner_readmission() {
    let host_request =
        worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let (world, _) = certified_supply_chain_world(execution, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scale::SupplyChainScale::court());
    let encoded =
        rmp_serde::to_vec_named(world.basis.descriptor()).expect("descriptor serializes safely");
    let restored = rmp_serde::from_slice(&encoded).expect("descriptor restores descriptively");

    let readmitted = world
        .runtime
        .readmit_branch_basis(&restored)
        .expect("the original owner recognizes the retained exact basis");
    assert_eq!(readmitted.descriptor(), world.basis.descriptor());

    let (foreign, _) = certified_supply_chain_world(execution, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scale::SupplyChainScale::court());
    assert!(matches!(
        foreign.runtime.readmit_branch_basis(&restored),
        Err(worth_relational::facade::branch::RelationalBranchBasisDenial::ForeignRuntime { .. })
    ));
}

#[test]
fn unretained_descriptor_cannot_follow_a_moved_reference() {
    let host_request =
        worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let (world, _) = certified_supply_chain_world(execution, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scale::SupplyChainScale::court());
    let runtime = world.runtime;
    let program = world.program;
    let handles = world.handles;
    let branch_id = BranchId("storm".to_owned());
    fork_supply_chain_branch_from_main(execution, &runtime, branch_id.clone());
    let identity = runtime.branch_identity(&branch_id).unwrap();
    let (_, basis) = runtime.observe_branch(&identity).unwrap();
    let descriptor = basis.descriptor().clone();
    drop(basis);

    let batch = lower_supply_chain_production_delta(
        execution,
        &runtime,
        &program,
        &handles,
        &branch_id,
        &std::collections::BTreeSet::new(),
        worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId::StormRerouteAurora,
    )
    .unwrap();
    commit_branch_batch(execution, &runtime, branch_id, batch);

    assert!(matches!(
        runtime.readmit_branch_basis(&descriptor),
        Err(worth_relational::facade::branch::RelationalBranchBasisDenial::UnavailableRetainedTarget)
    ));
}
