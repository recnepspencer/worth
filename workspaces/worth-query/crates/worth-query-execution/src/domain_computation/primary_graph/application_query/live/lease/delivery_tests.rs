use std::time::{Duration, UNIX_EPOCH};

use super::super::{
    WorthQueryApplicationLiveCloseOutcome, WorthQueryApplicationLiveControls,
    WorthQueryApplicationLiveOutcome,
};
use crate::domain_computation::primary_graph::tests::{
    fixture::{
        admit_touch_account_capability, governed_live_account_parameters,
        installed_authorization_world, installed_capability_live_world, live_account_parameters,
        live_scope, revoke_current_capability, Account, AccountIdentity, AccountSummaryParameters,
        Activity, GovernedLiveAccountActivityCause, GovernedLiveAccountActivityQuery,
        GovernedLiveAccountActivityResult, LiveAccountActivityCause, LiveAccountActivityQuery,
        LiveAccountActivityResult,
    },
    live_delivery_support::commit_live_activity,
};
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationDisclosed, WorthQueryApplicationDisclosureReceiptPosture,
    WorthQueryOperationAuthorizationDenialKind, WorthQueryPrincipalResolutionMode,
};

#[test]
fn committed_live_cause_projects_with_bounded_result_buffer_evidence() {
    let world = installed_authorization_world(true);
    let request = live_scope();
    let external = world.authenticate("alice", Duration::from_secs(60), &request);
    let principal = world
        .application
        .select_product_branch(world.application.product_runtime().default_branch())
        .expect("the selected product branch remains admitted")
        .resolve_authenticated_principal(
            &world.binding,
            external,
            &request,
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .unwrap();
    let query = world
        .application
        .installed_schema()
        .application_query(LiveAccountActivityQuery::reference())
        .unwrap();
    let account = world
        .application
        .select_product_branch(world.application.product_runtime().default_branch())
        .expect("the selected product branch remains admitted")
        .resolve_entity(
            AccountIdentity::reference(),
            "account-1".to_owned(),
            &request,
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .unwrap();
    let observer = world.application.result_buffer_observer();
    let mut lease = world
        .selected_product()
        .open_application_query_live::<
            LiveAccountActivityQuery,
            AccountSummaryParameters,
            LiveAccountActivityResult,
            _,
            _,
            Account,
            Activity,
            LiveAccountActivityCause,
        >(
            query,
            &principal,
            account,
            live_account_parameters("account-1"),
            WorthQueryApplicationLiveControls::bounded(request.clone(), 4, 16, 2_048).unwrap(),
        )
        .unwrap();
    let committed = commit_live_activity(&world, &principal, &request);

    let WorthQueryApplicationLiveOutcome::Delivered(update) = lease.poll() else {
        panic!("the committed installed live cause must deliver");
    };
    assert_eq!(update.commit_id(), committed.commit_id());
    assert_eq!(update.result().account(), "account-1");
    assert_eq!(
        update.result().activities(),
        &[("activity-primary".to_owned(), 11)]
    );
    let buffer = update
        .receipt()
        .result_buffer()
        .expect("live projection must carry bounded result-buffer evidence");
    assert!(buffer.released());
    assert!(buffer.peak_bytes() > 0);
    assert!(buffer.peak_bytes() <= buffer.limit_bytes());
    assert_eq!(observer.observe().active_buffers(), 0);
    assert_eq!(observer.observe().retained_bytes(), 0);
    let WorthQueryApplicationLiveCloseOutcome::Completed(completion) = lease.close() else {
        panic!("live close must complete its opening graph-read session");
    };
    assert_eq!(completion.release().released_reservation_count(), 1);
}

#[test]
fn governed_live_delivery_reuses_only_query_owned_current_authority() {
    let world = installed_capability_live_world();
    world
        .authorization_time
        .script(vec![UNIX_EPOCH + Duration::from_secs(100); 32]);
    let request = live_scope();
    let external = world.authenticate("alice", Duration::from_secs(60), &request);
    let principal = world
        .application
        .select_product_branch(world.application.product_runtime().default_branch())
        .expect("the selected product branch remains admitted")
        .resolve_authenticated_principal(
            &world.binding,
            external,
            &request,
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .unwrap();
    let committer_external = world.authenticate("bob", Duration::from_secs(60), &request);
    let committer = world
        .application
        .select_product_branch(world.application.product_runtime().default_branch())
        .expect("the selected product branch remains admitted")
        .resolve_authenticated_principal(
            &world.binding,
            committer_external,
            &request,
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .unwrap();
    let query = world
        .application
        .installed_schema()
        .application_query(GovernedLiveAccountActivityQuery::reference())
        .unwrap();
    let account = world
        .application
        .select_product_branch(world.application.product_runtime().default_branch())
        .expect("the selected product branch remains admitted")
        .resolve_entity(
            AccountIdentity::reference(),
            "account-1".to_owned(),
            &request,
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .unwrap();
    let capability = admit_touch_account_capability(&world, &principal, &request).unwrap();
    let mut lease = world
        .selected_product()
        .open_governed_application_query_live::<
            GovernedLiveAccountActivityQuery,
            AccountSummaryParameters,
            GovernedLiveAccountActivityResult,
            _,
            _,
            Account,
            Activity,
            GovernedLiveAccountActivityCause,
            _,
            _,
            _,
        >(
            query,
            &principal,
            account,
            capability,
            governed_live_account_parameters("account-1"),
            WorthQueryApplicationLiveControls::bounded(request.clone(), 4, 16, 2_048).unwrap(),
        )
        .unwrap();
    let committed = commit_live_activity(&world, &committer, &request);

    let WorthQueryApplicationLiveOutcome::Delivered(update) = lease.poll() else {
        panic!("current governed authority must deliver the committed cause");
    };
    assert_eq!(update.commit_id(), committed.commit_id());
    assert_eq!(update.result().account(), "account-1");
    assert_eq!(
        update.result().activities(),
        &[("activity-primary".to_owned(), 11)]
    );
    assert!(matches!(
        update.result().label(),
        WorthQueryApplicationDisclosed::Omitted(_)
    ));
    assert_eq!(
        update.receipt().disclosure().posture(),
        WorthQueryApplicationDisclosureReceiptPosture::Governed
    );
}

#[test]
fn revoked_capability_stops_governed_live_delivery_before_projection() {
    let world = installed_capability_live_world();
    world
        .authorization_time
        .script(vec![UNIX_EPOCH + Duration::from_secs(100); 32]);
    let request = live_scope();
    let external = world.authenticate("alice", Duration::from_secs(60), &request);
    let principal = world
        .application
        .select_product_branch(world.application.product_runtime().default_branch())
        .expect("the selected product branch remains admitted")
        .resolve_authenticated_principal(
            &world.binding,
            external,
            &request,
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .unwrap();
    let committer_external = world.authenticate("bob", Duration::from_secs(60), &request);
    let committer = world
        .application
        .select_product_branch(world.application.product_runtime().default_branch())
        .expect("the selected product branch remains admitted")
        .resolve_authenticated_principal(
            &world.binding,
            committer_external,
            &request,
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .unwrap();
    let query = world
        .application
        .installed_schema()
        .application_query(GovernedLiveAccountActivityQuery::reference())
        .unwrap();
    let account = world
        .application
        .select_product_branch(world.application.product_runtime().default_branch())
        .expect("the selected product branch remains admitted")
        .resolve_entity(
            AccountIdentity::reference(),
            "account-1".to_owned(),
            &request,
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .unwrap();
    let capability = admit_touch_account_capability(&world, &principal, &request).unwrap();
    let mut lease = world
        .selected_product()
        .open_governed_application_query_live::<
            GovernedLiveAccountActivityQuery,
            AccountSummaryParameters,
            GovernedLiveAccountActivityResult,
            _,
            _,
            Account,
            Activity,
            GovernedLiveAccountActivityCause,
            _,
            _,
            _,
        >(
            query,
            &principal,
            account,
            capability,
            governed_live_account_parameters("account-1"),
            WorthQueryApplicationLiveControls::bounded(request.clone(), 4, 16, 2_048).unwrap(),
        )
        .unwrap();
    commit_live_activity(&world, &committer, &request);
    revoke_current_capability(&world);

    let outcome = lease.poll();
    let WorthQueryApplicationLiveOutcome::AuthorizationDenied(denial) = outcome else {
        let posture = match outcome {
            WorthQueryApplicationLiveOutcome::Delivered(_) => "delivered",
            WorthQueryApplicationLiveOutcome::Pending => "pending",
            WorthQueryApplicationLiveOutcome::Overflow(_) => "overflow",
            WorthQueryApplicationLiveOutcome::StalePrincipal => "stale-principal",
            WorthQueryApplicationLiveOutcome::StaleScope => "stale-scope",
            WorthQueryApplicationLiveOutcome::ProjectionDenied(_) => "projection-denied",
            WorthQueryApplicationLiveOutcome::CauseDenied(_) => "cause-denied",
            WorthQueryApplicationLiveOutcome::Cancelled => "cancelled",
            WorthQueryApplicationLiveOutcome::DeadlineExceeded => "deadline-exceeded",
            WorthQueryApplicationLiveOutcome::Closed => "closed",
            WorthQueryApplicationLiveOutcome::Unavailable => "unavailable",
            WorthQueryApplicationLiveOutcome::AuthorizationDenied(_) => unreachable!(),
        };
        panic!("revoked governed live authority returned {posture}");
    };
    assert_eq!(
        denial.kind(),
        WorthQueryOperationAuthorizationDenialKind::StaleAuthorization
    );
    assert!(denial.identity().is_some());
    assert_eq!(
        denial.causes(),
        [WorthQueryOperationAuthorizationDenialKind::StaleAuthorization]
    );
}
