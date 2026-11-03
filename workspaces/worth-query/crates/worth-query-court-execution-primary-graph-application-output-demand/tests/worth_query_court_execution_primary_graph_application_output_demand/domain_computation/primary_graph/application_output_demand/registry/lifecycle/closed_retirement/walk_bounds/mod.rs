//! Actual mutation walk sites stay local when another occurrence grows.

use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::application_output_demand::registry::lifecycle::closed_retirement::walk_bounds::WalkBoundsDemandRegistryStateForTest;

use crate::domain_computation::primary_graph::application_output_demand::registry::tests as registry_tests;
use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::application_output_demand::registry::tests::support::key;
use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::application_output_demand::registry::tests::support::occurrence;
use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::application_output_demand::registry::tests::support::record;
use worth_query_execution::domain_computation::primary_graph::application_output_demand::registry::DemandState;
use worth_query_execution::domain_computation::primary_graph::application_output_demand::registry::checkpoint::WorthQueryOutputCheckpoint;
use worth_query_execution::domain_computation::primary_graph::application_output_demand::registry::checkpoint::WorthQueryOutputProgress;
use worth_query_execution::domain_computation::primary_graph::application_output_demand::registry::DemandRegistryState;
use worth_query_execution::facade::domain_computation::WorthQueryOutputDemandDenial;
use worth_query_execution::facade::domain_computation::WorthQueryOutputDemandDenialKind;
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
