//! Whole-orbit disclosure is indivisible and uses a genuinely issued capability.

use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::schema::OrbitSchema;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationOneShotDenialKind;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationQueryAdmissionDenialKind;
use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::declared_queries::declarations::whole;
use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::declared_queries::world;

use crate::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::declared_queries::*;
#[test]
fn governed_orbit_allows_its_exact_slot_and_refuses_omission_or_missing_rule() {
    let _isolation =
        worth_query_testkit_execution_primary_graph_application_attempt::domain_computation::primary_graph::tests::fixture::reconstruction_execution::isolated_request_owner();
    let world = world();
    let allowed = worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::declared_queries::read::run(&world, Allowed::reference(), 0);
    assert_eq!(
        allowed
            .orbit(whole::<OrbitSchema<true>, Allowed>())
            .unwrap()
            .len(),
        3
    );
    let host = worth_query_testkit_execution_primary_graph_application_attempt_2::test_request_policy::serial_request();
    let execution = worth_execution::ExecutionRequest::serial(&host);
    let request = worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::schema_types::live_scope();
    let error = worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::declared_queries::read::execute(&world, &execution, &request, Omitted::reference(), 0)
        .err()
        .expect("the query must refuse");
    assert!(
        matches!(error, worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::declared_queries::read::Stop::Read(ref error) if error.kind() == WorthQueryApplicationOneShotDenialKind::ProjectionUnavailable && error.subject() == "orbit"),
        "{error:?}"
    );
    let error = worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::declared_queries::read::execute(&world, &execution, &request, Missing::reference(), 0)
        .err()
        .expect("the query must refuse");
    assert!(
        matches!(error, worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::declared_queries::read::Stop::Admission(ref error) if error.kind() == WorthQueryApplicationQueryAdmissionDenialKind::DisclosureContractInvalid),
        "{error:?}"
    );
}
