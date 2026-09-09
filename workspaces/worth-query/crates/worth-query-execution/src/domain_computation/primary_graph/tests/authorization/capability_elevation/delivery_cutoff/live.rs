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
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationLiveControls, WorthQueryApplicationLiveOutcome,
    WorthQueryApprovedElevation, WorthQueryElevationClosureKind,
    WorthQueryOperationAuthorizationDenialKind,
};

#[test]
fn revocation_after_a_queued_live_cause_terminates_delivery() {
    let mut context = context(true);
    context.script_current_time();
    let capability = context.elevated_access();
    let mut lease = context
        .world
        .selected_product()
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
        >(
            context.query,
            &context.principal,
            context.account,
            capability,
            elevated_account_activity_parameters("account-1"),
            WorthQueryApplicationLiveControls::bounded(context.request.clone(), 4, 16, 2_048)
                .unwrap(),
        )
        .unwrap();
    commit_live_activity(&context.world, &context.committer, &context.request);
    let approved = context.approved.take().unwrap();
    close_elevation(&context.world, &context.request, approved);

    assert_authorization_denied(
        lease.poll(),
        WorthQueryOperationAuthorizationDenialKind::StaleAuthorization,
    );
    assert!(matches!(
        lease.poll(),
        WorthQueryApplicationLiveOutcome::Closed
    ));
    drop(lease);
    assert_resources_released(&context.world);
}

#[test]
fn query_time_expiry_after_a_queued_live_cause_terminates_delivery() {
    let context = context(true);
    context
        .world
        .authorization_time
        .script([time(100), time(100), time(100), time(106)]);
    let capability = context.elevated_access();
    let mut lease = context
        .world
        .selected_product()
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
        >(
            context.query,
            &context.principal,
            context.account,
            capability,
            elevated_account_activity_parameters("account-1"),
            WorthQueryApplicationLiveControls::bounded(context.request.clone(), 4, 16, 2_048)
                .unwrap(),
        )
        .unwrap();
    commit_live_activity(&context.world, &context.committer, &context.request);

    assert_authorization_denied(
        lease.poll(),
        WorthQueryOperationAuthorizationDenialKind::ElevationExpired,
    );
    assert!(matches!(
        lease.poll(),
        WorthQueryApplicationLiveOutcome::Closed
    ));
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
