use std::num::NonZeroUsize;
use std::time::Duration;

use worth_query_admission::facade::graph_read_access::WorthQueryGraphReadAccessRequirementKind;
use worth_query_declaration::facade::application_query::ApplicationQueryParameterSet;

use super::{
    current_controls, installed_forged_selector_query, installed_nested_query, installed_query,
};
use crate::domain_computation::primary_graph::tests::fixture::{
    installed_authorization_world, installed_authorization_world_with_label, live_scope,
    status_parameter, AccountStatus,
};

mod relation_predicate;
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationOneShotDenialKind, WorthQueryApplicationProjectionDenialKind,
    WorthQueryApplicationQueryAccessContext, WorthQueryApplicationQueryOmissionPosture,
    WorthQueryPrincipalResolutionMode,
};

mod nested_projection_preserves_sibling_slots_cardinality_and_direct;

#[test]
fn root_result_limit_does_not_cap_nested_dependency_records() {
    // Root cardinality is independent of nested dependency records.
    let world = super::super::fixture::installed_authorization_world_with_resource_profile(
        crate::domain_computation::execution_runtime::WorthQueryApplicationQueryResourceProfile::new(std::num::NonZeroUsize::new(64).expect("concurrent graph work is non-zero")),
    );
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
    let query = installed_nested_query(&world);
    let access = WorthQueryApplicationQueryAccessContext::new(&principal, &account);
    let controls = crate::domain_computation::primary_graph::WorthQueryProductQueryControls::new(
        NonZeroUsize::new(1).unwrap(),
        &request,
    );
    let plan = world
        .application
        .with_host_advancement(|phase| {
            world
                .selected_product(&(phase).test_host_request())
                .admit_application_query(
                    &phase,
                    &query,
                    &access,
                    ApplicationQueryParameterSet::new()
                        .bind(status_parameter(), "open".to_string())
                        .expect("fixture query parameter must encode"),
                    controls,
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
                .expect("one root may carry multiple bounded dependency records")
        })
        .expect("the fixture read owns its installed host call");

    assert_eq!(result.rows().len(), 1);
    assert_eq!(result.receipt().result_count(), 1);
    assert_eq!(result.receipt().projected_record_count(), 7);
    assert!(result.receipt().total_work_units() <= 10_000);
    assert_eq!(result.receipt().fallback_count(), 0);
}

#[test]
fn invented_selector_contract_denies_domain_projection() {
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
    let query = installed_forged_selector_query(&world);
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
                    ApplicationQueryParameterSet::new()
                        .bind(status_parameter(), "open".to_string())
                        .expect("fixture query parameter must encode"),
                    current_controls(&request),
                )
                .unwrap()
        })
        .expect("the fixture read owns its installed host call");

    let denial = world
        .application
        .with_host_advancement(|phase| {
            world
                .application
                .execute_application_query_one_shot(&phase, plan)
                .err()
                .expect("an invented selector contract must deny projection")
        })
        .expect("the fixture read owns its installed host call");

    assert_eq!(
        denial.kind(),
        WorthQueryApplicationOneShotDenialKind::Projection(
            WorthQueryApplicationProjectionDenialKind::FieldContractMismatch,
        )
    );
}

#[test]
fn variable_width_scalar_projects_without_a_caller_byte_estimate() {
    let world = installed_authorization_world_with_label(&"x".repeat(50_000), crate::domain_computation::execution_runtime::WorthQueryApplicationQueryResourceProfile::new(std::num::NonZeroUsize::new(64).expect("concurrent graph work is non-zero")));
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
    let query = installed_query(&world);
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
                    ApplicationQueryParameterSet::new()
                        .bind(status_parameter(), "open".to_string())
                        .expect("fixture query parameter must encode"),
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
                .expect("actual variable-width values need no caller byte forecast")
        })
        .expect("the fixture read owns its installed host call");

    assert_eq!(result.rows().len(), 1);
    assert_eq!(result.rows()[0].label(), "x".repeat(50_000));
}
