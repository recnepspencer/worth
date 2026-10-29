#[allow(unused_imports)]
pub(crate) use {worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scale::SupplyChainScale, crate::relational_certification::ordinary_context::root_branch_traversal_isolation::assert_ancestor_edge_is_visible, crate::relational_certification::ordinary_context::root_branch_traversal_isolation::assert_child_retains_ancestor_and_rejects_main_edge, crate::relational_certification::ordinary_context::root_branch_traversal_isolation::assert_main_edge_is_visible, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::certified_baseline::assert_oracle_matches, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::certified_baseline::certified_supply_chain_world, crate::relational_certification::ordinary_context::root_branch_traversal_isolation::current_snapshot, crate::relational_certification::ordinary_context::root_branch_traversal_isolation::edge_movement, crate::relational_certification::ordinary_context::root_branch_traversal_isolation::fork_from_main, crate::relational_certification::ordinary_context::root_branch_traversal_isolation::publish_main_rewire};

#[test]
fn branch_traversal_enumerates_edges_from_the_selected_immutable_root() {
    let host_request =
        worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let (world, baseline) = certified_supply_chain_world(execution, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scale::SupplyChainScale::court());
    assert_oracle_matches(execution, &world, &baseline);
    let edge = edge_movement(&world.handles, &baseline);
    let child_snapshot = fork_from_main(execution, &world.runtime, "storm");
    assert_ancestor_edge_is_visible(execution, &world.runtime, &child_snapshot, &edge);
    publish_main_rewire(execution, &world.runtime, &edge);
    let main_snapshot = current_snapshot(&world.runtime, "main");
    assert_main_edge_is_visible(execution, &world.runtime, &main_snapshot, &edge);
    assert_child_retains_ancestor_and_rejects_main_edge(
        execution,
        &world.runtime,
        &child_snapshot,
        &edge,
    );
}
