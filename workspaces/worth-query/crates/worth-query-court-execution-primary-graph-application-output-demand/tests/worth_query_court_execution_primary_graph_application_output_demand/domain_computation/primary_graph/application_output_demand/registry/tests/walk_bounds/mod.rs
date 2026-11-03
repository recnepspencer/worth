//! Canonical occurrence searches keep their producer/source boundary.

use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::application_output_demand::registry::tests::walk_bounds::check;

use worth_query_execution::domain_computation::primary_graph::application_output_demand::registry::refreshed_rejoin::awaited_by_stale_owner;
use worth_query_execution::domain_computation::primary_graph::application_output_demand::registry::refreshed_rejoin::newest_of_occurrence;
use worth_query_execution::domain_computation::primary_graph::application_output_demand::registry::refreshed_rejoin::occurrence_rows;
use crate::domain_computation::primary_graph::application_output_demand::registry::tests::*;
use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::recoverable_commit_support;
#[test]
fn registry_occurrence_rows_body_keeps_its_bound() {
    check(0);
}
#[test]
fn registry_newest_fold_body_keeps_its_bound() {
    check(1);
}
#[test]
fn registry_stale_owner_body_keeps_its_bound() {
    check(2);
}
use worth_query_execution::facade::domain_computation::WorthQueryApplicationCommitReceipt;
