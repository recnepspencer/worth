//! Genuine merge/split history isolates identity truth from membership and lineage.

use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::decision::identity::install;
use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::decision::identity::read;

use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::probe;
use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::program;
use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::schema::*;
use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::world;
use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::world::World;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::aggregate_observation::AggregateObservation;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::snapshot_lease::WorthQueryApplicationSnapshotLease;
use worth_execution::ExecutionRequest;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationCommitOutcome;
use std::sync::Arc;
use worth_execution::ExecutionAllocationPolicy as Allocation;
#[test]
fn aggregate_identity_changes_are_distinct_from_membership_and_incidental_history() {
    let _isolation = worth_query_testkit_execution_primary_graph_application_attempt::domain_computation::primary_graph::tests::fixture::reconstruction_execution::isolated_request_owner();
    let world = install();
    let host = worth_query_testkit_execution_primary_graph_application_attempt_2::test_request_policy::serial_request();
    let execution = ExecutionRequest::serial(&host);
    let originals = [read(&world, 0, &execution), read(&world, 3, &execution)];
    let winner =
        usize::from(originals[0].body.native().anchor() > originals[1].body.native().anchor());
    let request = worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::schema_types::live_scope();
    for key in [231, 232] {
        let change = worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::program::swap_successors(&world, &request, 0, 3);
        assert!(matches!(
            worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::commit(&world, &request, change, key),
            WorthQueryApplicationCommitOutcome::Committed(_)
        ));
    }
    for (index, key) in [0, 3].into_iter().enumerate() {
        let current = read(&world, key, &execution);
        let old = &originals[index].body;
        assert_eq!(
            old.native().members().collect::<Vec<_>>(),
            current.body.native().members().collect::<Vec<_>>()
        );
        assert_eq!(
            old.native().definition(),
            current.body.native().definition()
        );
        assert!(old.native().lineage().next().is_none());
        assert!(
            current.body.native().lineage().next().is_some(),
            "real commits changed stored history"
        );
        let same = index == winner;
        assert_eq!(
            old.native().identity() == current.body.native().identity(),
            same
        );
        assert_eq!(
            old.same_truth_with_poll(&current.body, || Ok::<_, ()>(()))
                .unwrap(),
            same
        );
        assert_eq!(old == &current.body, same);
        assert_eq!(
            current.still_equal(old, &execution),
            same,
            "only aggregate truth is compared; no walk can mask missing ID comparison"
        );
    }
}
