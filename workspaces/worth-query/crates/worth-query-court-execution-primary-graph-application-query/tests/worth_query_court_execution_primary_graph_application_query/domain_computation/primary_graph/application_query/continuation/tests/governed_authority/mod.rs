use std::num::NonZeroUsize;
use std::time::Duration;
use std::time::UNIX_EPOCH;
use worth_query_installation::facade::WorthQueryInstalledApplicationQuery;

use worth_query_execution::facade::domain_computation::WorthQueryApplicationQueryContinuation;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::capability_access_fixture::admit_touch_account_capability;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::governed_live_query::governed_live_account_parameters;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::capability_world_installation::installed_capability_authorization_world;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::schema_types::live_scope;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::Account;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::AccountIdentity;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::application_queries::AccountSummaryParameters;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::authorization_world_installation::AuthorizationWorld;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::GovernedLiveAccountActivityQuery;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::governed_live_query::GovernedLiveAccountActivityResult;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::IdentityExecutionSchema;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::Principal;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationEntityIdentity;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationQueryAccessContext;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationQueryAdmissionDenialKind;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationQueryResumeControls;
use worth_query_execution::facade::domain_computation::WorthQueryAuthenticatedPrincipal;
use worth_query_execution::facade::domain_computation::WorthQueryPrincipalResolutionMode;

pub type GovernedContinuation = WorthQueryApplicationQueryContinuation<
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
    world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();

            let continuation = issue(
                &host_custody,
                &world,
                &principal,
                &account,
                &query,
                &request,
            );
            let access = WorthQueryApplicationQueryAccessContext::new(&principal, &account);

            let denial = world
                .application
                .readmit_application_query_continuation(
                    &host_custody.phase_for_application(&world.application),
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
        })
        .expect("the fixture block owns its installed host call")
}

#[test]
fn governed_continuation_consumes_fresh_capability_evidence() {
    let (world, principal, account, query, request) = context();
    world
        .authorization_time
        .script(vec![UNIX_EPOCH + Duration::from_secs(100); 32]);
    world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();

            let continuation = issue(
                &host_custody,
                &world,
                &principal,
                &account,
                &query,
                &request,
            );
            let fresh = admit_touch_account_capability(&host_custody, &world, &principal, &request)
                .unwrap();
            let access = WorthQueryApplicationQueryAccessContext::new(&principal, &account);
            let plan = world
                .application
                .readmit_governed_application_query_continuation(
                    &host_custody.phase_for_application(&world.application),
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
                .execute_application_query_continuation_page(
                    &host_custody.phase_for_application(&world.application),
                    plan,
                )
                .unwrap();
            assert_eq!(page.rows().len(), 1);
            assert!(page.receipt().basis_released());
        })
        .expect("the fixture block owns its installed host call")
}

pub fn context() -> (
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
    let (principal, account, query) = world
        .application
        .with_host_advancement(|phase| {
            let phase_custody = phase.test_host_request();
            let borrowed_execution = phase
                .execution_request_for(world.application.product_runtime())
                .expect("the context lends its installed phase");
            let execution = &borrowed_execution;
            let principal = world
                .application
                .select_product_branch(
                    execution,
                    world.application.product_runtime().default_branch(),
                )
                .expect("the selected product branch remains admitted")
                .resolve_authenticated_principal(
                    &phase_custody.phase_for_application(&world.application),
                    &world.binding,
                    &external,
                    &request,
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap();
            let account = world
                .application
                .select_product_branch(
                    execution,
                    world.application.product_runtime().default_branch(),
                )
                .expect("the selected product branch remains admitted")
                .resolve_entity(
                    &phase_custody.phase_for_application(&world.application),
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
            (principal, account, query)
        })
        .expect("the governed context owns its identity read");
    (world, principal, account, query, request)
}

pub fn issue(
    phase_custody: &worth_query_execution::domain_computation::primary_graph::application_contribution::request_execution::advancement::test_host_policy::TestHostRequest<'_>,
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
    let capability =
        admit_touch_account_capability(phase_custody, world, principal, request).unwrap();
    let access = WorthQueryApplicationQueryAccessContext::new(principal, account);
    let plan = world
        .selected_product(phase_custody)
        .admit_governed_application_query_continuation(
            &phase_custody.phase_for_application(&world.application),
            query,
            &access,
            capability,
            governed_live_account_parameters("account-1"),
            worth_query_execution::facade::domain_computation::WorthQueryProductQueryControls::new(
                NonZeroUsize::new(1).unwrap(),
                request,
            ),
        )
        .unwrap();
    let page = world
        .application
        .execute_application_query_continuation_page(
            &phase_custody.phase_for_application(&world.application),
            plan,
        )
        .unwrap();
    page.into_parts()
        .1
        .expect("the two-activity governed query must continue")
}

pub fn resume_controls(
    request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
) -> WorthQueryApplicationQueryResumeControls<'_> {
    WorthQueryApplicationQueryResumeControls::new(NonZeroUsize::new(1).unwrap(), request)
}
