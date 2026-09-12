use std::num::NonZeroUsize;
use std::time::{Duration, UNIX_EPOCH};
use worth_query_installation::facade::WorthQueryInstalledApplicationQuery;

use super::super::authority::WorthQueryApplicationQueryContinuation;
use crate::domain_computation::primary_graph::{
    tests::fixture::{
        admit_touch_account_capability, governed_live_account_parameters,
        installed_capability_authorization_world, live_scope, Account, AccountIdentity,
        AccountSummaryParameters, AuthorizationWorld, GovernedLiveAccountActivityQuery,
        GovernedLiveAccountActivityResult, IdentityExecutionSchema, Principal,
    },
    WorthQueryApplicationEntityIdentity, WorthQueryApplicationQueryAccessContext,
    WorthQueryApplicationQueryAdmissionDenialKind, WorthQueryApplicationQueryResumeControls,
    WorthQueryAuthenticatedPrincipal, WorthQueryPrincipalResolutionMode,
};

type GovernedContinuation = WorthQueryApplicationQueryContinuation<
    IdentityExecutionSchema,
    GovernedLiveAccountActivityQuery,
    AccountSummaryParameters,
    GovernedLiveAccountActivityResult,
    Account,
>;

#[test]
fn governed_cursor_alone_cannot_resume_disclosure() {
    let (world, principal, account, query, request) = context();
    world
        .authorization_time
        .script(vec![UNIX_EPOCH + Duration::from_secs(100); 32]);
    let continuation = issue(&world, &principal, &account, &query, &request);
    let access = WorthQueryApplicationQueryAccessContext::new(&principal, &account);

    let denial = world
        .application
        .readmit_application_query_continuation(
            &query,
            &access,
            governed_live_account_parameters("account-1"),
            continuation,
            resume_controls(&request),
        )
        .err()
        .expect("a cursor carries no governed disclosure authority");

    assert_eq!(
        denial.kind(),
        WorthQueryApplicationQueryAdmissionDenialKind::DisclosureGovernanceRequired
    );
}

#[test]
fn governed_continuation_consumes_fresh_capability_evidence() {
    let (world, principal, account, query, request) = context();
    world
        .authorization_time
        .script(vec![UNIX_EPOCH + Duration::from_secs(100); 32]);
    let continuation = issue(&world, &principal, &account, &query, &request);
    let fresh = admit_touch_account_capability(&world, &principal, &request).unwrap();
    let access = WorthQueryApplicationQueryAccessContext::new(&principal, &account);
    let plan = world
        .application
        .readmit_governed_application_query_continuation(
            &query,
            &access,
            fresh,
            governed_live_account_parameters("account-1"),
            continuation,
            resume_controls(&request),
        )
        .unwrap();

    let page = world
        .application
        .execute_application_query_continuation_page(plan)
        .unwrap();
    assert_eq!(page.rows().len(), 1);
    assert!(page.receipt().basis_released());
}

fn context() -> (
    AuthorizationWorld,
    WorthQueryAuthenticatedPrincipal<IdentityExecutionSchema, Principal, u64>,
    WorthQueryApplicationEntityIdentity<IdentityExecutionSchema, Account>,
    WorthQueryInstalledApplicationQuery<
        IdentityExecutionSchema,
        GovernedLiveAccountActivityQuery,
        AccountSummaryParameters,
        GovernedLiveAccountActivityResult,
        Account,
    >,
    worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
) {
    let world = installed_capability_authorization_world();
    let request = live_scope();
    let external = world.authenticate("alice", Duration::from_secs(60), &request);
    let principal = world
        .application
        .select_product_branch(world.application.product_runtime().default_branch())
        .expect("the selected product branch remains admitted")
        .resolve_authenticated_principal(
            &world.binding,
            &external,
            &request,
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
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
    let query = world
        .application
        .installed_schema()
        .certification_query(GovernedLiveAccountActivityQuery::reference())
        .unwrap();
    (world, principal, account, query, request)
}

fn issue(
    world: &AuthorizationWorld,
    principal: &WorthQueryAuthenticatedPrincipal<IdentityExecutionSchema, Principal, u64>,
    account: &WorthQueryApplicationEntityIdentity<IdentityExecutionSchema, Account>,
    query: &WorthQueryInstalledApplicationQuery<
        IdentityExecutionSchema,
        GovernedLiveAccountActivityQuery,
        AccountSummaryParameters,
        GovernedLiveAccountActivityResult,
        Account,
    >,
    request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
) -> GovernedContinuation {
    let capability = admit_touch_account_capability(world, principal, request).unwrap();
    let access = WorthQueryApplicationQueryAccessContext::new(principal, account);
    let plan = world
        .selected_product()
        .admit_governed_application_query_continuation(
            query,
            &access,
            capability,
            governed_live_account_parameters("account-1"),
            crate::domain_computation::primary_graph::WorthQueryProductQueryControls::new(
                NonZeroUsize::new(1).unwrap(),
                NonZeroUsize::new(10_000).unwrap(),
                request,
            ),
        )
        .unwrap();
    let page = world
        .application
        .execute_application_query_continuation_page(plan)
        .unwrap();
    page.into_parts()
        .1
        .expect("the two-activity governed query must continue")
}

fn resume_controls(
    request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
) -> WorthQueryApplicationQueryResumeControls<'_> {
    WorthQueryApplicationQueryResumeControls::new(
        NonZeroUsize::new(1).unwrap(),
        NonZeroUsize::new(10_000).unwrap(),
        request,
    )
}
