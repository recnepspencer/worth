#[allow(unused_imports)] use crate::relational_certification::reference_context::owner_service_completion::supply_chain_cutover::*;
pub(crate) use {std::collections::BTreeSet};

pub(crate) use {worth_testkit_relational::relational_certification::ordinary_context::invariant_oracle_expectations::expected_supply_chain_branch};
pub(crate) use {worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::certified_baseline::certified_supply_chain_world, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::comparison::compare, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::runtime_driver::lowering::lower_supply_chain_production_delta, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::observation::observe_observation as observe_supply_chain_observation, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::production_world::ProductionSeededSupplyChainWorld, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scale::SupplyChainScale};
pub(crate) use {worth_relational::facade::branch::AdmittedRelationalBranchBasis, worth_relational::facade::branch::RelationalBranchIdentity, worth_relational::facade::branch::RelationalOwnerServicePorts};
pub(crate) use {worth_relational::facade::history::BranchId};
pub(crate) use {worth_relational::facade::mvcc::RelationalBranchTransactionAdmissionDenial, worth_relational::facade::mvcc::RelationalPublicationOutcome, worth_relational::facade::mvcc::RelationalTransactionIntent};

#[test]
fn port_publication_moves_supply_chain_truth_and_stales_the_direct_basis() {
    let host_request =
        worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let (world, _baseline) = certified_supply_chain_world(execution, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scale::SupplyChainScale::court());
    let services = world.runtime.owner_component_services();
    let storm = fork_supply_chain_branch(execution, &services, "storm");
    let predecessor = services
        .basis_port()
        .admit_branch_basis(&storm)
        .expect("the basis port admits the exact storm predecessor");
    let before = observe_basis(execution, &world, &predecessor);
    compare(
        &expected_supply_chain_branch(&world.program, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::Storm, None),
        &before,
    )
    .expect("the fork starts at the independent Supply Chain baseline");

    let batch = lower_supply_chain_production_delta(
        execution,
        &world.runtime,
        &world.program,
        &world.handles,
        storm.branch_id(),
        &std::collections::BTreeSet::new(),
        worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId::StormRerouteAurora,
    )
    .expect("the real storm branch admits its semantic reroute delta");
    let mut transaction = world
        .runtime
        .begin_branch_transaction(&predecessor, worth_relational::facade::mvcc::RelationalTransactionIntent::ordinary())
        .expect("the exact port-issued predecessor opens a transaction");
    transaction
        .push_batch(
            batch,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("the Supply Chain reroute remains inside its declared budget");
    let candidate = services
        .preparation_port()
        .prepare_branch_transaction(execution, transaction)
        .expect("the preparation port builds the canonical reroute candidate");
    let performed = match services
        .publication_port()
        .compare_and_publish(execution, candidate)
    {
        worth_relational::facade::mvcc::RelationalPublicationOutcome::Performed(performed) => performed,
        outcome => panic!("the real reroute did not perform through the port: {outcome:?}"),
    };
    let committed = services
        .settlement_port()
        .settle_performed_publication(performed)
        .expect("the settlement port completes the performed reroute");

    assert!(matches!(
        world
            .runtime
            .begin_branch_transaction(&predecessor, worth_relational::facade::mvcc::RelationalTransactionIntent::ordinary()),
        Err(worth_relational::facade::mvcc::RelationalBranchTransactionAdmissionDenial::StaleBasis)
    ));
    let (_, direct_basis) = world
        .runtime
        .observe_branch(&storm)
        .expect("the direct route observes the port movement");
    let (_, port_basis) = services
        .basis_port()
        .observe_branch(&storm)
        .expect("the port observes the same canonical movement");
    assert_eq!(direct_basis.descriptor(), port_basis.descriptor());
    let direct_after = observe_basis(execution, &world, &direct_basis);
    let port_after = observe_basis(execution, &world, &port_basis);
    assert_eq!(direct_after, port_after);
    assert_ne!(
        before, port_after,
        "a success-shaped no-op must fail this court"
    );
    compare(
        &expected_supply_chain_branch(
            &world.program,
            worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::Storm,
            Some(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId::StormRerouteAurora),
        ),
        &port_after,
    )
    .expect("the moved port observation matches the independent storm oracle");
    world
        .runtime
        .snapshots()
        .release_snapshot(&committed.snapshot)
        .expect("the semantic port proof releases its settlement snapshot");
}

#[test]
fn direct_commit_moves_supply_chain_truth_visible_through_the_basis_port() {
    let host_request =
        worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let (world, _baseline) = certified_supply_chain_world(execution, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scale::SupplyChainScale::court());
    let services = world.runtime.owner_component_services();
    let maintenance = fork_supply_chain_branch(execution, &services, "maintenance");
    let predecessor = services
        .basis_port()
        .admit_branch_basis(&maintenance)
        .expect("the basis port admits the maintenance predecessor");
    let before = observe_basis(execution, &world, &predecessor);

    let batch = lower_supply_chain_production_delta(
        execution,
        &world.runtime,
        &world.program,
        &world.handles,
        maintenance.branch_id(),
        &std::collections::BTreeSet::new(),
        worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId::MaintainAtlasBerth,
    )
    .expect("the real maintenance branch admits its semantic berth delta");
    let mut transaction = world
        .runtime
        .begin_branch_transaction(&predecessor, worth_relational::facade::mvcc::RelationalTransactionIntent::ordinary())
        .expect("the exact port-issued predecessor opens the direct route");
    transaction
        .push_batch(
            batch,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("the Supply Chain maintenance delta remains in budget");
    let committed = transaction
        .commit(execution, &world.runtime)
        .expect("the compatibility commit performs canonically");

    assert!(matches!(
        world
            .runtime
            .begin_branch_transaction(&predecessor, worth_relational::facade::mvcc::RelationalTransactionIntent::ordinary()),
        Err(worth_relational::facade::mvcc::RelationalBranchTransactionAdmissionDenial::StaleBasis)
    ));
    let (_, port_basis) = services
        .basis_port()
        .observe_branch(&maintenance)
        .expect("the basis port immediately observes the direct movement");
    let after = observe_basis(execution, &world, &port_basis);
    assert_ne!(
        before, after,
        "a success-shaped direct no-op must fail this court"
    );
    compare(
        &expected_supply_chain_branch(
            &world.program,
            worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::Maintenance,
            Some(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId::MaintainAtlasBerth),
        ),
        &after,
    )
    .expect("the port observation matches the independent maintenance oracle");
    world
        .runtime
        .snapshots()
        .release_snapshot(&committed.snapshot)
        .expect("the semantic direct proof releases its settlement snapshot");
}
