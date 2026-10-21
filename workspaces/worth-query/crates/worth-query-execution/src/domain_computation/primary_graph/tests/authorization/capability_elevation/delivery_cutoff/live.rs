use super::super::super::super::{
    fixture::{
        elevated_account_activity_parameters, Account, AccountSummaryParameters, Activity,
        AuthorizationWorld, ElevatedAccountActivityCause, ElevatedAccountActivityQuery,
        ElevatedAccountActivityResult,
    },
    live_delivery_support::commit_live_activity,
};
use super::super::super::capability_progression::time;
use super::context::{assert_resources_released, context};
use crate::domain_computation::authorization::WorthQueryOperationAuthorizationRefusalKind as AuthorizationRefusal;
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationLiveControls, WorthQueryApplicationLiveOutcome,
    WorthQueryApprovedElevation, WorthQueryElevationClosureKind,
    WorthQueryOperationAuthorizationDenialKind,
};

#[test]
fn revocation_after_a_queued_live_cause_terminates_delivery() {
    let mut context = { context(true) };
    context.script_current_time();
    let capability = context
        .world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            context.elevated_access(&host_custody)
        })
        .expect("the fixture read owns its installed host call");
    let mut lease = context
        .world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            context
        .world
        .selected_product(&host_custody, )
        .open_governed_application_query_live::<
            ElevatedAccountActivityQuery,
            AccountSummaryParameters,
            ElevatedAccountActivityResult,
            _,
            _,
            Account,
            Activity,
            ElevatedAccountActivityCause,
            _,
            _,
            _,
        >(&host_custody.phase_for_application(&context.world.application),
            context.query,
            &context.principal,
            context.account,
            capability,
            elevated_account_activity_parameters("account-1"),
            WorthQueryApplicationLiveControls::bounded(context.request.clone(), 4, 16)
                .unwrap(),
        )
        .unwrap()
        })
        .expect("the fixture read owns its installed host call");
    commit_live_activity(&context.world, &context.committer, &context.request);
    let approved = context.approved.take().unwrap();
    close_elevation(&context.world, &context.request, approved);

    context
        .world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            assert_authorization_denied(
                lease.next(
                    &host_custody.phase_for_application(&context.world.application),
                    &context.principal,
                    &context.request,
                ),
                WorthQueryOperationAuthorizationDenialKind::Refused(
                    AuthorizationRefusal::StaleAuthorization,
                ),
            )
        })
        .expect("the fixture read owns its installed host call");
    context
        .world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            assert!(matches!(
                lease.next(
                    &host_custody.phase_for_application(&context.world.application),
                    &context.principal,
                    &context.request
                ),
                WorthQueryApplicationLiveOutcome::Closed
            ))
        })
        .expect("the fixture read owns its installed host call");
    drop(lease);
    assert_resources_released(&context.world);
}

#[test]
fn query_time_expiry_after_a_queued_live_cause_terminates_delivery() {
    let context = { context(true) };
    context
        .world
        .authorization_time
        .script([time(100), time(100), time(100), time(106)]);
    let capability = context
        .world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            context.elevated_access(&host_custody)
        })
        .expect("the fixture read owns its installed host call");
    let mut lease = context
        .world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            context
        .world
        .selected_product(&host_custody, )
        .open_governed_application_query_live::<
            ElevatedAccountActivityQuery,
            AccountSummaryParameters,
            ElevatedAccountActivityResult,
            _,
            _,
            Account,
            Activity,
            ElevatedAccountActivityCause,
            _,
            _,
            _,
        >(&host_custody.phase_for_application(&context.world.application),
            context.query,
            &context.principal,
            context.account,
            capability,
            elevated_account_activity_parameters("account-1"),
            WorthQueryApplicationLiveControls::bounded(context.request.clone(), 4, 16)
                .unwrap(),
        )
        .unwrap()
        })
        .expect("the fixture read owns its installed host call");
    commit_live_activity(&context.world, &context.committer, &context.request);

    context
        .world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            assert_authorization_denied(
                lease.next(
                    &host_custody.phase_for_application(&context.world.application),
                    &context.principal,
                    &context.request,
                ),
                WorthQueryOperationAuthorizationDenialKind::Refused(
                    AuthorizationRefusal::ElevationExpired,
                ),
            )
        })
        .expect("the fixture read owns its installed host call");
    context
        .world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            assert!(matches!(
                lease.next(
                    &host_custody.phase_for_application(&context.world.application),
                    &context.principal,
                    &context.request
                ),
                WorthQueryApplicationLiveOutcome::Closed
            ))
        })
        .expect("the fixture read owns its installed host call");
    drop(lease);
    assert_resources_released(&context.world);
}

fn assert_authorization_denied(
    outcome: WorthQueryApplicationLiveOutcome<
        ElevatedAccountActivityQuery,
        ElevatedAccountActivityResult,
    >,
    expected: WorthQueryOperationAuthorizationDenialKind,
) {
    let WorthQueryApplicationLiveOutcome::AuthorizationDenied(denial) = outcome else {
        panic!("elevated live delivery did not terminate at authorization re-admission");
    };
    assert_eq!(denial.kind(), expected);
    assert!(denial.identity().is_some());
    assert_eq!(denial.causes(), [expected]);
}

fn close_elevation(
    world: &AuthorizationWorld,
    request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
    approved: WorthQueryApprovedElevation,
) {
    let mandatory = super::super::terminal_lifecycle_support::close_exact(world, request, approved);
    assert_eq!(
        mandatory.closure_kind(),
        WorthQueryElevationClosureKind::Revoked
    );
}
