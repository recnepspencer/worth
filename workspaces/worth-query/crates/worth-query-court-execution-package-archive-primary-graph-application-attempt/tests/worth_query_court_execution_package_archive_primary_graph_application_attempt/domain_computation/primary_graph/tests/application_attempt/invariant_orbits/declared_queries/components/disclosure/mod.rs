//! Whole-slot omission refuses before Native acquisition; guards retain separate authority.
use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::declared_queries::components::{Allowed, Missing, Omitted};
use crate::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::declared_queries::components::*;
#[test]
fn governed_components_refuse_closed_slots_before_native_and_guard_reads() {
    let _isolation =
        worth_query_testkit_execution_primary_graph_application_attempt::domain_computation::primary_graph::tests::fixture::reconstruction_execution::isolated_request_owner();
    let world = interleaved_world();
    let _observation =
        worth_query_execution::facade::domain_computation::WorthQueryExecutionObservation::install_on_this_thread();
    let count=worth_query_execution::domain_computation::primary_graph::application_query::read_execution::entry_observation::component_native_acquisitions_on_this_thread_for_test;
    let before = count();
    let allowed = worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::declared_queries::read::run(&world, Allowed::reference(), 0);
    assert_eq!(
        allowed
            .components(selection::<OrbitSchema<true>, Allowed>())
            .unwrap()
            .len(),
        2
    );
    assert!(
        count() > before,
        "positive control must enter original Native acquisition"
    );
    let guarded = worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::declared_queries::read::run(&world, GuardAllowed::reference(), 0);
    assert!(guarded.rows().is_empty());
    assert!(
        guarded
            .components(selection::<OrbitSchema<true>, GuardAllowed>())
            .unwrap()
            .is_empty(),
        "an admitted false guard yields genuine complete empty evidence"
    );
    let host = worth_query_testkit_execution_primary_graph_application_attempt_2::test_request_policy::serial_request();
    let execution = worth_execution::ExecutionRequest::serial(&host);
    let request = worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::schema_types::live_scope();
    let before = count();
    let denied = worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::declared_queries::read::execute(&world, &execution, &request, Omitted::reference(), 0)
        .err()
        .unwrap();
    assert!(
        matches!(denied,worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::declared_queries::read::Stop::Read(ref stop) if stop.kind()==WorthQueryApplicationOneShotDenialKind::ProjectionUnavailable && stop.subject()=="shells"),
        "{denied:?}"
    );
    assert_eq!(count(), before);
    let denied = worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::declared_queries::read::execute(&world, &execution, &request, Missing::reference(), 0)
        .err()
        .unwrap();
    assert!(
        matches!(denied,worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::declared_queries::read::Stop::Admission(ref stop) if stop.kind()==WorthQueryApplicationQueryAdmissionDenialKind::DisclosureContractInvalid),
        "{denied:?}"
    );
    assert_eq!(count(), before);
    let denied = worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::declared_queries::read::execute(&world, &execution, &request, GuardDenied::reference(), 0)
        .err()
        .unwrap();
    assert!(
        matches!(denied,worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::declared_queries::read::Stop::Admission(ref stop) if stop.kind()==WorthQueryApplicationQueryAdmissionDenialKind::DisclosureContractInvalid),
        "{denied:?}"
    );
    assert_eq!(
        count(),
        before,
        "guard authorization precedes census cohort acquisition"
    );
    let denied = worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::declared_queries::read::execute(
        &world,
        &execution,
        &request,
        AggregateDenied::reference(),
        0,
    )
    .err()
    .unwrap();
    assert!(
        matches!(denied,worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::declared_queries::read::Stop::Admission(ref stop) if stop.kind()==WorthQueryApplicationQueryAdmissionDenialKind::DisclosureContractInvalid),
        "{denied:?}"
    );
    assert_eq!(
        count(),
        before,
        "aggregate identity permission is independently required"
    );
    let denied = worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::declared_queries::read::execute(&world, &execution, &request, OrderingDenied::reference(), 0)
        .err()
        .unwrap();
    assert!(
        matches!(denied,worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::declared_queries::read::Stop::Admission(ref stop) if stop.kind()==WorthQueryApplicationQueryAdmissionDenialKind::DisclosureContractInvalid),
        "{denied:?}"
    );
    assert_eq!(
        count(),
        before,
        "component ordering permission is independently required"
    );
}
