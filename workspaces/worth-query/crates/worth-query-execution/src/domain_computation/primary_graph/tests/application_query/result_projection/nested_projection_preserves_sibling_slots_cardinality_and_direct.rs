//! nested projection preserves sibling slots cardinality and direction.
use super::*;

#[test]
fn nested_projection_preserves_sibling_slots_cardinality_and_direction() {
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
    let admitted_session = plan.graph_work_session_identity();
    let admitted_basis = plan.basis_identity().clone();

    let buffer_observer = world.application.result_buffer_observer();
    let outcome = world
        .application
        .with_host_advancement(|phase| {
            world
                .application
                .execute_application_query_one_shot(&phase, plan)
        })
        .expect("the fixture read owns its installed host call");
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
    let (primary, secondary, all, reverse) = result.rows()[0].activity_identities();
    assert_eq!(secondary, Some(all[1]));
    assert_eq!(primary, all[0]);
    assert_eq!(reverse, all);
    assert_ne!(all[0], all[1]);
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
    let buffer = world
        .application
        .with_host_advancement(|_phase| {
            result
                .receipt()
                .result_buffer()
                .expect("one-shot execution owns bounded result-buffer evidence")
        })
        .expect("the fixture read owns its installed host call");
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
    assert!(
        buffer_observer.observe().retained_bytes() > 0,
        "the returned observed sources keep their funded custody alive"
    );
    assert_eq!(
        buffer_observer.observe().peak_observed_bytes(),
        buffer.peak_bytes()
    );
    drop(result);
    assert_eq!(buffer_observer.observe().retained_bytes(), 0);
}
