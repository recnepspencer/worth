use super::world::supply_chain::certified_supply_chain_world;
use super::world::supply_chain::SupplyChainScale;
use worth_relational::facade::history::BranchId;

#[test]
fn supply_chain_fork_acquires_one_target_head_obligation() {
    let host_request =
        crate::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let (world, _) = certified_supply_chain_world(execution, SupplyChainScale::court());
    let source = BranchId("main".to_owned());
    let target = BranchId("storm".to_owned());
    let (_, basis) = world
        .runtime
        .observe_fork_source(&source)
        .expect("main has a retained source artifact");
    let fork = world
        .runtime
        .fork_branch(execution, target.clone(), basis)
        .expect("fork creates a retained target reference");

    assert_eq!(
        world
            .runtime
            .branch_retention_cost_counters(fork.target_identity())
            .unwrap()
            .head_installs,
        1
    );
    assert_eq!(
        world
            .runtime
            .branch_reference_state(&source)
            .expect("source remains registered")
            .lifecycle_posture(),
        worth_relational::facade::branch::RelationalBranchLifecyclePosture::Live,
    );
    assert_eq!(
        world
            .runtime
            .branch_reference_state(&target)
            .expect("target is registered")
            .lifecycle_posture(),
        worth_relational::facade::branch::RelationalBranchLifecyclePosture::Live,
    );
}
