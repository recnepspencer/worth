//! Interruption at native freshness lookup reaches the real query door.
use super::*;
use crate::domain_computation::primary_graph::{
    entity_resolution::interruption_control::FreshnessCancellationGuardForTest,
    WorthQueryApplicationOneShotDenialKind,
};
use worth_query_admission::facade::authenticated_principal::{
    WorthQueryCancellationSource, WorthQueryRequestScope,
};
#[derive(Clone, Copy)]
enum Door {
    Admission,
    CurrentRead,
    BasisRead,
}
#[test]
fn admission_keeps_cancellation_at_the_native_freshness_lookup() {
    check(Door::Admission);
}
#[test]
fn one_shot_keeps_cancellation_at_current_scope_freshness() {
    check(Door::CurrentRead);
}
#[test]
fn one_shot_keeps_cancellation_at_retained_scope_freshness() {
    check(Door::BasisRead);
}
fn check(door: Door) {
    let _observation =
        crate::facade::primary_graph::WorthQueryExecutionObservation::install_on_this_thread();
    let world = installed_authorization_world(true);
    let cancellation = WorthQueryCancellationSource::new();
    let request = WorthQueryRequestScope::new(None, cancellation.token());
    let external = world.authenticate("alice", Duration::from_secs(60), &request);
    let (selected, principal, account) = world
        .application
        .with_host_advancement(|phase| {
            let selected = world.selected_product(&phase.test_host_request());
            let principal = selected
                .resolve_authenticated_principal(
                    &phase,
                    &world.binding,
                    &external,
                    &request,
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap();
            let account = selected
                .resolve_entity(
                    &phase,
                    AccountStatus::reference(),
                    "open".to_owned(),
                    &request,
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap();
            (selected, principal, account)
        })
        .unwrap();
    let query = installed_query(&world);
    let access = WorthQueryApplicationQueryAccessContext::new(&principal, &account);
    if matches!(door, Door::Admission) {
        world
            .application
            .with_application_advancement(&request, |phase| {
                let execution = phase
                    .checked_execution_request_for(world.application.product_runtime())
                    .unwrap();
                let selected = selected.retain_selection(&execution).unwrap();
                let control =
                    FreshnessCancellationGuardForTest::install_on_this_thread(cancellation, 0);
                let refusal = selected
                    .admit_application_query(
                        &phase,
                        &query,
                        &access,
                        ApplicationQueryParameterSet::new()
                            .bind(status_parameter(), "open".to_owned())
                            .unwrap(),
                        current_controls(&request),
                    )
                    .err()
                    .unwrap();
                assert!(
                    control.triggered(),
                    "the actual native lookup boundary must have been reached"
                );
                assert_eq!(
                    refusal.kind(),
                    WorthQueryApplicationQueryAdmissionDenialKind::Cancelled
                );
            })
            .unwrap();
    } else {
        let plan = world
            .application
            .with_host_advancement(|phase| {
                let host = phase.test_host_request();
                selected
                    .retain_selection(&host.borrowed_request_for_test())
                    .unwrap()
                    .admit_application_query(
                        &phase,
                        &query,
                        &access,
                        ApplicationQueryParameterSet::new()
                            .bind(status_parameter(), "open".to_owned())
                            .unwrap(),
                        current_controls(&request),
                    )
                    .unwrap()
            })
            .unwrap();
        world
            .application
            .with_application_advancement(&request, |phase| {
                let preceding_reads = usize::from(matches!(door, Door::BasisRead));
                let control = FreshnessCancellationGuardForTest::install_on_this_thread(
                    cancellation,
                    preceding_reads,
                );
                let refusal = world
                    .application
                    .execute_application_query_one_shot(&phase, plan)
                    .err()
                    .unwrap();
                assert!(
                    control.triggered(),
                    "the actual native lookup boundary must have been reached"
                );
                assert_eq!(
                    refusal.kind(),
                    WorthQueryApplicationOneShotDenialKind::Cancelled
                );
            })
            .unwrap();
    }
}
