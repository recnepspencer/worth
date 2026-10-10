use std::time::Duration;

use crate::domain_computation::primary_graph::application_attempt::Movement;

use worth_query_declaration::facade::application_query::ApplicationQueryParameterSet;
use worth_relational::facade::identity::PartitionId;
use worth_relational::facade::symbols::ClientKey;
use worth_relational::facade::transactions::{
    AspectFieldPatch, CreateIntent, DeleteRelationIntent, EntityReference, MutationIntent,
    RelationMutationIntent, RelationSpec, WorkerIntentBatch,
};

use super::current_controls;
use crate::domain_computation::primary_graph::{
    tests::fixture::{
        cross_root_definition, installed_authorization_world, AccountStatus, CrossRootQuery,
        ScopedAccountSummaryQuery,
    },
    WorthQueryApplicationObservedFact, WorthQueryApplicationQueryAccessContext,
    WorthQueryPrincipalResolutionMode,
};

#[path = "root_selection/result_set.rs"]
mod result_set;

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
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            world
                .application
                .select_product_branch(
                    &host_custody.borrowed_request_for_test(),
                    world.application.product_runtime().default_branch(),
                )
                .expect("the selected product branch remains admitted")
                .resolve_authenticated_principal(
                    &phase,
                    &world.binding,
                    &external,
                    &request,
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap()
        })
        .expect("the fixture read owns its installed host call");
    let account = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            world
                .application
                .select_product_branch(
                    &host_custody.borrowed_request_for_test(),
                    world.application.product_runtime().default_branch(),
                )
                .expect("the selected product branch remains admitted")
                .resolve_entity(
                    &phase,
                    AccountStatus::reference(),
                    "open".to_string(),
                    &request,
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap()
        })
        .expect("the fixture read owns its installed host call");
    let query = world
        .application
        .installed_schema()
        .certification_query(ScopedAccountSummaryQuery::reference())
        .unwrap();
    let access = WorthQueryApplicationQueryAccessContext::new(&principal, &account);
    let plan = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            world
                .selected_product(&(phase).test_host_request())
                .admit_application_query(
                    &phase,
                    &query,
                    &access,
                    ApplicationQueryParameterSet::new(),
                    current_controls(&request),
                )
                .unwrap()
        })
        .expect("the fixture read owns its installed host call");

    assert_eq!(
        plan.graph_read_plan()
            .requirements()
            .counters()
            .predicate_support_count(),
        0
    );
    let result = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            world
                .application
                .execute_application_query_one_shot(&phase, plan)
                .unwrap()
        })
        .expect("the fixture read owns its installed host call");
    assert_eq!(result.rows().len(), 1);
    assert_eq!(result.rows()[0].status(), "open");
    assert_eq!(result.receipt().examined_candidate_count(), 1);
}

mod declared_root_paths_retain_per_row_native_witnesses;
