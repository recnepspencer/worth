use std::collections::BTreeSet;

use super::world::supply_chain::{
    certified_supply_chain_world, commit_branch_batch, fork_supply_chain_branch_from_main,
    lower_supply_chain_production_delta, observe_supply_chain_observation, DeltaId,
    SupplyChainScale,
};
use worth_relational::facade::history::BranchId;

#[test]
fn admitted_supply_chain_observation_is_repeatable_after_branch_moves() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let (world, _) = certified_supply_chain_world(execution, SupplyChainScale::court());
    let branch_id = BranchId("storm".to_owned());
    fork_supply_chain_branch_from_main(&world.runtime, branch_id.clone());
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
        &BTreeSet::new(),
        DeltaId::StormRerouteAurora,
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
