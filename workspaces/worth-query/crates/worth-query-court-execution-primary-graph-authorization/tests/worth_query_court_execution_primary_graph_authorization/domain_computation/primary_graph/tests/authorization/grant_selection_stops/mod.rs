//! Cancellation during grant selection reaches the actual admission door.
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::authenticated_principal;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::capability_world_installation::installed_capability_authorization_world;
use crate::domain_computation::primary_graph::tests::authorization::capability_progression::admitted_capability_access;
use crate::domain_computation::primary_graph::tests::authorization::capability_progression::time;
#[test]
fn grant_selection_keeps_a_cancellation_raised_after_admission_checks() {
    check(false);
}
#[test]
fn grant_selection_keeps_a_cancellation_at_the_native_observation() {
    check(true);
}
pub fn check(at_observation: bool) {
    let _observation =
        worth_query_execution::facade::domain_computation::WorthQueryExecutionObservation::install_on_this_thread();
    let world = installed_capability_authorization_world();
    world.authorization_time.hold(time(100));
    let cancellation =
        worth_query_admission::facade::authenticated_principal::WorthQueryCancellationSource::new();
    let scope = worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope::new(
        Some(std::time::Instant::now() + std::time::Duration::from_secs(60)),
        cancellation.token(),
        worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system(),
    );
    let principal = world
        .application
        .with_host_advancement(|phase| {
            authenticated_principal(&phase.test_host_request_for_test(), &world, &scope)
        })
        .unwrap();
    world.application.with_application_advancement(&scope, |phase| {
        let _late = if at_observation {
            worth_query_execution::court_doors_for_test::domain_computation::authorization::capability_observation::grant_selection::late_cancellation::CancelGrantSelectionGuardForTest::at_observation_on_this_thread(cancellation.clone())
        } else {
            worth_query_execution::court_doors_for_test::domain_computation::authorization::capability_observation::grant_selection::late_cancellation::CancelGrantSelectionGuardForTest::install_on_this_thread(cancellation.clone())
        };
        let refusal = admitted_capability_access(&phase.test_host_request_for_test(), &world, &principal, &scope, 100).err().unwrap();
        assert_eq!(refusal.kind(), worth_query_execution::facade::domain_computation::WorthQueryOperationAuthorizationDenialKind::Stopped(Err(worth_query_execution::facade::domain_computation::WorthQueryProviderSessionControlStopKind::Cancelled)));
    }).unwrap();
}
