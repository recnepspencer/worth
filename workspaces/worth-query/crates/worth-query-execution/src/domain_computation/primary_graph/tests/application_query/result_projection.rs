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
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationOneShotDenialKind, WorthQueryApplicationProjectionDenialKind,
    WorthQueryApplicationQueryAccessContext, WorthQueryApplicationQueryOmissionPosture,
    WorthQueryPrincipalResolutionMode,
};

#[test]
fn nested_projection_preserves_sibling_slots_cardinality_and_direction() {
    let world = installed_authorization_world(true);
    let request = live_scope();
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
    let query = installed_nested_query(&world);
    let access = WorthQueryApplicationQueryAccessContext::new(&principal, &account);
    let plan = world
        .selected_product()
        .admit_application_query(
            &query,
            &access,
            ApplicationQueryParameterSet::new()
                .bind(status_parameter(), "open".to_string())
                .expect("fixture query parameter must encode"),
            current_controls(&request),
        )
        .unwrap();
    let admitted_session = plan.graph_work_session_identity();
    let admitted_basis = plan.basis_identity().clone();

    let buffer_observer = world.application.result_buffer_observer();
    let outcome = world.application.execute_application_query_one_shot(plan);
    assert!(
        outcome.is_ok(),
        "nested projection outcome: {:?}; buffer observation: {:?}",
        outcome.err(),
        buffer_observer.observe()
    );
    let result = outcome.unwrap();

    assert_eq!(result.rows().len(), 1);
    assert_eq!(result.rows()[0].primary_sequence(), 11);
    assert_eq!(result.rows()[0].secondary_sequence(), Some(22));
    assert_eq!(result.rows()[0].all_sequences(), &[11, 22]);
    assert_eq!(result.rows()[0].reverse_sequences(), &[11, 22]);
    assert_eq!(result.receipt().projected_record_count(), 7);
    assert_eq!(result.receipt().projected_field_count(), 6);
    assert_eq!(result.receipt().adjacency_list_read_count(), 4);
    assert_eq!(result.receipt().edge_scan_count(), 6);
    assert!(result.receipt().ordering_comparison_count() > 0);
    assert_eq!(result.receipt().per_result_neighbor_lookup_count(), 0);
    assert_eq!(result.receipt().fallback_count(), 0);
    assert_eq!(result.receipt().result_count(), 1);
    assert_eq!(result.receipt().truncation_count(), 0);
    assert_eq!(
        result.receipt().omission_posture(),
        WorthQueryApplicationQueryOmissionPosture::NoOmission
    );
    assert!(result.receipt().total_work_units() <= 10_000);
    let buffer = result
        .receipt()
        .result_buffer()
        .expect("one-shot execution owns bounded result-buffer evidence");
    assert!(buffer.released());
    assert!(buffer.peak_bytes() > 0);
    assert!(buffer.peak_bytes() <= buffer.limit_bytes());
    assert!(result
        .receipt()
        .graph_read_plan()
        .requirements()
        .requires_kind(WorthQueryGraphReadAccessRequirementKind::ReverseAdjacency));
    assert!(result
        .receipt()
        .graph_read_plan()
        .requirements()
        .requires_kind(WorthQueryGraphReadAccessRequirementKind::OrderingSupport));
    let completion = result.receipt().read_completion();
    assert_eq!(completion.session_identity(), admitted_session);
    assert_eq!(completion.basis_identity(), &admitted_basis);
    assert!(completion.basis_release().released());
    assert_eq!(completion.basis_release().identity(), &admitted_basis);
    assert_eq!(completion.release().released_reservation_count(), 1);
    assert_eq!(
        completion.release().scope(),
        worth_query_admission::integration::WorthQueryExecutionCapacityReservationScope::GraphWork
    );
    let dependencies = completion.dependencies();
    assert!(dependencies.includes_predicate_and_negative_space());
    assert!(dependencies.includes_ordering());
    assert!(dependencies.includes_membership_and_traversal());
    assert!(dependencies.includes_projection());
    assert_eq!(
        dependencies.examined_candidates(),
        result.receipt().examined_candidate_count()
    );
    assert_eq!(
        dependencies.projected_records(),
        result.receipt().projected_record_count()
    );
    assert_eq!(
        dependencies.projected_fields(),
        result.receipt().projected_field_count()
    );
    assert_eq!(buffer_observer.observe().active_buffers(), 0);
    assert_eq!(buffer_observer.observe().retained_bytes(), 0);
    assert_eq!(
        buffer_observer.observe().peak_observed_bytes(),
        buffer.peak_bytes()
    );
}

#[test]
fn root_result_limit_does_not_cap_nested_dependency_records() {
    let world = installed_authorization_world(true);
    let request = live_scope();
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
    let query = installed_nested_query(&world);
    let access = WorthQueryApplicationQueryAccessContext::new(&principal, &account);
    let controls = crate::domain_computation::primary_graph::WorthQueryProductQueryControls::new(
        NonZeroUsize::new(1).unwrap(),
        NonZeroUsize::new(10_000).unwrap(),
        &request,
    );
    let plan = world
        .selected_product()
        .admit_application_query(
            &query,
            &access,
            ApplicationQueryParameterSet::new()
                .bind(status_parameter(), "open".to_string())
                .expect("fixture query parameter must encode"),
            controls,
        )
        .unwrap();

    let result = world
        .application
        .execute_application_query_one_shot(plan)
        .expect("one root may carry multiple bounded dependency records");

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
    let query = installed_forged_selector_query(&world);
    let access = WorthQueryApplicationQueryAccessContext::new(&principal, &account);
    let plan = world
        .selected_product()
        .admit_application_query(
            &query,
            &access,
            ApplicationQueryParameterSet::new()
                .bind(status_parameter(), "open".to_string())
                .expect("fixture query parameter must encode"),
            current_controls(&request),
        )
        .unwrap();

    let buffer_observer = world.application.result_buffer_observer();
    let denial = world
        .application
        .execute_application_query_one_shot(plan)
        .err()
        .expect("an invented selector contract must deny projection");

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
    let world = installed_authorization_world_with_label(&"x".repeat(25_000));
    let request = live_scope();
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
    let query = installed_query(&world);
    let access = WorthQueryApplicationQueryAccessContext::new(&principal, &account);
    let plan = world
        .selected_product()
        .admit_application_query(
            &query,
            &access,
            ApplicationQueryParameterSet::new()
                .bind(status_parameter(), "open".to_string())
                .expect("fixture query parameter must encode"),
            current_controls(&request),
        )
        .unwrap();
    let result_buffer_limit = plan
        .graph_read_plan()
        .budget_check()
        .max_inline_result_bytes();
    let buffer_observer = world.application.result_buffer_observer();

    let denial = world
        .application
        .execute_application_query_one_shot(plan)
        .err()
        .expect("owned scalar memory beyond the admitted buffer must deny");

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
    assert!(observation.peak_rejected_bytes() >= 25_000);
}
