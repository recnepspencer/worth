use std::num::NonZeroUsize;
use std::time::Duration;

use worth_query_declaration::facade::application_query::ApplicationQueryParameterSet;

use super::fixture::{
    installed_authorization_world, installed_two_principal_authorization_world, live_scope,
    status_parameter, Account, AccountStatus, AccountSummaryParameters, AccountSummaryQuery,
    AccountSummaryResult, ForgedSelectorQuery, ForgedSelectorResult, GovernedAccountSummaryQuery,
    IdentityExecutionSchema, LiveAccountActivityQuery, LiveAccountActivityResult,
    NestedAccountQuery, NestedAccountResult, OrderedAccountSummaryQuery,
};
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationQueryAccessContext, WorthQueryApplicationQueryAdmissionDenialKind,
    WorthQueryPrincipalResolutionMode,
};
mod disclosure_noninterference;
mod graph_work_capacity;
mod identity_convergence;
mod lifecycle;
mod lifecycle_mutations;
mod optional_result_presence;
mod planning_budget;
mod result_projection;
mod root_guard_basis;
mod root_selection;
mod runtime_support;
mod scope_liveness;

fn installed_query(
    world: &super::fixture::AuthorizationWorld,
) -> worth_query_installation::facade::WorthQueryInstalledApplicationQuery<
    IdentityExecutionSchema,
    AccountSummaryQuery,
    AccountSummaryParameters,
    AccountSummaryResult,
    Account,
> {
    world
        .application
        .installed_schema()
        .application_query(AccountSummaryQuery::reference())
        .unwrap()
}

fn installed_governed_query(
    world: &super::fixture::AuthorizationWorld,
) -> worth_query_installation::facade::WorthQueryInstalledApplicationQuery<
    IdentityExecutionSchema,
    GovernedAccountSummaryQuery,
    AccountSummaryParameters,
    AccountSummaryResult,
    Account,
> {
    world
        .application
        .installed_schema()
        .application_query(GovernedAccountSummaryQuery::reference())
        .unwrap()
}

fn installed_ordered_query(
    world: &super::fixture::AuthorizationWorld,
) -> worth_query_installation::facade::WorthQueryInstalledApplicationQuery<
    IdentityExecutionSchema,
    OrderedAccountSummaryQuery,
    AccountSummaryParameters,
    AccountSummaryResult,
    Account,
> {
    world
        .application
        .installed_schema()
        .application_query(OrderedAccountSummaryQuery::reference())
        .unwrap()
}

fn installed_live_query(
    world: &super::fixture::AuthorizationWorld,
) -> worth_query_installation::facade::WorthQueryInstalledApplicationQuery<
    IdentityExecutionSchema,
    LiveAccountActivityQuery,
    AccountSummaryParameters,
    LiveAccountActivityResult,
    Account,
> {
    world
        .application
        .installed_schema()
        .application_query(LiveAccountActivityQuery::reference())
        .unwrap()
}

fn installed_nested_query(
    world: &super::fixture::AuthorizationWorld,
) -> worth_query_installation::facade::WorthQueryInstalledApplicationQuery<
    IdentityExecutionSchema,
    NestedAccountQuery,
    AccountSummaryParameters,
    NestedAccountResult,
    Account,
> {
    world
        .application
        .installed_schema()
        .application_query(NestedAccountQuery::reference())
        .unwrap()
}

fn installed_forged_selector_query(
    world: &super::fixture::AuthorizationWorld,
) -> worth_query_installation::facade::WorthQueryInstalledApplicationQuery<
    IdentityExecutionSchema,
    ForgedSelectorQuery,
    AccountSummaryParameters,
    ForgedSelectorResult,
    Account,
> {
    world
        .application
        .installed_schema()
        .application_query(ForgedSelectorQuery::reference())
        .unwrap()
}

fn current_controls(
    request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
) -> crate::domain_computation::primary_graph::WorthQueryProductQueryControls<'_> {
    crate::domain_computation::primary_graph::WorthQueryProductQueryControls::new(
        NonZeroUsize::new(10).unwrap(),
        NonZeroUsize::new(10_000).unwrap(),
        request,
    )
}

#[test]
fn execution_runtime_mints_plan_from_exact_mapped_principal_and_typed_scope() {
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
    let account = world
        .application
        .select_product_branch(world.application.product_runtime().default_branch())
        .expect("the selected product branch remains admitted")
        .resolve_entity(
            AccountStatus::reference(),
            "open".to_string(),
            &request,
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .unwrap();
    let query = installed_query(&world);
    let access = WorthQueryApplicationQueryAccessContext::new(&principal, &account);

    let plan = world
        .selected_product()
        .admit_application_query(
            &query,
            &access,
            ApplicationQueryParameterSet::new().bind(status_parameter(), "open".to_string()),
            current_controls(&request),
        )
        .unwrap();

    assert_eq!(plan.query_identity(), query.identity());
    assert_eq!(plan.scope().binding_identity(), query.binding_identity());
    assert!(plan.graph_read_plan().is_admitted());
    assert_eq!(plan.graph_work_branch(), plan.basis_identity().branch_id());
    assert_eq!(
        plan.graph_work_principal_entity_id(),
        principal.principal_entity_id()
    );
    assert_eq!(plan.graph_work_scope_entity_id(), Some(account.entity_id()));
    assert!(plan.graph_work_runtime_ordinal() > 0);
    assert!(plan.graph_work_managed_run_identity().as_u64() > 0);
    assert!(plan.graph_work_decision_fact_count() > 0);
    assert!(!plan.graph_work_provider().is_empty());
    assert_eq!(
        plan.graph_read_plan()
            .budget_check()
            .max_inline_intermediate_set_size(),
        10_000
    );

    let result = world
        .application
        .execute_application_query_one_shot(plan)
        .unwrap();
    assert_eq!(result.rows().len(), 1);
    assert_eq!(result.rows()[0].status(), "open");
    assert_eq!(result.rows()[0].label(), "primary");
    assert_eq!(result.receipt().query_identity(), query.identity());
    assert_eq!(result.receipt().projected_record_count(), 1);
    assert_eq!(result.receipt().projected_field_count(), 2);
    assert_eq!(result.receipt().fallback_count(), 0);
    assert_eq!(result.receipt().edge_scan_count(), 0);
    assert_eq!(result.receipt().per_result_neighbor_lookup_count(), 0);
    assert!(result.receipt().basis_released());
}

#[test]
fn mapped_stranger_cannot_admit_a_valid_foreign_account_scope() {
    let world = installed_two_principal_authorization_world(true);
    let request = live_scope();
    let external = world.authenticate("bob", Duration::from_secs(60), &request);
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
            AccountStatus::reference(),
            "open".to_string(),
            &request,
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .unwrap();
    let query = installed_query(&world);
    let access = WorthQueryApplicationQueryAccessContext::new(&principal, &account);

    let denial = world
        .selected_product()
        .admit_application_query(
            &query,
            &access,
            ApplicationQueryParameterSet::new().bind(status_parameter(), "open".to_string()),
            current_controls(&request),
        )
        .err()
        .expect("a mapped stranger must not receive query plan authority");

    assert_eq!(
        denial.kind(),
        WorthQueryApplicationQueryAdmissionDenialKind::Authorization(
            crate::domain_computation::primary_graph::WorthQueryOperationAuthorizationDenialKind::PermissionDenied,
        )
    );
}

#[test]
fn foreign_scope_and_missing_disclosure_governance_open_no_plan_authority() {
    let world = installed_authorization_world(true);
    let foreign = installed_authorization_world(true);
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
    let foreign_account = foreign
        .application
        .select_product_branch(foreign.application.product_runtime().default_branch())
        .expect("the selected product branch remains admitted")
        .resolve_entity(
            AccountStatus::reference(),
            "open".to_string(),
            &request,
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .unwrap();
    let query = installed_query(&world);
    let crossed = WorthQueryApplicationQueryAccessContext::new(&principal, &foreign_account);
    let denial = world
        .selected_product()
        .admit_application_query(
            &query,
            &crossed,
            ApplicationQueryParameterSet::new().bind(status_parameter(), "open".to_string()),
            current_controls(&request),
        )
        .err()
        .expect("foreign scope must deny");
    assert_eq!(
        denial.kind(),
        WorthQueryApplicationQueryAdmissionDenialKind::ForeignScope
    );

    let governed = installed_governed_query(&world);
    let local_account = world
        .application
        .select_product_branch(world.application.product_runtime().default_branch())
        .expect("the selected product branch remains admitted")
        .resolve_entity(
            AccountStatus::reference(),
            "open".to_string(),
            &request,
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .unwrap();
    let local = WorthQueryApplicationQueryAccessContext::new(&principal, &local_account);
    let denial = world
        .selected_product()
        .admit_application_query(
            &governed,
            &local,
            ApplicationQueryParameterSet::new().bind(status_parameter(), "open".to_string()),
            current_controls(&request),
        )
        .err()
        .expect("missing disclosure governance must deny");
    assert_eq!(
        denial.kind(),
        WorthQueryApplicationQueryAdmissionDenialKind::DisclosureGovernanceRequired
    );
}

#[test]
fn path_bound_ordering_mechanism_opens_exact_plan_authority() {
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
    let account = world
        .application
        .select_product_branch(world.application.product_runtime().default_branch())
        .expect("the selected product branch remains admitted")
        .resolve_entity(
            AccountStatus::reference(),
            "open".to_string(),
            &request,
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .unwrap();
    let query = installed_ordered_query(&world);
    let access = WorthQueryApplicationQueryAccessContext::new(&principal, &account);
    let plan = world
        .selected_product()
        .admit_application_query(
            &query,
            &access,
            ApplicationQueryParameterSet::new().bind(status_parameter(), "open".to_string()),
            current_controls(&request),
        )
        .unwrap();
    assert!(plan
        .graph_read_plan()
        .requirements()
        .requires_kind(
            worth_query_admission::facade::graph_read_access::WorthQueryGraphReadAccessRequirementKind::OrderingSupport,
        ));
    let result = world
        .application
        .execute_application_query_one_shot(plan)
        .unwrap();
    assert_eq!(result.rows()[0].label(), "primary");
}
