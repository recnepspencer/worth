use std::collections::BTreeSet;

use super::super::invariant_oracle_expectations::expected_supply_chain_branch;
use super::super::world::supply_chain::{
    certified_supply_chain_world, compare, lower_supply_chain_production_delta,
    observe_supply_chain_observation, BranchLabel, DeltaId, ProductionSeededSupplyChainWorld,
    SupplyChainScale,
};
use worth_relational::facade::branch::{
    AdmittedRelationalBranchBasis, RelationalBranchIdentity, RelationalOwnerServicePorts,
};
use worth_relational::facade::history::BranchId;
use worth_relational::facade::mvcc::{
    RelationalBranchTransactionAdmissionDenial, RelationalPublicationOutcome,
    RelationalTransactionIntent,
};

#[test]
fn port_publication_moves_supply_chain_truth_and_stales_the_direct_basis() {
    let (world, _baseline) = certified_supply_chain_world(SupplyChainScale::court());
    let services = world.runtime.owner_component_services();
    let storm = fork_supply_chain_branch(&services, "storm");
    let predecessor = services
        .basis_port()
        .admit_branch_basis(&storm)
        .expect("the basis port admits the exact storm predecessor");
    let before = observe_basis(&world, &predecessor);
    compare(
        &expected_supply_chain_branch(&world.program, BranchLabel::Storm, None),
        &before,
    )
    .expect("the fork starts at the independent Supply Chain baseline");

    let batch = lower_supply_chain_production_delta(
        &world.runtime,
        &world.program,
        &world.handles,
        storm.branch_id(),
        &BTreeSet::new(),
        DeltaId::StormRerouteAurora,
    )
    .expect("the real storm branch admits its semantic reroute delta");
    let mut transaction = world
        .runtime
        .begin_branch_transaction(&predecessor, RelationalTransactionIntent::ordinary())
        .expect("the exact port-issued predecessor opens a transaction");
    transaction
        .push_batch(batch)
        .expect("the Supply Chain reroute remains inside its declared budget");
    let candidate = services
        .preparation_port()
        .prepare_branch_transaction(transaction)
        .expect("the preparation port builds the canonical reroute candidate");
    let performed = match services.publication_port().compare_and_publish(candidate) {
        RelationalPublicationOutcome::Performed(performed) => performed,
        outcome => panic!("the real reroute did not perform through the port: {outcome:?}"),
    };
    let committed = services
        .settlement_port()
        .settle_performed_publication(performed)
        .expect("the settlement port completes the performed reroute");

    assert!(matches!(
        world
            .runtime
            .begin_branch_transaction(&predecessor, RelationalTransactionIntent::ordinary()),
        Err(RelationalBranchTransactionAdmissionDenial::StaleBasis)
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
    let direct_after = observe_basis(&world, &direct_basis);
    let port_after = observe_basis(&world, &port_basis);
    assert_eq!(direct_after, port_after);
    assert_ne!(
        before, port_after,
        "a success-shaped no-op must fail this court"
    );
    compare(
        &expected_supply_chain_branch(
            &world.program,
            BranchLabel::Storm,
            Some(DeltaId::StormRerouteAurora),
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
    let (world, _baseline) = certified_supply_chain_world(SupplyChainScale::court());
    let services = world.runtime.owner_component_services();
    let maintenance = fork_supply_chain_branch(&services, "maintenance");
    let predecessor = services
        .basis_port()
        .admit_branch_basis(&maintenance)
        .expect("the basis port admits the maintenance predecessor");
    let before = observe_basis(&world, &predecessor);

    let batch = lower_supply_chain_production_delta(
        &world.runtime,
        &world.program,
        &world.handles,
        maintenance.branch_id(),
        &BTreeSet::new(),
        DeltaId::MaintainAtlasBerth,
    )
    .expect("the real maintenance branch admits its semantic berth delta");
    let mut transaction = world
        .runtime
        .begin_branch_transaction(&predecessor, RelationalTransactionIntent::ordinary())
        .expect("the exact port-issued predecessor opens the direct route");
    transaction
        .push_batch(batch)
        .expect("the Supply Chain maintenance delta remains in budget");
    let committed = transaction
        .commit(&world.runtime)
        .expect("the compatibility commit performs canonically");

    assert!(matches!(
        world
            .runtime
            .begin_branch_transaction(&predecessor, RelationalTransactionIntent::ordinary()),
        Err(RelationalBranchTransactionAdmissionDenial::StaleBasis)
    ));
    let (_, port_basis) = services
        .basis_port()
        .observe_branch(&maintenance)
        .expect("the basis port immediately observes the direct movement");
    let after = observe_basis(&world, &port_basis);
    assert_ne!(
        before, after,
        "a success-shaped direct no-op must fail this court"
    );
    compare(
        &expected_supply_chain_branch(
            &world.program,
            BranchLabel::Maintenance,
            Some(DeltaId::MaintainAtlasBerth),
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

fn fork_supply_chain_branch(
    services: &RelationalOwnerServicePorts,
    branch: &str,
) -> RelationalBranchIdentity {
    let (_, source) = services
        .fork_port()
        .observe_fork_source(&BranchId("main".to_owned()))
        .expect("the non-empty Supply Chain main branch is a fork source");
    services
        .fork_port()
        .fork_branch(BranchId(branch.to_owned()), source)
        .expect("the fork port installs the exact Supply Chain branch")
        .target_identity()
        .clone()
}

fn observe_basis(
    world: &ProductionSeededSupplyChainWorld,
    basis: &AdmittedRelationalBranchBasis,
) -> super::super::world::supply_chain::ObservedSupplyChainState {
    let observation = basis.observation();
    let handles = world.handles.for_observation(&observation);
    observe_supply_chain_observation(&world.program, &handles, &world.runtime, &observation)
        .expect("the owner-issued basis remains neutrally observable")
}
