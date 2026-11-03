use std::num::NonZeroUsize;
use std::time::Duration;
use std::time::UNIX_EPOCH;

use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::capability_access_fixture::admit_touch_account_capability;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::capability_world_installation::installed_capability_world_with_label;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::schema_types::live_scope;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::Account;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::AccountIdentity;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::authorization_world_installation::AuthorizationWorld;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::ForbiddenRootGuardQuery;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::GovernedRootGuardQuery;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::IdentityExecutionSchema;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::Principal;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationEntityIdentity;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationQueryAccessContext;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationQueryAdmissionDenialKind;
use worth_query_execution::facade::domain_computation::WorthQueryAuthenticatedPrincipal;
use worth_query_execution::facade::domain_computation::WorthQueryPrincipalResolutionMode;
use worth_query_declaration::facade::application_query::ApplicationQueryParameterSet;

pub struct RootGuardContext {
    pub world: AuthorizationWorld,
    pub request: worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
    pub principal: WorthQueryAuthenticatedPrincipal<IdentityExecutionSchema, Principal, u64>,
    pub account: WorthQueryApplicationEntityIdentity<IdentityExecutionSchema, Account>,
}

#[test]
fn governed_root_guard_changes_membership_only_with_explicit_influence() {
    let matching = execute_governed_root_guard("guard-match");
    let different = execute_governed_root_guard("different-protected-label");

    assert_eq!(matching, 2);
    assert_eq!(different, 0);
}

#[test]
fn forbidden_root_guard_influence_denies_before_result_construction() {
    let context = root_guard_context("guard-match");
    let query = context
        .world
        .application
        .installed_schema()
        .certification_query(ForbiddenRootGuardQuery::reference())
        .unwrap();
    context
        .world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();

            let capability = admit_touch_account_capability(
                &(host_custody.phase_for_application(&context.world.application))
                    .test_host_request(),
                &context.world,
                &context.principal,
                &context.request,
            )
            .unwrap();
            let access =
                WorthQueryApplicationQueryAccessContext::new(&context.principal, &context.account);
            let denial = context
                .world
                .selected_product(
                    &(host_custody.phase_for_application(&context.world.application))
                        .test_host_request(),
                )
                .admit_governed_application_query(
                    &host_custody.phase_for_application(&context.world.application),
                    &query,
                    &access,
                    capability,
                    ApplicationQueryParameterSet::<ForbiddenRootGuardQuery>::new(),
                    current_controls(&context.request),
                )
                .err()
                .expect("a guard without row-presence influence must fail admission");

            assert_eq!(
                denial.kind(),
                WorthQueryApplicationQueryAdmissionDenialKind::DisclosureContractInvalid
            );
        })
        .expect("the fixture block owns its installed host call")
}

pub fn execute_governed_root_guard(label: &str) -> usize {
    let context = root_guard_context(label);
    let query = context
        .world
        .application
        .installed_schema()
        .certification_query(GovernedRootGuardQuery::reference())
        .unwrap();
    context
        .world
        .application
        .with_host_advancement(|phase| {
            let custody = phase.test_host_request();
            let phase_custody = &custody;
            let capability = admit_touch_account_capability(
                phase_custody,
                &context.world,
                &context.principal,
                &context.request,
            )
            .unwrap();
            let access =
                WorthQueryApplicationQueryAccessContext::new(&context.principal, &context.account);
            let plan = context
                .world
                .selected_product(phase_custody)
                .admit_governed_application_query(
                    &phase_custody.phase_for_application(&context.world.application),
                    &query,
                    &access,
                    capability,
                    ApplicationQueryParameterSet::<GovernedRootGuardQuery>::new(),
                    current_controls(&context.request),
                )
                .unwrap();
            let result = context
                .world
                .application
                .execute_application_query_one_shot(
                    &phase_custody.phase_for_application(&context.world.application),
                    plan,
                )
                .unwrap();
            assert!(result.receipt().basis_released());
            result.rows().len()
        })
        .expect("the root guard fixture owns its query call")
}

pub fn root_guard_context(label: &str) -> RootGuardContext {
    let world = installed_capability_world_with_label(label);
    world
        .authorization_time
        .script(vec![UNIX_EPOCH + Duration::from_secs(100); 32]);
    let request = live_scope();
    let external = world.authenticate("alice", Duration::from_secs(60), &request);

    let (principal, account) = world
        .application
        .with_host_advancement(|phase| {
            let host_request = phase.test_host_request();
            let phase_custody = &host_request;
            let borrowed_execution = phase
                .execution_request_for(world.application.product_runtime())
                .expect("the fixture lends its installed phase");
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

            (principal, account)
        })
        .expect("the read-context fixture owns its identity call");
    RootGuardContext {
        world,
        request,
        principal,
        account,
    }
}

pub fn current_controls(
    request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
) -> worth_query_execution::facade::domain_computation::WorthQueryProductQueryControls<'_> {
    worth_query_execution::facade::domain_computation::WorthQueryProductQueryControls::new(
        NonZeroUsize::new(2).unwrap(),
        request,
    )
}
