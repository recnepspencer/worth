//! Which owner calls the sealed read set keeps for each fact, for an owner
//! reading through the reader of a real admitted operation.

use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::application_contribution::partitioned_computation::attribution_tests::StatusReadBy;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::application_contribution::partitioned_computation::attribution_tests::routed;

use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::read_set::computation_routing::ComputationFactReaders;
#[test]
fn a_fact_two_gathers_read_is_kept_for_both_partitions() {
    let (readers, partitions) = routed(StatusReadBy::ItemKeys, 1, false);

    assert_eq!(partitions.len(), 2, "odd and even items are two partitions");
    // The facts are in key order: the label, then the status.
    assert_eq!(
        readers.expect("one computation ran"),
        [
            ComputationFactReaders::read_by_for_test(false, [], partitions),
            ComputationFactReaders::read_by_for_test(false, [1, 2, 3], []),
        ]
    );
}
#[test]
fn a_fact_the_membership_read_is_a_membership_fact() {
    let (readers, partitions) = routed(StatusReadBy::Membership, 1, false);

    assert_eq!(
        readers.expect("one computation ran"),
        [
            ComputationFactReaders::read_by_for_test(false, [], partitions),
            ComputationFactReaders::read_by_for_test(true, [], []),
        ]
    );
}
#[test]
fn a_fact_the_handler_also_read_stays_the_item_keys() {
    // The handler reads its own facts again on every attempt, so it is not a
    // reader the computation keeps: the status is still the item keys' fact.
    let (readers, partitions) = routed(StatusReadBy::ItemKeys, 1, true);

    assert_eq!(
        readers.expect("one computation ran"),
        [
            ComputationFactReaders::read_by_for_test(false, [], partitions),
            ComputationFactReaders::read_by_for_test(false, [1, 2, 3], []),
        ]
    );
}
#[test]
fn an_attempt_that_ran_two_computations_keeps_no_computation_facts() {
    // Partition identities name the partitions of one computation only.
    assert!(routed(StatusReadBy::ItemKeys, 2, false).0.is_none());
}
