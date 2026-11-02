//! Only genuine immutable custody survives branch retirement; mutable admissions stay strict.
use super::*;
use worth_relational::facade::branch::{
    RelationalBranchBasisDenial as Denial, RelationalBranchDeletionOutcome,
    RelationalBranchObservation, RelationalForkDenial,
};
use worth_relational::facade::runtime::RelationalRuntime;
use worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::observation::{observe_entities, observe_relations};

#[test]
fn retired_basis_can_be_retained_again_without_readmitting_a_descriptor_currentness_or_fork() {
    let request = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&request);
    let (mut world, _) = certified_supply_chain_world(execution, SupplyChainScale::court());
    let (foreign, _) = certified_supply_chain_world(execution, SupplyChainScale::court());
    let branch = BranchId("retained-observation-source".into());
    fork_supply_chain_branch_from_main(execution, &world.runtime, branch.clone());
    let identity = world.runtime.branch_identity(&branch).unwrap();
    let (descriptor, basis) = world.runtime.observe_branch(&identity).unwrap();
    // Read immutable entity/relation truth directly. The world-level observer
    // also classifies authored branch labels and today's parent metadata,
    // neither of which is part of this retained-read contract.
    let truth = |runtime: &RelationalRuntime, observation: &RelationalBranchObservation| {
        let view = runtime
            .read_truth()
            .read_observation(execution, observation)
            .unwrap()
            .unwrap();
        (
            observe_entities(&world.program, &world.handles, &view).unwrap(),
            observe_relations(&world.handles, &view).unwrap(),
        )
    };
    let original = truth(&world.runtime, &basis.observation());
    let live = world.runtime.retain_component_basis(&basis).unwrap();
    world.runtime.release_component_basis(live).unwrap();
    assert!(matches!(
        world.runtime.delete_branch(execution, &identity).unwrap(),
        RelationalBranchDeletionOutcome::Deleted(_)
    ));
    assert!(
        matches!(world.runtime.observe_branch(&identity), Err(Denial::UnknownBranch(ref absent)) if absent == &branch)
    );
    assert!(
        matches!(world.runtime.readmit_branch_basis(&descriptor), Err(Denial::UnknownBranch(ref absent)) if absent == &branch)
    );
    assert!(
        matches!(world.runtime.owner_component_services().basis_port().compare_current_exact(&basis), Err(Denial::UnknownBranch(ref absent)) if absent == &branch)
    );
    assert!(matches!(
        world.runtime.fork_port().admit_retained_fork_source(&basis),
        Err(RelationalForkDenial::SourceBranchMissing)
    ));
    assert!(matches!(
        foreign.runtime.retain_component_basis(&basis),
        Err(Denial::ForeignRuntime { .. })
    ));

    let lease = world.runtime.retain_component_basis(&basis).unwrap();
    let restored = world
        .runtime
        .readmit_retained_branch_basis(&descriptor, &lease)
        .unwrap();
    assert_eq!(restored, basis);
    let snapshot = world
        .runtime
        .retain_observation_snapshot(&restored)
        .unwrap();
    assert_eq!(truth(&world.runtime, snapshot.observation()), original);
    world.runtime.release_component_basis(lease).unwrap();
    // The independently retained snapshot still owns its exact immutable view.
    assert_eq!(truth(&world.runtime, snapshot.observation()), original);
    snapshot.release();
}
