//! Stops raised at the native lookup retain the ordinary resolution answer.
use super::*;
use crate::domain_computation::primary_graph::entity_resolution::interruption_control::FreshnessCancellationGuardForTest;
use worth_query_admission::facade::authenticated_principal::{
    WorthQueryCancellationSource, WorthQueryRequestScope,
};
#[test]
fn ordinary_resolution_keeps_lookup_cancellation_and_deadline() {
    let _observation =
        crate::facade::primary_graph::WorthQueryExecutionObservation::install_on_this_thread();
    for deadline_stop in [false, true] {
        let world = installed_authorization_world(true);
        let selected = world
            .application
            .with_host_advancement(|phase| world.selected_product(&phase.test_host_request()))
            .unwrap();
        let cancellation = WorthQueryCancellationSource::new();
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(1);
        let request =
            WorthQueryRequestScope::new(deadline_stop.then_some(deadline), cancellation.token());
        world
            .application
            .with_application_advancement(&request, |phase| {
                let control = if deadline_stop {
                    FreshnessCancellationGuardForTest::deadline_on_this_thread(deadline)
                } else {
                    FreshnessCancellationGuardForTest::install_on_this_thread(cancellation, 0)
                };
                let denial = selected
                    .resolve_entity(
                        &phase,
                        AccountStatus::reference(),
                        "open".to_owned(),
                        &request,
                        WorthQueryPrincipalResolutionMode::Ordinary,
                    )
                    .err()
                    .unwrap();
                assert!(control.triggered(), "the native lookup ran after admission");
                assert_eq!(
                    denial.kind(),
                    if deadline_stop {
                        WorthQueryEntityResolutionDenialKind::DeadlineExceeded
                    } else {
                        WorthQueryEntityResolutionDenialKind::Cancelled
                    }
                );
                assert_eq!(denial.subject(), "AccountStatus");
            })
            .unwrap();
    }
}
