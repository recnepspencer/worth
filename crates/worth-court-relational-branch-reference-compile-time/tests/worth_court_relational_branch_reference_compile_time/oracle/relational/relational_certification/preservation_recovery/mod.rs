#[allow(unused_imports)] use crate::relational_certification::reference_context::preservation_recovery::*;
pub(crate) use {worth_testkit_relational::relational_certification::ordinary_context::invariant_oracle_expectations::expected_supply_chain_branch};
pub(crate) use {worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::certified_baseline::assert_oracle_matches, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::certified_baseline::canonical_empty_supply_chain_runtime, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::certified_baseline::certified_supply_chain_world, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::runtime_driver::branch_operations::commit_branch_batch, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::comparison::compare, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::runtime_driver::lowering::lower_supply_chain_production_delta};
pub(crate) use {worth_relational::facade::history::BranchId};

/// Preservation evidence only: Phase 5 does not claim the Phase-10 recovery-
/// handle, fresh-process, or durable-owner lifecycle contract.
#[test]
fn checkpoint_preserves_shared_and_rewired_root_shape() {
    let host_request =
        worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let scale = worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scale::SupplyChainScale::court();
    let (world, baseline) = certified_supply_chain_world(execution, scale);
    assert_oracle_matches(execution, &world, &baseline);
    for branch in ["storm", "rewire"] {
        let (_, source) = world
            .runtime
            .observe_fork_source(&BranchId("main".to_owned()))
            .expect("main branch is a live fork source");
        world
            .runtime
            .fork_branch(execution, BranchId(branch.to_owned()), source)
            .expect("branch fork shares the immutable baseline root");
    }

    let delta = worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId::RewireAuroraPortCall;
    let batch = lower_supply_chain_production_delta(
        execution,
        &world.runtime,
        &world.program,
        &world.handles,
        &BranchId("rewire".to_owned()),
        &std::collections::BTreeSet::new(),
        delta,
    )
    .expect("the named Port3 topology delta observes production pre-state");
    commit_branch_batch(
        execution,
        &world.runtime,
        BranchId("rewire".to_owned()),
        batch,
    );

    world
        .runtime
        .durability_authority()
        .checkpoint(execution)
        .expect("checkpoint captures exact branch roots");
    let plan = world
        .runtime
        .durability()
        .recovery_plan(
            execution,
            worth_relational::facade::durability::RecoveryVerificationMode::NormalRecoveryVerification,
        )
        .expect("original request admits native recovery input");
    let mut recovered = canonical_empty_supply_chain_runtime(scale);
    recovered
        .durability_recovery()
        .recover(execution, plan)
        .expect("in-memory preservation recovery succeeds");

    let storm = recovered
        .branch_identity(&BranchId("storm".to_owned()))
        .expect("storm identity is reissued");
    let rewire = recovered
        .branch_identity(&BranchId("rewire".to_owned()))
        .expect("rewire identity is reissued");
    let sharing = recovered
        .observe_branch_sharing(&[
            recovered.main_branch_identity(),
            storm.clone(),
            rewire.clone(),
        ])
        .expect("recovered catalog artifacts retain root linkage");
    assert_eq!(sharing.unique_root_count(), 2);
    assert_eq!(sharing.unique_canonical_commit_artifacts(), 2);

    let rewire_observed =
        observe_recovered_branch(execution, &world, &recovered, &rewire, "rewire");
    compare(
        &expected_supply_chain_branch(&world.program, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::Rewire, Some(delta)),
        &rewire_observed,
    )
    .expect("the rewire branch recovers exactly the independent Port3 oracle delta");
}
