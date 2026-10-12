//! Cancellation during grant selection reaches the actual admission door.
use super::super::application_attempt::authenticated_principal;
use super::super::fixture::installed_capability_authorization_world;
use super::capability_progression::{admitted_capability_access, time};
#[test]
fn grant_selection_keeps_a_cancellation_raised_after_admission_checks() {
    check(false);
}
#[test]
fn grant_selection_keeps_a_cancellation_at_the_native_observation() {
    check(true);
}
fn check(at_observation: bool) {
    let _observation =
        crate::facade::primary_graph::WorthQueryExecutionObservation::install_on_this_thread();
    let world = installed_capability_authorization_world();
    world.authorization_time.hold(time(100));
    let cancellation =
        worth_query_admission::facade::authenticated_principal::WorthQueryCancellationSource::new();
    let scope = worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope::new(
        Some(std::time::Instant::now() + std::time::Duration::from_secs(60)),
        cancellation.token(),
    );
    let principal = world
        .application
        .with_host_advancement(|phase| {
            authenticated_principal(&phase.test_host_request(), &world, &scope)
        })
        .unwrap();
    world.application.with_application_advancement(&scope, |phase| {
        let _late = if at_observation {
            crate::domain_computation::authorization::CancelGrantSelectionGuardForTest::at_observation_on_this_thread(cancellation.clone())
        } else {
            crate::domain_computation::authorization::CancelGrantSelectionGuardForTest::install_on_this_thread(cancellation.clone())
        };
        let refusal = admitted_capability_access(&phase.test_host_request(), &world, &principal, &scope, 100).err().unwrap();
        assert_eq!(refusal.kind(), crate::domain_computation::authorization::WorthQueryOperationAuthorizationDenialKind::Stopped(Err(crate::domain_computation::WorthQueryProviderSessionControlStopKind::Cancelled)));
    }).unwrap();
}
