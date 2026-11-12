pub(crate) use {worth_testkit_relational::relational_certification::ordinary_context::invariant_oracle_expectations::expected_supply_chain_branch};
pub(crate) use {worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::certified_baseline::assert_oracle_matches, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::certified_baseline::certified_supply_chain_world, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::runtime_driver::branch_operations::commit_supply_chain_delta, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::comparison::compare, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::runtime_driver::lowering::lower_supply_chain_production_delta, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::observation::observe_snapshot as observe_supply_chain_snapshot, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::runtime_driver::branch_operations::snapshot_for_supply_chain_identity, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKey, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKey, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKind, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scale::SupplyChainScale, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::handles::SupplyChainSemanticHandles};
pub(crate) use {worth_relational::facade::history::BranchId};
pub(crate) use {worth_relational::facade::runtime::RelationalRuntime};

pub(crate) fn fork_from_main(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    branch: &str,
) {
    let (_, source) = runtime
        .observe_fork_source(&BranchId("main".to_owned()))
        .unwrap();
    runtime
        .fork_branch(execution, BranchId(branch.to_owned()), source)
        .unwrap();
}

pub(crate) fn prove_named_delta_cow(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    branch: &str,
    label: BranchLabel,
    delta: DeltaId,
) {
    let (world, baseline) = certified_supply_chain_world(execution, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scale::SupplyChainScale::court());
    assert_oracle_matches(execution, &world, &baseline);
    fork_from_main(execution, &world.runtime, branch);
    fork_from_main(execution, &world.runtime, "cost-sibling");
    let selected = world
        .runtime
        .branch_identity(&BranchId(branch.to_owned()))
        .unwrap();
    let main = world.runtime.main_branch_identity();
    let sibling = world
        .runtime
        .branch_identity(&BranchId("cost-sibling".to_owned()))
        .unwrap();
    let before_main = world.runtime.observe_branch(&main).unwrap().0;
    let before_sibling = world.runtime.observe_branch(&sibling).unwrap().0;
    let batch = lower_supply_chain_production_delta(
        execution,
        &world.runtime,
        &world.program,
        &world.handles,
        &BranchId(branch.to_owned()),
        &std::collections::BTreeSet::new(),
        delta,
    )
    .expect("the actual branch pre-state admits its named delta");
    commit_supply_chain_delta(
        execution,
        &world.runtime,
        &world.program,
        BranchId(branch.to_owned()),
        delta,
        batch,
    );

    assert_eq!(world.runtime.observe_branch(&main).unwrap().0, before_main);
    assert_eq!(
        world.runtime.observe_branch(&sibling).unwrap().0,
        before_sibling
    );
    let snapshot = snapshot_for_supply_chain_identity(&world.runtime, &selected);
    let observed = observe_supply_chain_snapshot(
        execution,
        &world.program,
        &world.handles.for_snapshot(snapshot.clone()),
        &world.runtime,
        &snapshot,
    )
    .unwrap();
    compare(
        &expected_supply_chain_branch(&world.program, label, Some(delta)),
        &observed,
    )
    .unwrap();
    world
        .runtime
        .snapshots()
        .release_snapshot(&snapshot)
        .expect("the exact semantic proof releases its selected snapshot");
}
