use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::provider::branch_commit_coordination::WorthQueryApplicationBranchCommitCoordinator as Coordinator;

use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::capability_world_installation::installed_capability_authorization_world;

#[test]
fn same_occurrence_shares_a_lane_until_retirement() {
    let world = installed_capability_authorization_world();
    let selected = world
        .application
        .with_host_advancement(|phase| world.selected_product(&phase.test_host_request_for_test()))
        .unwrap();
    let observation = selected.product().observation_for_test();
    let owner = Coordinator::new_for_test();
    let first = owner.lane_for_for_test(observation);
    let second = owner.lane_for_for_test(observation);
    assert!(std::sync::Arc::ptr_eq(&first, &second));
    assert_eq!(owner.entry_count_for_test(), 1);
    owner.retire_for_test(observation.lifecycle_incarnation());
    assert_eq!(owner.entry_count_for_test(), 0);
    drop((first, second));
}

#[test]
fn expired_weak_replacement_keeps_the_entry_claim_until_retirement() {
    let world = installed_capability_authorization_world();
    let selected = world
        .application
        .with_host_advancement(|phase| {
            world.selected_product(&(phase).test_host_request_for_test())
        })
        .expect("the fixture read owns its installed host call");
    let observation = selected.product().observation_for_test();
    let owner = Coordinator::new_for_test();
    let first = owner.lane_for_for_test(observation);

    drop(first);

    let replacement = owner.lane_for_for_test(observation);
    assert_eq!(owner.entry_count_for_test(), 1);

    owner.retire_for_test(observation.lifecycle_incarnation());

    drop(replacement);
}
