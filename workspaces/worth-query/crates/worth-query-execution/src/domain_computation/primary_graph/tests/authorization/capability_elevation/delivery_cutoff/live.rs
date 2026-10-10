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
    crate::domain_computation::primary_graph::with_test_advancement(|host_custody| {
        let mut context = context(
            &host_custody,
            &host_custody.borrowed_request_for_test(),
            true,
        );
        context.script_current_time();
        let capability = context.elevated_access(&host_custody);
        let mut lease = context
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
            WorthQueryApplicationLiveControls::bounded(context.request.clone(), 4, 16, 2_048)
                .unwrap(),
        )
        .unwrap();
        commit_live_activity(&context.world, &context.committer, &context.request);
        let approved = context.approved.take().unwrap();
        close_elevation(&host_custody, &context.world, &context.request, approved);

        assert_authorization_denied(
            lease.next(
                &host_custody.phase_for_application(&context.world.application),
                &context.principal,
                &context.request,
            ),
            WorthQueryOperationAuthorizationDenialKind::StaleAuthorization,
        );
        assert!(matches!(
            lease.next(
                &host_custody.phase_for_application(&context.world.application),
                &context.principal,
                &context.request
            ),
            WorthQueryApplicationLiveOutcome::Closed
        ));
        drop(lease);
        assert_resources_released(&context.world);
    })
}

#[test]
fn query_time_expiry_after_a_queued_live_cause_terminates_delivery() {
    crate::domain_computation::primary_graph::with_test_advancement(|host_custody| {
        let context = context(
            &host_custody,
            &host_custody.borrowed_request_for_test(),
            true,
        );
        context
            .world
            .authorization_time
            .script([time(100), time(100), time(100), time(106)]);
        let capability = context.elevated_access(&host_custody);
        let mut lease = context
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
            WorthQueryApplicationLiveControls::bounded(context.request.clone(), 4, 16, 2_048)
                .unwrap(),
        )
        .unwrap();
        commit_live_activity(&context.world, &context.committer, &context.request);

        assert_authorization_denied(
            lease.next(
                &host_custody.phase_for_application(&context.world.application),
                &context.principal,
                &context.request,
            ),
            WorthQueryOperationAuthorizationDenialKind::ElevationExpired,
        );
        assert!(matches!(
            lease.next(
                &host_custody.phase_for_application(&context.world.application),
                &context.principal,
                &context.request
            ),
            WorthQueryApplicationLiveOutcome::Closed
        ));
        drop(lease);
        assert_resources_released(&context.world);
    })
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
    phase_custody: &crate::domain_computation::primary_graph::TestHostRequest<'_>,
    world: &AuthorizationWorld,
    request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
    approved: WorthQueryApprovedElevation,
) {
    let mandatory = super::super::terminal_lifecycle_support::close_exact(
        phase_custody,
        world,
        request,
        approved,
    );
    assert_eq!(
        mandatory.closure_kind(),
        WorthQueryElevationClosureKind::Revoked
    );
}
