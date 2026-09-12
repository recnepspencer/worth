use std::time::Duration;

use worth_query_declaration::facade::application_query::ApplicationQueryParameterSet;

use super::current_controls;
use crate::domain_computation::primary_graph::{
    tests::fixture::{
        cross_root_definition, installed_authorization_world, AccountStatus, CrossRootQuery,
        ScopedAccountSummaryQuery,
    },
    WorthQueryApplicationQueryAccessContext, WorthQueryPrincipalResolutionMode,
};

#[test]
fn root_path_guard_literal_changes_canonical_query_identity() {
    let open = cross_root_definition("open").into_erased();
    let closed = cross_root_definition("closed").into_erased();

    assert_ne!(open.canonical_basis(), closed.canonical_basis());
}

#[test]
fn exact_scope_root_executes_without_an_invented_predicate() {
    let world = installed_authorization_world(true);
    let request = super::super::fixture::live_scope();
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
            AccountStatus::reference(),
            "open".to_string(),
            &request,
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .unwrap();
    let query = world
        .application
        .installed_schema()
        .certification_query(ScopedAccountSummaryQuery::reference())
        .unwrap();
    let access = WorthQueryApplicationQueryAccessContext::new(&principal, &account);
    let plan = world
        .selected_product()
        .admit_application_query(
            &query,
            &access,
            ApplicationQueryParameterSet::new(),
            current_controls(&request),
        )
        .unwrap();

    assert_eq!(
        plan.graph_read_plan()
            .requirements()
            .counters()
            .predicate_support_count(),
        0
    );
    let result = world
        .application
        .execute_application_query_one_shot(plan)
        .unwrap();
    assert_eq!(result.rows().len(), 1);
    assert_eq!(result.rows()[0].status(), "open");
    assert_eq!(result.receipt().examined_candidate_count(), 1);
}

#[test]
fn declared_root_paths_union_and_deduplicate_before_projection() {
    let world = installed_authorization_world(true);
    let request = super::super::fixture::live_scope();
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
            AccountStatus::reference(),
            "open".to_string(),
            &request,
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .unwrap();
    let query = world
        .application
        .installed_schema()
        .certification_query(CrossRootQuery::reference())
        .unwrap();
    let access = WorthQueryApplicationQueryAccessContext::new(&principal, &account);
    let plan = world
        .selected_product()
        .admit_application_query(
            &query,
            &access,
            ApplicationQueryParameterSet::new(),
            current_controls(&request),
        )
        .unwrap();
    assert_eq!(
        plan.graph_read_plan()
            .requirements()
            .counters()
            .predicate_support_count(),
        1
    );
    let predicate_fields = plan
        .graph_read_plan()
        .requirements()
        .rows()
        .iter()
        .flat_map(|row| row.predicate_field_authorities())
        .map(|field| field.native_field_key().as_str())
        .collect::<Vec<_>>();
    assert_eq!(predicate_fields, ["AccountStatus"]);
    let result = world
        .application
        .execute_application_query_one_shot(plan)
        .unwrap();

    assert_eq!(
        result
            .rows()
            .iter()
            .map(|row| row.sequence())
            .collect::<Vec<_>>(),
        [11, 22]
    );
    assert_eq!(result.receipt().result_count(), 2);
    assert_eq!(result.receipt().adjacency_list_read_count(), 3);
    assert_eq!(result.receipt().edge_scan_count(), 4);
    assert_eq!(result.receipt().examined_candidate_count(), 3);
    assert_eq!(result.receipt().work().predicate_work_units(), 6);
    assert_eq!(result.receipt().fallback_count(), 0);
    assert_eq!(result.receipt().per_result_neighbor_lookup_count(), 0);
}
