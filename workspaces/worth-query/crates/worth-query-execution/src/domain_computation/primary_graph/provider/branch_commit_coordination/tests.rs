use super::WorthQueryApplicationBranchCommitCoordinator as Coordinator;

use crate::domain_computation::primary_graph::tests::fixture::installed_capability_authorization_world;

#[test]
fn same_occurrence_shares_a_lane_until_retirement() {
    let world = installed_capability_authorization_world();
    let selected = world
        .application
        .with_host_advancement(|phase| world.selected_product(&phase.test_host_request()))
        .unwrap();
    let observation = selected.product().observation();
    let owner = Coordinator::new();
    let first = owner.lane_for(observation);
    let second = owner.lane_for(observation);
    assert!(std::sync::Arc::ptr_eq(&first, &second));
    assert_eq!(owner.entry_count(), 1);
    owner.retire(observation.lifecycle_incarnation());
    assert_eq!(owner.entry_count(), 0);
    drop((first, second));
}

#[test]
fn expired_weak_replacement_keeps_the_entry_claim_until_retirement() {
    let world = installed_capability_authorization_world();
    let selected = world
        .application
        .with_host_advancement(|phase| world.selected_product(&(phase).test_host_request()))
        .expect("the fixture read owns its installed host call");
    let observation = selected.product().observation();
    let owner = Coordinator::new();
    let first = owner.lane_for(observation);

    drop(first);

    let replacement = owner.lane_for(observation);
    assert_eq!(owner.entry_count(), 1);

    owner.retire(observation.lifecycle_incarnation());

    drop(replacement);
}
