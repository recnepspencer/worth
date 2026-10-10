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
    // One root contains seven projected records plus its funded field and
    // adjacency evidence. Declare this fixture's finite byte capacity separately
    // from the root count; the default per-root cap is not a shape guarantee.
    const RESULT_BYTES: usize = 16 * 1024;
    let world = super::super::fixture::installed_authorization_world_with_resource_profile(
        crate::domain_computation::execution_runtime::WorthQueryApplicationQueryResourceProfile::bounded(
            5120, RESULT_BYTES, 10_000, 64,
        ).unwrap(),
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
        NonZeroUsize::new(10_000).unwrap(),
        &request,
    );
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
            let host_custody = phase.test_host_request();
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
    let buffer = result.receipt().result_buffer().unwrap();
    assert_eq!(buffer.limit_bytes(), RESULT_BYTES);
    assert!(buffer.peak_bytes() <= RESULT_BYTES);
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
            let host_custody = phase.test_host_request();
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

    let buffer_observer = world.application.result_buffer_observer();
    let denial = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
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
    assert_eq!(buffer_observer.observe().active_buffers(), 0);
    assert_eq!(buffer_observer.observe().retained_bytes(), 0);
    assert!(buffer_observer.observe().peak_observed_bytes() > 0);
}

#[test]
fn variable_width_scalar_overflow_denies_and_releases_the_result_buffer() {
    let world = installed_authorization_world_with_label(&"x".repeat(50_000));
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
            let host_custody = phase.test_host_request();
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
    let result_buffer_limit = plan
        .graph_read_plan()
        .budget_check()
        .max_inline_result_bytes();
    let buffer_observer = world.application.result_buffer_observer();

    let denial = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            world
                .application
                .execute_application_query_one_shot(&phase, plan)
                .err()
                .expect("owned scalar memory beyond the admitted buffer must deny")
        })
        .expect("the fixture read owns its installed host call");

    assert_eq!(
        denial.kind(),
        WorthQueryApplicationOneShotDenialKind::ResultBufferLimitExceeded
    );
    let observation = buffer_observer.observe();
    assert_eq!(observation.active_buffers(), 0);
    assert_eq!(observation.retained_bytes(), 0);
    assert!(observation.peak_observed_bytes() > 0);
    assert!(observation.peak_observed_bytes() <= result_buffer_limit);
    assert!(observation.peak_rejected_bytes() > result_buffer_limit);
    assert!(observation.peak_rejected_bytes() >= 50_000);
}
