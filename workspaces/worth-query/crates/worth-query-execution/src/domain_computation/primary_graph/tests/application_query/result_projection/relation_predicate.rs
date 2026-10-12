use std::time::Duration;

use super::super::current_controls;
use crate::domain_computation::primary_graph::tests::fixture::{
    installed_authorization_world, live_scope, selected_activity_parameters, Account,
    AccountStatus, SelectedActivityParameters, SelectedActivityQuery, SelectedActivityResult,
};
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationQueryAccessContext, WorthQueryPrincipalResolutionMode,
};

#[test]
fn filters_siblings_before_child_projection() {
    let world = installed_authorization_world(true);
    let request = live_scope();
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
                .unwrap()
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
                .unwrap()
                .resolve_entity(
                    &phase,
                    AccountStatus::reference(),
                    "open".to_owned(),
                    &request,
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap()
        })
        .expect("the fixture read owns its installed host call");
    let query = world
        .application
        .installed_schema()
        .certification_query::<
            SelectedActivityQuery,
            SelectedActivityParameters,
            SelectedActivityResult,
            Account,
        >(SelectedActivityQuery::reference())
        .unwrap();
    let access = WorthQueryApplicationQueryAccessContext::new(&principal, &account);
    let plan = world
        .application
        .with_host_advancement(|phase| {
            world
                .selected_product(&(phase).test_host_request())
                .admit_application_query(
                    &phase,
                    &query,
                    &access,
                    selected_activity_parameters("open", "activity-primary"),
                    current_controls(&request),
                )
                .unwrap()
        })
        .expect("the fixture read owns its installed host call");

    let result = world
        .application
        .with_host_advancement(|phase| {
            world
                .application
                .execute_application_query_one_shot(&phase, plan)
                .expect("the selected related entity projects without its sibling")
        })
        .expect("the fixture read owns its installed host call");

    assert_eq!(result.rows()[0].sequence(), 11);
    assert_eq!(result.receipt().projected_record_count(), 2);
    assert_eq!(result.receipt().projected_field_count(), 1);
    assert!(result.receipt().work().predicate_work_units() >= 5);
    let source = result.observed_sources()[0].footprint_for_test();
    assert_eq!(source.entities.len(), 3);
    assert_eq!(source.aspects.len(), 4);
    assert!(
        source.aspects.iter().any(|aspect| {
            aspect.entity == account.entity_id()
                && aspect.aspect.as_str() == AccountStatus::reference().aspect()
        }),
        "the root predicate is a source dependency even when its field is not projected"
    );
    assert_eq!(source.adjacencies.len(), 1);
    assert_eq!(source.adjacencies[0].endpoints.len(), 2);
}
