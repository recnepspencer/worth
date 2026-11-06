use std::num::NonZeroUsize;
use std::time::Duration;
use std::time::UNIX_EPOCH;

use worth_query_admission::facade::application_query::admit_application_query_parameters;

use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::capability_access_fixture::admit_touch_account_capability;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::governed_live_query::governed_live_account_parameters;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::capability_world_installation::installed_capability_live_world_with_label;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::schema_types::live_scope;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::Account;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::AccountIdentity;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::authorization_world_installation::AuthorizationWorld;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::GovernedAccountOmissionQuery;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::GovernedLiveAccountActivityQuery;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::IdentityExecutionSchema;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::Principal;
use worth_query_execution::facade::domain_computation::WorthQueryAdmittedApplicationQueryPlan;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationQueryAccessContext;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationEntityIdentity;
use worth_query_execution::facade::domain_computation::WorthQueryAuthenticatedPrincipal;
use worth_query_execution::facade::domain_computation::WorthQueryPrincipalResolutionMode;

pub struct GovernanceContext {
    pub world: AuthorizationWorld,
    pub request: worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
    pub principal: WorthQueryAuthenticatedPrincipal<IdentityExecutionSchema, Principal, u64>,
    pub account: WorthQueryApplicationEntityIdentity<IdentityExecutionSchema, Account>,
}

#[test]
fn governance_rejects_real_query_parameter_principal_and_scope_substitution() {
    let context = governance_context("primary");
    let query = context
        .world
        .application
        .installed_schema()
        .certification_query(GovernedLiveAccountActivityQuery::reference())
        .unwrap();
    let access = WorthQueryApplicationQueryAccessContext::new(&context.principal, &context.account);
    context
        .world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request_for_test();
            let borrowed_execution = host_custody.borrowed_request_for_test();
            let execution = &borrowed_execution;
            let plan = admit_plan(
                &host_custody.phase_for_application_for_test(&context.world.application),
                &context,
                &query,
                &access,
            );
            let alternate_query = context
                .world
                .application
                .installed_schema()
                .certification_query(GovernedAccountOmissionQuery::reference())
                .unwrap();
            let alternate_parameters = admit_application_query_parameters(
                &query,
                governed_live_account_parameters("account-2"),
            )
            .unwrap();
            let bob = resolve_principal(
                &host_custody,
                &host_custody.borrowed_request_for_test(),
                &context.world,
                &context.request,
                "bob",
            );
            let account_two = context
                .world
                .application
                .select_product_branch_for_test(
                    execution,
                    context
                        .world
                        .application
                        .product_runtime()
                        .default_branch_for_test(),
                )
                .expect("the selected product branch remains admitted")
                .resolve_entity(
                    &host_custody.phase_for_application_for_test(&context.world.application),
                    AccountIdentity::at("AccountIdentity"),
                    "account-2".to_owned(),
                    &context.request,
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap();

            assert!(!(*plan.governance_for_test()).computation_matches_for_test(
                &plan.graph_work,
                (*plan.runtime_authority_for_test()),
                alternate_query.identity(),
                (*plan.parameters_for_test()).identity(),
                plan.principal.principal_entity_id_for_test(),
                (*plan.scope_for_test()).entity_id_for_test(),
            ));
            assert!(!(*plan.governance_for_test()).computation_matches_for_test(
                &plan.graph_work,
                (*plan.runtime_authority_for_test()),
                (*plan.query_for_test()).identity(),
                alternate_parameters.identity(),
                plan.principal.principal_entity_id_for_test(),
                (*plan.scope_for_test()).entity_id_for_test(),
            ));
            assert!(!(*plan.governance_for_test()).computation_matches_for_test(
                &plan.graph_work,
                (*plan.runtime_authority_for_test()),
                (*plan.query_for_test()).identity(),
                (*plan.parameters_for_test()).identity(),
                bob.principal_entity_id_for_test(),
                (*plan.scope_for_test()).entity_id_for_test(),
            ));
            assert!(!(*plan.governance_for_test()).computation_matches_for_test(
                &plan.graph_work,
                (*plan.runtime_authority_for_test()),
                (*plan.query_for_test()).identity(),
                (*plan.parameters_for_test()).identity(),
                plan.principal.principal_entity_id_for_test(),
                account_two.entity_id_for_test(),
            ));

            for matches in [
                (*plan.governance_for_test()).readmission_matches_for_test(
                    (*plan.runtime_authority_for_test()),
                    alternate_query.identity(),
                    (*plan.parameters_for_test()).identity(),
                    plan.principal.principal_entity_id_for_test(),
                    (*plan.scope_for_test()).entity_id_for_test(),
                ),
                (*plan.governance_for_test()).readmission_matches_for_test(
                    (*plan.runtime_authority_for_test()),
                    (*plan.query_for_test()).identity(),
                    alternate_parameters.identity(),
                    plan.principal.principal_entity_id_for_test(),
                    (*plan.scope_for_test()).entity_id_for_test(),
                ),
                (*plan.governance_for_test()).readmission_matches_for_test(
                    (*plan.runtime_authority_for_test()),
                    (*plan.query_for_test()).identity(),
                    (*plan.parameters_for_test()).identity(),
                    bob.principal_entity_id_for_test(),
                    (*plan.scope_for_test()).entity_id_for_test(),
                ),
                (*plan.governance_for_test()).readmission_matches_for_test(
                    (*plan.runtime_authority_for_test()),
                    (*plan.query_for_test()).identity(),
                    (*plan.parameters_for_test()).identity(),
                    plan.principal.principal_entity_id_for_test(),
                    account_two.entity_id_for_test(),
                ),
            ] {
                assert!(
                    !matches,
                    "readmission must retain every pre-session affinity"
                );
            }
        })
        .expect("the fixture block owns its installed host call")
}

pub fn governance_context(label: &str) -> GovernanceContext {
    let world = installed_capability_live_world_with_label(label);
    world.authorization_time.script(std::iter::repeat_n(
        UNIX_EPOCH + Duration::from_secs(100),
        16,
    ));
    let request = live_scope();

    let (principal, account) = world
        .application
        .with_host_advancement(|phase| {
            let host_request = phase.test_host_request_for_test();
            let phase_custody = &host_request;
            let borrowed_execution = phase
                .execution_request_for_for_test(world.application.product_runtime())
                .expect("the fixture lends its installed phase");
            let execution = &borrowed_execution;
            let principal = resolve_principal(
                phase_custody,
                &(phase_custody).borrowed_request_for_test(),
                &world,
                &request,
                "alice",
            );
            let account = world
                .application
                .select_product_branch_for_test(
                    execution,
                    world
                        .application
                        .product_runtime()
                        .default_branch_for_test(),
                )
                .expect("the selected product branch remains admitted")
                .resolve_entity(
                    &phase_custody.phase_for_application_for_test(&world.application),
                    AccountIdentity::at("AccountIdentity"),
                    "account-1".to_owned(),
                    &request,
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap();

            (principal, account)
        })
        .expect("the read-context fixture owns its identity call");
    GovernanceContext {
        world,
        request,
        principal,
        account,
    }
}

pub fn resolve_principal(
    phase_custody: &worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::request_execution::advancement::test_host_policy::TestHostRequest<'_>,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    world: &AuthorizationWorld,
    request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
    subject: &str,
) -> WorthQueryAuthenticatedPrincipal<IdentityExecutionSchema, Principal, u64> {
    let external = world.authenticate(subject, Duration::from_secs(60), request);
    world
        .application
        .select_product_branch_for_test(
            execution,
            world
                .application
                .product_runtime()
                .default_branch_for_test(),
        )
        .expect("the selected product branch remains admitted")
        .resolve_authenticated_principal(
            &phase_custody.phase_for_application_for_test(&world.application),
            &world.binding,
            &external,
            request,
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .unwrap()
}

pub fn admit_plan<'a>(
    phase_custody: &worth_query_execution::facade::application_contribution::WorthQueryAdvancementPhase<'_>,
    context: &'a GovernanceContext,
    query: &'a worth_query_installation::facade::WorthQueryInstalledApplicationQuery<
        IdentityExecutionSchema,
        GovernedLiveAccountActivityQuery,
        worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::application_queries::AccountSummaryParameters,
        worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::governed_live_query::GovernedLiveAccountActivityResult,
        Account,
    >,
    access: &WorthQueryApplicationQueryAccessContext<
        'a,
        IdentityExecutionSchema,
        Principal,
        u64,
        Account,
    >,
) -> WorthQueryAdmittedApplicationQueryPlan<
    'a,
    IdentityExecutionSchema,
    GovernedLiveAccountActivityQuery,
    worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::application_queries::AccountSummaryParameters,
    worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::governed_live_query::GovernedLiveAccountActivityResult,
    Principal,
    u64,
    Account,
>{
    let capability = admit_touch_account_capability(
        &(phase_custody).test_host_request_for_test(),
        &context.world,
        &context.principal,
        &context.request,
    )
    .unwrap();
    context
        .world
        .selected_product(&(phase_custody).test_host_request_for_test())
        .admit_governed_application_query(
            phase_custody,
            query,
            access,
            capability,
            governed_live_account_parameters("account-1"),
            controls(&context.request),
        )
        .unwrap()
}

pub fn governance_matches_plan(
    owner: &WorthQueryAdmittedApplicationQueryPlan<
        '_,
        IdentityExecutionSchema,
        GovernedLiveAccountActivityQuery,
        worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::application_queries::AccountSummaryParameters,
        worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::governed_live_query::GovernedLiveAccountActivityResult,
        Principal,
        u64,
        Account,
    >,
    candidate: &WorthQueryAdmittedApplicationQueryPlan<
        '_,
        IdentityExecutionSchema,
        GovernedLiveAccountActivityQuery,
        worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::application_queries::AccountSummaryParameters,
        worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::governed_live_query::GovernedLiveAccountActivityResult,
        Principal,
        u64,
        Account,
    >,
) -> bool {
    (*owner.governance_for_test()).computation_matches_for_test(
        &candidate.graph_work,
        (*candidate.runtime_authority_for_test()),
        (*owner.query_for_test()).identity(),
        (*owner.parameters_for_test()).identity(),
        owner.principal.principal_entity_id_for_test(),
        (*owner.scope_for_test()).entity_id_for_test(),
    )
}

pub fn controls(
    request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
) -> worth_query_execution::facade::domain_computation::WorthQueryProductQueryControls<'_> {
    worth_query_execution::facade::domain_computation::WorthQueryProductQueryControls::new(
        NonZeroUsize::new(4).unwrap(),
        request,
    )
}
#[path = "session_minted_governance/mod.rs"]
pub mod session_minted_governance;
