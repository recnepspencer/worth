#[allow(unused_imports)]
pub(crate) use {std::collections::BTreeSet, worth_relational::facade::history::BranchId, worth_relational::facade::branch::RelationalBranchDeletionOutcome, worth_relational::facade::mvcc::RelationalTransactionIntent};

#[test]
fn discarded_candidate_leaves_truth_unchanged_and_releases_its_obligation() {
    let host_request =
        worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    use {worth_testkit_relational::relational_certification::ordinary_context::invariant_oracle_expectations::expected_supply_chain_branch};
    use {worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::certified_baseline::certified_supply_chain_world, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::comparison::compare, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::runtime_driver::lowering::lower_supply_chain_production_delta, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::observation::observe_observation as observe_supply_chain_observation, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scale::SupplyChainScale};

    let (world, _baseline) = certified_supply_chain_world(execution, SupplyChainScale::court());
    let services = world.runtime.owner_component_services();
    let (_, source) = services
        .fork_port()
        .observe_fork_source(&BranchId("main".to_owned()))
        .expect("the non-empty Supply Chain main branch is a fork source");
    let identity = services
        .fork_port()
        .fork_branch(execution, BranchId("storm".to_owned()), source)
        .expect("the fork port installs the exact storm branch")
        .target_identity()
        .clone();
    let (descriptor, basis) = services
        .basis_port()
        .observe_branch(&identity)
        .expect("the basis port observes the storm predecessor");
    let batch = lower_supply_chain_production_delta(
        execution,
        &world.runtime,
        &world.program,
        &world.handles,
        identity.branch_id(),
        &std::collections::BTreeSet::new(),
        DeltaId::StormRerouteAurora,
    )
    .expect("the real storm branch admits its semantic reroute delta");
    let mut transaction = world
        .runtime
        .begin_branch_transaction(&basis, worth_relational::facade::mvcc::RelationalTransactionIntent::ordinary())
        .expect("the exact storm predecessor opens a transaction");
    transaction
        .push_batch(
            batch,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("the semantic reroute remains inside its declared budget");
    let candidate = services
        .preparation_port()
        .prepare_branch_transaction(execution, transaction)
        .expect("the preparation port installs the canonical candidate");
    let discarded = services
        .preparation_port()
        .discard_prepared_candidate(candidate)
        .expect("the preparation port consumes the candidate without movement");
    assert_eq!(discarded.branch(), identity.branch_id());
    assert_eq!(
        discarded.released_record_reservation_count(),
        0,
        "the update-only Supply Chain delta must not invent record reservations"
    );

    let (after_descriptor, after_basis) = services
        .basis_port()
        .observe_branch(&identity)
        .expect("discard leaves the predecessor observable");
    assert_eq!(after_descriptor, descriptor);
    let observation = after_basis.observation();
    let handles = world.handles.for_observation(&observation);
    let observed = observe_supply_chain_observation(
        execution,
        &world.program,
        &handles,
        &world.runtime,
        &observation,
    )
    .expect("discarded work leaves neutral Supply Chain truth observable");
    compare(
        &expected_supply_chain_branch(&world.program, BranchLabel::Storm, None),
        &observed,
    )
    .expect("discarded work leaves the independent baseline unchanged");
    assert!(matches!(
        services.lifecycle_port().delete_branch(execution, &identity),
        Ok(worth_relational::facade::branch::RelationalBranchDeletionOutcome::Deleted(ref deleted))
            if deleted.identity() == &identity
    ));
}
