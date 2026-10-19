use super::super::super::super::fixture::{
    elevated_account_activity_parameters, AuthorizationWorld,
};
use super::context::{assert_resources_released, buffer_limit, context, one};
use crate::domain_computation::authorization::WorthQueryOperationAuthorizationRefusalKind as AuthorizationRefusal;
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationOneShotDenialKind, WorthQueryApplicationQueryAccessContext,
    WorthQueryApplicationQueryAdmissionDenialKind, WorthQueryApplicationQueryResumeControls,
    WorthQueryApprovedElevation, WorthQueryElevationClosureKind,
    WorthQueryOperationAuthorizationDenialKind,
};

#[test]
fn revocation_after_cursor_denies_the_next_page_from_current_truth() {
    let mut context = { context(false) };
    context.script_current_time();
    let access = WorthQueryApplicationQueryAccessContext::new(&context.principal, &context.account);
    let first = context
        .world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            context.elevated_access(&host_custody)
        })
        .expect("the fixture read owns its installed host call");
    let plan = context
        .world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            context
                .world
                .selected_product(&host_custody)
                .admit_governed_application_query_continuation(
                    &host_custody.phase_for_application(&context.world.application),
                    &context.query,
                    &access,
                    first,
                    elevated_account_activity_parameters("account-1"),
                    crate::domain_computation::primary_graph::WorthQueryProductQueryControls::new(
                        one(),
                        Some(buffer_limit()),
                        &context.request,
                    ),
                )
                .unwrap()
        })
        .expect("the fixture read owns its installed host call");
    let page = context
        .world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            context
                .world
                .application
                .execute_application_query_continuation_page(
                    &host_custody.phase_for_application(&context.world.application),
                    plan,
                )
                .unwrap()
        })
        .expect("the fixture read owns its installed host call");
    let (rows, continuation, receipt) = page.into_parts();
    assert_eq!(rows[0].account(), "account-1");
    assert_eq!(rows[0].activities(), &[("activity-primary".to_owned(), 11)]);
    assert!(receipt.basis_released());
    let continuation = continuation.expect("the first activity page must mint a cursor");
    let pending = context
        .world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            context.elevated_access(
                &(host_custody.phase_for_application(&context.world.application))
                    .test_host_request(),
            )
        })
        .expect("the fixture read owns its installed host call");

    let approved = context.approved.take().unwrap();
    close_elevation(&context.world, &context.request, approved);
    let denial = context
        .world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            context
                .world
                .application
                .readmit_governed_application_query_continuation(
                    &host_custody.phase_for_application(&context.world.application),
                    &context.query,
                    &access,
                    pending,
                    elevated_account_activity_parameters("account-1"),
                    continuation,
                    WorthQueryApplicationQueryResumeControls::new(
                        one(),
                        Some(buffer_limit()),
                        &context.request,
                    ),
                )
                .err()
                .expect("revocation after the cursor must deny readmission")
        })
        .expect("the fixture read owns its installed host call");

    assert_eq!(
        denial.kind(),
        WorthQueryApplicationQueryAdmissionDenialKind::Authorization(
            WorthQueryOperationAuthorizationDenialKind::Refused(
                AuthorizationRefusal::StaleAuthorization
            ),
        )
    );
    assert_resources_released(&context.world);
}

#[test]
fn revocation_after_selected_plan_denies_before_result_delivery() {
    let mut context = { context(false) };
    context.script_current_time();
    let access = WorthQueryApplicationQueryAccessContext::new(&context.principal, &context.account);
    let capability = context
        .world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            context.elevated_access(&host_custody)
        })
        .expect("the fixture read owns its installed host call");
    let plan = context
        .world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            context
                .world
                .selected_product(&host_custody)
                .admit_governed_application_query(
                    &host_custody.phase_for_application(&context.world.application),
                    &context.query,
                    &access,
                    capability,
                    elevated_account_activity_parameters("account-1"),
                    crate::domain_computation::primary_graph::WorthQueryProductQueryControls::new(
                        one(),
                        Some(buffer_limit()),
                        &context.request,
                    ),
                )
                .unwrap()
        })
        .expect("the fixture read owns its installed host call");

    let approved = context.approved.take().unwrap();
    close_elevation(&context.world, &context.request, approved);
    let denial = context
        .world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            context
                .world
                .application
                .execute_application_query_one_shot(
                    &host_custody.phase_for_application(&context.world.application),
                    plan,
                )
                .err()
                .expect("selected data must not pin elevation authority")
        })
        .expect("the fixture read owns its installed host call");

    assert_stale_authorization(denial.kind());
    assert_resources_released(&context.world);
}

fn assert_stale_authorization(kind: WorthQueryApplicationOneShotDenialKind) {
    assert_eq!(
        kind,
        WorthQueryApplicationOneShotDenialKind::Authorization(
            WorthQueryOperationAuthorizationDenialKind::Refused(
                AuthorizationRefusal::StaleAuthorization
            ),
        )
    );
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
