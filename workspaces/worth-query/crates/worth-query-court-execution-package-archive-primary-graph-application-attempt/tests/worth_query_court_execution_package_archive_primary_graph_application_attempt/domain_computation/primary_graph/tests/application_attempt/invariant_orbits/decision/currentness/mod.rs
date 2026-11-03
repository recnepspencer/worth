//! Retained authenticated handler candidates consume both actual source facts.

use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::decision::currentness::prepare;
use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::decision::currentness::publish;

use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::program;
use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::schema::*;
use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::world::World;
use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::decision::binding::*;
use crate::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::decision::identity;
use worth_query_execution::facade::domain_computation::HandlerResult;
use worth_execution::ExecutionRequest;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationCommitOutcome;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_entry;
use worth_execution::ExecutionAllocationPolicy as Allocation;
use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;
use worth_query_declaration::facade::application_operation::ApplicationMutationIdentities;
#[test]
fn retained_orbit_candidate_stales_after_identity_change_and_commits_after_unrelated_edit() {
    let _isolation = worth_query_testkit_execution_primary_graph_application_attempt::domain_computation::primary_graph::tests::fixture::reconstruction_execution::isolated_request_owner();
    for changed in [true, false] {
        let world = worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::decision::identity::install();
        let host = worth_query_testkit_execution_primary_graph_application_attempt_2::test_request_policy::serial_request();
        let execution = ExecutionRequest::serial(&host);
        let groups = [
            worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::decision::identity::read(&world, 0, &execution),
            worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::decision::identity::read(&world, 3, &execution),
        ];
        let loser = if groups[0].body.native().anchor() < groups[1].body.native().anchor() {
            3
        } else {
            0
        };
        let request = worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::schema_types::live_scope();
        let (candidate, key) = prepare(&world, &request, loser, "retained-orbit");
        let before = world.committed_version();
        if changed {
            for key in [233, 234] {
                let change = worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::program::swap_successors(&world, &request, 0, 3);
                assert!(matches!(
                    worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::commit(&world, &request, change, key),
                    WorthQueryApplicationCommitOutcome::Committed(_)
                ));
            }
            let current = worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::decision::identity::read(&world, loser, &execution);
            assert_ne!(
                groups[usize::from(loser == 3)].body.native().identity(),
                current.body.native().identity()
            );
        } else {
            let (other, idempotency) = prepare(&world, &request, 6, "unrelated-mark");
            assert!(matches!(
                publish(&world, &request, other, idempotency),
                WorthQueryApplicationCommitOutcome::Committed(_)
            ));
        }
        assert_ne!(
            world.committed_version(),
            before,
            "the ordinary edit really moved the head"
        );
        let outcome = publish(&world, &request, candidate, key);
        if changed {
            let WorthQueryApplicationCommitOutcome::Stale(stale) = outcome else {
                panic!("both complete facts must be stale: {outcome:?}")
            };
            assert_eq!(
                stale.stale_fact_count(),
                2,
                "repeated orbit/orbit_of retains one aggregate and one walk"
            );
        } else {
            assert!(matches!(
                outcome,
                WorthQueryApplicationCommitOutcome::Committed(_)
            ));
        }
    }
}
