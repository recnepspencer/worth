use std::collections::BTreeSet;

use super::world::supply_chain::{
    certified_supply_chain_world, commit_branch_batch, fork_supply_chain_branch_from_main,
    lower_supply_chain_production_delta, DeltaId, SupplyChainScale,
};
use worth_relational::facade::branch::RelationalBranchBasisDenial;
use worth_relational::facade::history::BranchId;

#[test]
fn transported_descriptor_requires_owner_readmission() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let (world, _) = certified_supply_chain_world(execution, SupplyChainScale::court());
    let encoded =
        rmp_serde::to_vec_named(world.basis.descriptor()).expect("descriptor serializes safely");
    let restored = rmp_serde::from_slice(&encoded).expect("descriptor restores descriptively");

    let readmitted = world
        .runtime
        .readmit_branch_basis(&restored)
        .expect("the original owner recognizes the retained exact basis");
    assert_eq!(readmitted.descriptor(), world.basis.descriptor());

    let (foreign, _) = certified_supply_chain_world(execution, SupplyChainScale::court());
    assert!(matches!(
        foreign.runtime.readmit_branch_basis(&restored),
        Err(RelationalBranchBasisDenial::ForeignRuntime { .. })
    ));
}

#[test]
fn unretained_descriptor_cannot_follow_a_moved_reference() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let (world, _) = certified_supply_chain_world(execution, SupplyChainScale::court());
    let runtime = world.runtime;
    let program = world.program;
    let handles = world.handles;
    let branch_id = BranchId("storm".to_owned());
    fork_supply_chain_branch_from_main(&runtime, branch_id.clone());
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
        &BTreeSet::new(),
        DeltaId::StormRerouteAurora,
    )
    .unwrap();
    commit_branch_batch(execution, &runtime, branch_id, batch);

    assert!(matches!(
        runtime.readmit_branch_basis(&descriptor),
        Err(RelationalBranchBasisDenial::UnavailableRetainedTarget)
    ));
}
