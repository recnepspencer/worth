use std::num::NonZeroUsize;
use std::time::{Duration, UNIX_EPOCH};

use super::super::super::fixture::{
    admit_touch_account_capability, installed_capability_world_with_label, live_scope, Account,
    AccountIdentity, AuthorizationWorld, ForbiddenRootGuardQuery, GovernedRootGuardQuery,
    IdentityExecutionSchema, Principal,
};
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationEntityIdentity, WorthQueryApplicationQueryAccessContext,
    WorthQueryApplicationQueryAdmissionDenialKind, WorthQueryAuthenticatedPrincipal,
    WorthQueryPrincipalResolutionMode,
};
use worth_query_declaration::facade::application_query::ApplicationQueryParameterSet;

struct RootGuardContext {
    world: AuthorizationWorld,
    request: worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
    principal: WorthQueryAuthenticatedPrincipal<IdentityExecutionSchema, Principal, u64>,
    account: WorthQueryApplicationEntityIdentity<IdentityExecutionSchema, Account>,
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
        .application_query(ForbiddenRootGuardQuery::reference())
        .unwrap();
    let capability =
        admit_touch_account_capability(&context.world, &context.principal, &context.request)
            .unwrap();
    let access = WorthQueryApplicationQueryAccessContext::new(&context.principal, &context.account);
    let denial = context
        .world
        .selected_product()
        .admit_governed_application_query(
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
}

fn execute_governed_root_guard(label: &str) -> usize {
    let context = root_guard_context(label);
    let query = context
        .world
        .application
        .installed_schema()
        .application_query(GovernedRootGuardQuery::reference())
        .unwrap();
    let capability =
        admit_touch_account_capability(&context.world, &context.principal, &context.request)
            .unwrap();
    let access = WorthQueryApplicationQueryAccessContext::new(&context.principal, &context.account);
    let plan = context
        .world
        .selected_product()
        .admit_governed_application_query(
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
        .execute_application_query_one_shot(plan)
        .unwrap();
    assert!(result.receipt().basis_released());
    result.rows().len()
}

fn root_guard_context(label: &str) -> RootGuardContext {
    let world = installed_capability_world_with_label(label);
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
    RootGuardContext {
        world,
        request,
        principal,
        account,
    }
}

fn current_controls(
    request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
) -> crate::domain_computation::primary_graph::WorthQueryProductQueryControls<'_> {
    crate::domain_computation::primary_graph::WorthQueryProductQueryControls::new(
        NonZeroUsize::new(2).unwrap(),
        NonZeroUsize::new(512).unwrap(),
        request,
    )
}
