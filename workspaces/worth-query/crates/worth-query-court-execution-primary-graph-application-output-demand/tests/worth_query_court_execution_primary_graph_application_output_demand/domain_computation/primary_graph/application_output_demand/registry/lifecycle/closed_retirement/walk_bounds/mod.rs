//! Actual mutation walk sites stay local when another occurrence grows.

use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::application_output_demand::registry::lifecycle::closed_retirement::walk_bounds::WalkBoundsDemandRegistryStateForTest;

use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::DemandRegistryState;
#[test]
fn registry_row_retirement_walk_body_keeps_its_bound() {
    DemandRegistryState::assert_walk_site(0);
}
#[test]
fn registry_follow_newest_walk_body_keeps_its_bound() {
    DemandRegistryState::assert_walk_site(1);
}
#[test]
fn registry_newest_ready_walk_body_keeps_its_bound() {
    DemandRegistryState::assert_walk_site(2);
}
#[test]
fn registry_held_walk_body_keeps_its_bound() {
    DemandRegistryState::assert_walk_site(3);
}
#[test]
fn registry_prerequisite_release_walk_body_keeps_its_bound() {
    DemandRegistryState::assert_walk_site(4);
}
#[test]
fn registry_member_release_walk_body_keeps_its_bound() {
    DemandRegistryState::assert_walk_site(5);
}
#[test]
fn registry_revive_ready_walk_body_keeps_its_bound() {
    DemandRegistryState::assert_walk_site(6);
}
