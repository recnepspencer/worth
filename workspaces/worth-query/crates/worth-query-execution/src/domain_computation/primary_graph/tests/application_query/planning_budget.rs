use std::num::NonZeroUsize;
use std::time::Duration;

use worth_query_admission::facade::graph_read_access::WorthQueryGraphReadPlanReviewDenialKind;
use worth_query_declaration::facade::application_query::ApplicationQueryParameterSet;

use super::installed_nested_query;
use crate::domain_computation::execution_runtime::WorthQueryApplicationQueryResourceProfile;
use crate::domain_computation::primary_graph::tests::fixture::{
    installed_authorization_world, installed_authorization_world_with_resource_profile, live_scope,
    status_parameter, AccountStatus,
};
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationQueryAccessContext, WorthQueryApplicationQueryAdmissionDenialKind,
    WorthQueryPrincipalResolutionMode,
};

#[test]
fn nested_query_total_work_exhaustion_returns_no_plan_authority() {
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
    let query = installed_nested_query(&world);
    let access = WorthQueryApplicationQueryAccessContext::new(&principal, &account);
    let controls = crate::domain_computation::primary_graph::WorthQueryProductQueryControls::new(
        NonZeroUsize::new(10).unwrap(),
        Some(NonZeroUsize::new(1).unwrap()),
        &request,
    );

    let denial = world
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
                .err()
                .expect("exhausted total work must not mint execution authority")
        })
        .expect("the fixture read owns its installed host call");

    assert_eq!(
        denial.kind(),
        WorthQueryApplicationQueryAdmissionDenialKind::WorkLimitExceeded
    );
}

#[test]
fn caller_work_cannot_widen_the_installed_index_profile() {
    let resources =
        WorthQueryApplicationQueryResourceProfile::bounded(1, 4_096, 100_000, 64).unwrap();
    let world = installed_authorization_world_with_resource_profile(resources);
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
        NonZeroUsize::new(10).unwrap(),
        Some(NonZeroUsize::new(100_000).unwrap()),
        &request,
    );

    let denial = world
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
                .err()
                .expect("caller work cannot widen installed index capacity")
        })
        .expect("the fixture read owns its installed host call");

    assert_eq!(
        denial.kind(),
        WorthQueryApplicationQueryAdmissionDenialKind::GraphReadPlan(
            WorthQueryGraphReadPlanReviewDenialKind::BudgetExceeded
        )
    );
}

#[test]
fn installer_profile_changes_admission_without_changing_query_identity() {
    let default_world = installed_authorization_world(true);
    let resources =
        WorthQueryApplicationQueryResourceProfile::bounded(32_768, 4_096, 32_768, 64).unwrap();
    let world = installed_authorization_world_with_resource_profile(resources);
    assert_eq!(
        installed_nested_query(&default_world).identity(),
        installed_nested_query(&world).identity()
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
        NonZeroUsize::new(10).unwrap(),
        Some(NonZeroUsize::new(20_000).unwrap()),
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
                .expect("the installer-owned profile should admit the exact plan")
        })
        .expect("the fixture read owns its installed host call");

    assert_eq!(
        plan.graph_read_plan()
            .budget_check()
            .max_inline_index_bytes(),
        Some(32_768)
    );
    assert_eq!(
        plan.graph_read_plan()
            .budget_check()
            .max_inline_result_bytes(),
        Some(40_960)
    );
    assert_eq!(
        plan.graph_read_plan()
            .budget_check()
            .max_inline_intermediate_set_size(),
        Some(20_000)
    );
}
