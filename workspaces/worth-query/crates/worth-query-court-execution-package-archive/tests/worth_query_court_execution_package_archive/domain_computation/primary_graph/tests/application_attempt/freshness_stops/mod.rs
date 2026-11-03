//! A native freshness interruption is visible at each operation door.
use crate::process::domain_computation::primary_graph::tests::application_attempt::*;
use worth_query_execution::facade::domain_computation::WorthQueryOperationAuthorizationDenialKind;
use worth_query_testkit_execution_primary_graph_application_attempt::domain_computation::primary_graph::entity_resolution::interruption_control::FreshnessCancellationGuardForTest;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationAttemptDenialKind;
use worth_query_admission::facade::authenticated_principal::WorthQueryCancellationSource;
use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;
#[test]
fn operation_authorization_keeps_the_scope_freshness_stop() {
    check(false);
}
#[test]
fn decision_observation_keeps_the_identity_freshness_stop() {
    check(true);
}
pub fn check(observe_field: bool) {
    let _observation =
        worth_query_execution::facade::domain_computation::WorthQueryExecutionObservation::install_on_this_thread();
    let world = installed_authorization_world(true);
    let source = WorthQueryCancellationSource::new();
    let request = WorthQueryRequestScope::new(None, source.token());
    let (principal, account) = world
        .application
        .with_host_advancement(|phase| {
            (
                authenticated_principal(&phase.test_host_request(), &world, &request),
                resolved_account(&phase.test_host_request(), &world, "open", &request),
            )
        })
        .unwrap();
    world
        .application
        .with_application_advancement(&request, |phase| {
            let execution = phase
                .checked_execution_request_for(world.application.product_runtime())
                .unwrap();
            if observe_field {
                let admission = admitted_operation(&phase, &world, &principal, &account, &request);
                let mut attempt = world
                    .application
                    .begin_application_read_attempt(&execution, admission)
                    .unwrap();
                let control = FreshnessCancellationGuardForTest::install_on_this_thread(source, 0);
                let refusal = attempt
                    .observe_field(&execution, &account, AccountStatus::reference())
                    .unwrap_err();
                assert!(control.triggered());
                assert_eq!(
                    refusal.kind(),
                    WorthQueryApplicationAttemptDenialKind::Cancelled
                );
            } else {
                let selected = world.selected_product(&phase.test_host_request());
                let operation = world
                    .application
                    .installed_schema()
                    .installed_operation(TouchAccountOperation::reference())
                    .unwrap();
                let control = FreshnessCancellationGuardForTest::install_on_this_thread(source, 0);
                let refusal = selected
                    .authorize_operation(
                        &phase,
                        &principal,
                        &account,
                        &operation,
                        Default::default(),
                        &request,
                    )
                    .err()
                    .unwrap();
                assert!(control.triggered());
                assert_eq!(
                    refusal.kind(),
                    WorthQueryOperationAuthorizationDenialKind::Stopped(Err(worth_query_execution::facade::domain_computation::WorthQueryProviderSessionControlStopKind::Cancelled))
                );
            }
        })
        .unwrap();
}
