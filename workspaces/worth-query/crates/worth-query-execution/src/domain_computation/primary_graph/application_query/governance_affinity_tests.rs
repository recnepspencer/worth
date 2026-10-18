use std::num::NonZeroUsize;
use std::time::{Duration, UNIX_EPOCH};

use worth_query_admission::facade::application_query::admit_application_query_parameters;

use super::super::tests::fixture::{
    admit_touch_account_capability, governed_live_account_parameters,
    installed_capability_live_world_with_label, live_scope, Account, AccountIdentity,
    AuthorizationWorld, GovernedAccountOmissionQuery, GovernedLiveAccountActivityQuery,
    IdentityExecutionSchema, Principal,
};
use super::{WorthQueryAdmittedApplicationQueryPlan, WorthQueryApplicationQueryAccessContext};
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationEntityIdentity, WorthQueryAuthenticatedPrincipal,
    WorthQueryPrincipalResolutionMode,
};

struct GovernanceContext {
    world: AuthorizationWorld,
    request: worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
    principal: WorthQueryAuthenticatedPrincipal<IdentityExecutionSchema, Principal, u64>,
    account: WorthQueryApplicationEntityIdentity<IdentityExecutionSchema, Account>,
}

#[test]
fn governance_matches_only_the_managed_session_that_minted_it() {
    let context = governance_context("primary");
    let query = context
        .world
        .application
        .installed_schema()
        .certification_query(GovernedLiveAccountActivityQuery::reference())
        .unwrap();
    let access = WorthQueryApplicationQueryAccessContext::new(&context.principal, &context.account);
    let foreign = governance_context("foreign");
    let foreign_query = foreign
        .world
        .application
        .installed_schema()
        .certification_query(GovernedLiveAccountActivityQuery::reference())
        .unwrap();
    let foreign_access =
        WorthQueryApplicationQueryAccessContext::new(&foreign.principal, &foreign.account);
    let foreign_plan = foreign
        .world
        .application
        .with_host_advancement(|phase| {
            admit_plan(&phase, &foreign, &foreign_query, &foreign_access)
        })
        .expect("the foreign fixture owns its admission call");
    context
        .world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();

            let first = admit_plan(
                &host_custody.phase_for_application(&context.world.application),
                &context,
                &query,
                &access,
            );
            let second = admit_plan(
                &host_custody.phase_for_application(&context.world.application),
                &context,
                &query,
                &access,
            );

            assert!(governance_matches_plan(&first, &first));
            assert!(!governance_matches_plan(&first, &second));
            assert_ne!(
                first.graph_work_session_identity(),
                second.graph_work_session_identity()
            );
            assert_ne!(
                first.graph_work_managed_run_identity(),
                second.graph_work_managed_run_identity()
            );

            assert!(!first.governance.computation_matches(
                &foreign_plan.graph_work,
                foreign_plan.runtime_authority,
                first.query.identity(),
                first.parameters.identity(),
                first.principal.principal_entity_id(),
                first.scope.entity_id(),
            ));
        })
        .expect("the fixture block owns its installed host call")
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
            let host_custody = phase.test_host_request();
            let borrowed_execution = host_custody.borrowed_request_for_test();
            let execution = &borrowed_execution;
            let plan = admit_plan(
                &host_custody.phase_for_application(&context.world.application),
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
                .select_product_branch(
                    execution,
                    context.world.application.product_runtime().default_branch(),
                )
                .expect("the selected product branch remains admitted")
                .resolve_entity(
                    &host_custody.phase_for_application(&context.world.application),
                    AccountIdentity::reference(),
                    "account-2".to_owned(),
                    &context.request,
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap();

            assert!(!plan.governance.computation_matches(
                &plan.graph_work,
                plan.runtime_authority,
                alternate_query.identity(),
                plan.parameters.identity(),
                plan.principal.principal_entity_id(),
                plan.scope.entity_id(),
            ));
            assert!(!plan.governance.computation_matches(
                &plan.graph_work,
                plan.runtime_authority,
                plan.query.identity(),
                alternate_parameters.identity(),
                plan.principal.principal_entity_id(),
                plan.scope.entity_id(),
            ));
            assert!(!plan.governance.computation_matches(
                &plan.graph_work,
                plan.runtime_authority,
                plan.query.identity(),
                plan.parameters.identity(),
                bob.principal_entity_id(),
                plan.scope.entity_id(),
            ));
            assert!(!plan.governance.computation_matches(
                &plan.graph_work,
                plan.runtime_authority,
                plan.query.identity(),
                plan.parameters.identity(),
                plan.principal.principal_entity_id(),
                account_two.entity_id(),
            ));

            for matches in [
                plan.governance.readmission_matches(
                    plan.runtime_authority,
                    alternate_query.identity(),
                    plan.parameters.identity(),
                    plan.principal.principal_entity_id(),
                    plan.scope.entity_id(),
                ),
                plan.governance.readmission_matches(
                    plan.runtime_authority,
                    plan.query.identity(),
                    alternate_parameters.identity(),
                    plan.principal.principal_entity_id(),
                    plan.scope.entity_id(),
                ),
                plan.governance.readmission_matches(
                    plan.runtime_authority,
                    plan.query.identity(),
                    plan.parameters.identity(),
                    bob.principal_entity_id(),
                    plan.scope.entity_id(),
                ),
                plan.governance.readmission_matches(
                    plan.runtime_authority,
                    plan.query.identity(),
                    plan.parameters.identity(),
                    plan.principal.principal_entity_id(),
                    account_two.entity_id(),
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

fn governance_context(label: &str) -> GovernanceContext {
    let world = installed_capability_live_world_with_label(label);
    world.authorization_time.script(std::iter::repeat_n(
        UNIX_EPOCH + Duration::from_secs(100),
        16,
    ));
    let request = live_scope();

    let (principal, account) = world
        .application
        .with_host_advancement(|phase| {
            let host_request = phase.test_host_request();
            let phase_custody = &host_request;
            let borrowed_execution = phase
                .execution_request_for(world.application.product_runtime())
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
    GovernanceContext {
        world,
        request,
        principal,
        account,
    }
}

fn resolve_principal(
    phase_custody: &crate::domain_computation::primary_graph::TestHostRequest<'_>,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    world: &AuthorizationWorld,
    request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
    subject: &str,
) -> WorthQueryAuthenticatedPrincipal<IdentityExecutionSchema, Principal, u64> {
    let external = world.authenticate(subject, Duration::from_secs(60), request);
    world
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
            request,
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .unwrap()
}

fn admit_plan<'a>(
    phase_custody: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
    context: &'a GovernanceContext,
    query: &'a worth_query_installation::facade::WorthQueryInstalledApplicationQuery<
        IdentityExecutionSchema,
        GovernedLiveAccountActivityQuery,
        super::super::tests::fixture::AccountSummaryParameters,
        super::super::tests::fixture::GovernedLiveAccountActivityResult,
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
    super::super::tests::fixture::AccountSummaryParameters,
    super::super::tests::fixture::GovernedLiveAccountActivityResult,
    Principal,
    u64,
    Account,
> {
    let capability = admit_touch_account_capability(
        &(phase_custody).test_host_request(),
        &context.world,
        &context.principal,
        &context.request,
    )
    .unwrap();
    context
        .world
        .selected_product(&(phase_custody).test_host_request())
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

fn governance_matches_plan(
    owner: &WorthQueryAdmittedApplicationQueryPlan<
        '_,
        IdentityExecutionSchema,
        GovernedLiveAccountActivityQuery,
        super::super::tests::fixture::AccountSummaryParameters,
        super::super::tests::fixture::GovernedLiveAccountActivityResult,
        Principal,
        u64,
        Account,
    >,
    candidate: &WorthQueryAdmittedApplicationQueryPlan<
        '_,
        IdentityExecutionSchema,
        GovernedLiveAccountActivityQuery,
        super::super::tests::fixture::AccountSummaryParameters,
        super::super::tests::fixture::GovernedLiveAccountActivityResult,
        Principal,
        u64,
        Account,
    >,
) -> bool {
    owner.governance.computation_matches(
        &candidate.graph_work,
        candidate.runtime_authority,
        owner.query.identity(),
        owner.parameters.identity(),
        owner.principal.principal_entity_id(),
        owner.scope.entity_id(),
    )
}

fn controls(
    request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
) -> crate::domain_computation::primary_graph::WorthQueryProductQueryControls<'_> {
    crate::domain_computation::primary_graph::WorthQueryProductQueryControls::new(
        NonZeroUsize::new(4).unwrap(),
        Some(NonZeroUsize::new(512).unwrap()),
        request,
    )
}
