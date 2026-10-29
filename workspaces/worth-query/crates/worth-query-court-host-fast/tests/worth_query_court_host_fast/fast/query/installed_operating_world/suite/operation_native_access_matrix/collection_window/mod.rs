pub(crate) use worth_query_worlds::facade::query::suite::operation_native_access_matrix::collection_window::*;

#[test]
fn bound_window_preserves_canonical_identity_order_and_cursor_meaning() {
    let host_request = worth_query_worlds::facade::query::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut workspace = matrix_workspace(request_execution, "collection-window-identity", 0, false);
    let identities = [
        insert_matrix_value(
            request_execution,
            &mut workspace,
            0,
            matrix_value_with_order(0, "30"),
        ),
        insert_matrix_value(
            request_execution,
            &mut workspace,
            1,
            matrix_value_with_order(1, "10"),
        ),
        insert_matrix_value(
            request_execution,
            &mut workspace,
            2,
            matrix_value_with_order(2, "20"),
        ),
    ];
    let (collection, order_key) = bound_collection(&mut workspace);
    let window = first_window(&collection, 2);

    assert_eq!(order_values(&collection, &window, &order_key), ["10", "20"]);
    assert_eq!(
        window
            .rows()
            .iter()
            .map(|row| row.entity_identity())
            .collect::<Vec<_>>(),
        vec![&identities[1], &identities[2]]
    );
    assert!(matches!(
        window.continuation(),
        domain::WorthQueryCollectionContinuation::LiveMore(_)
    ));
    assert_eq!(window.counters().rows_visited, 2);
    assert_eq!(window.counters().unrelated_rows_scanned, 0);
    assert_eq!(collection.counters().maintenance_rows_indexed, 3);

    let next = match window.continuation() {
        domain::WorthQueryCollectionContinuation::LiveMore(cursor) => cursor.clone(),
        _ => panic!("bounded live collection omitted its continuation"),
    };
    let admitted = collection.declare_window(next, breadth(2)).unwrap();
    let tail = collection.resolve_window(admitted).unwrap();
    assert_eq!(tail.rows()[0].entity_identity(), &identities[0]);
    assert_eq!(
        tail.continuation(),
        &domain::WorthQueryCollectionContinuation::Complete
    );
}

#[test]
fn ordering_change_moves_the_same_entity_and_view_identity() {
    let host_request = worth_query_worlds::facade::query::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut workspace = matrix_workspace(request_execution, "collection-window-reorder", 0, false);
    let alpha = insert_matrix_value(
        request_execution,
        &mut workspace,
        0,
        matrix_value_with_order(0, "20"),
    );
    let moved = insert_matrix_value(
        request_execution,
        &mut workspace,
        1,
        matrix_value_with_order(1, "30"),
    );
    let _beta = insert_matrix_value(
        request_execution,
        &mut workspace,
        2,
        matrix_value_with_order(2, "40"),
    );
    let (before, before_order_key) = bound_collection(&mut workspace);
    let before_window = first_window(&before, 3);
    assert_eq!(
        order_values(&before, &before_window, &before_order_key),
        ["20", "30", "40"]
    );
    let before_handle = before_window
        .rows()
        .iter()
        .find(|row| row.entity_identity() == &moved)
        .unwrap()
        .clone();

    workspace
        .update(request_execution, moved.clone(), |mutation| {
            mutation.set_aspect(
                runtime::WorthQueryAspectTouch::whole_aspect(matrix_aspect_key()),
                matrix_value_with_order(1, "10"),
            )
        })
        .unwrap();
    let (after, after_order_key) = bound_collection(&mut workspace);
    let after_window = first_window(&after, 3);
    assert_eq!(
        order_values(&after, &after_window, &after_order_key),
        ["10", "20", "40"]
    );
    let after_handle = &after_window.rows()[0];

    assert_eq!(after_handle.entity_identity(), &moved);
    assert_eq!(
        after_handle.entity_identity(),
        before_handle.entity_identity()
    );
    assert_eq!(
        after_handle.view_local_identity(),
        before_handle.view_local_identity()
    );
    assert_eq!(after_window.rows()[1].entity_identity(), &alpha);
}

#[test]
fn foreign_cursor_denies_before_any_row_or_index_work() {
    let host_request = worth_query_worlds::facade::query::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut workspace = matrix_workspace(request_execution, "collection-window-foreign", 3, false);
    let (owner, _) = bound_collection(&mut workspace);
    let (foreign, _) = bound_collection(&mut workspace);

    let denial = match owner.declare_window(foreign.beginning_cursor(), breadth(2)) {
        TransitionOutcome::Denied(denial) => denial,
        _ => panic!("foreign capability cursor was admitted"),
    };

    assert_eq!(
        denial.kind(),
        domain::WorthQueryCollectionWindowDenialKind::ForeignCapability
    );
    assert_eq!(denial.counters().ordered_index_probes, 0);
    assert_eq!(denial.counters().rows_visited, 0);
    assert_eq!(denial.counters().window_rows_materialized, 0);
}

#[test]
fn window_resolution_cost_is_independent_of_total_collection_width() {
    let host_request = worth_query_worlds::facade::query::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut small = matrix_workspace(request_execution, "collection-window-scale-small", 4, false);
    let mut large = matrix_workspace(
        request_execution,
        "collection-window-scale-large",
        512,
        true,
    );
    let (small_collection, _) = bound_collection(&mut small);
    let (large_collection, _) = bound_collection(&mut large);
    let small_window = first_window(&small_collection, 3);
    let large_window = first_window(&large_collection, 3);

    assert_eq!(small_window.counters(), large_window.counters());
    assert_eq!(large_window.counters().rows_visited, 3);
    assert_eq!(large_window.counters().ordered_index_probes, 1);
    assert_eq!(large_window.counters().unrelated_rows_scanned, 0);
}

#[test]
fn grouped_collection_requires_native_access_instead_of_degrading_to_an_offset_window() {
    let host_request = worth_query_worlds::facade::query::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    use worth_query_worlds::facade::query::suite::installed_operation_fixture::collection_impact::{
        impact_collection_workspace, ImpactCollectionRead,
    };
    use worth_query_worlds::facade::query::suite::installed_operation_fixture::{GeometryDomain, ReadFamily};

    let mut workspace =
        impact_collection_workspace(request_execution, "collection-window-grouping-denial")
            .unwrap();
    let installed = workspace.domain(GeometryDomain).unwrap();
    let bound = workspace
        .observe_operating_world(workspace.current_world())
        .unwrap()
        .family(ReadFamily)
        .bind(&installed, ImpactCollectionRead)
        .unwrap();
    let consumer = bound.consumer_projection_contract().unwrap();
    let settled = bound
        .admit_execution_resources(
            (),
            worth_query_worlds::facade::query::suite::installed_operation_fixture::execution_resource_request(),
            &workspace,
        )
        .unwrap()
        .execute(&mut workspace)
        .unwrap()
        .publish()
        .unwrap()
        .consume(
            consumer,
            read::project_facts()
                .entity_identities()
                .view_local_identities(),
        )
        .unwrap()
        .settle()
        .unwrap();
    let stop = match settled.into_bound_collection() {
        TransitionOutcome::Denied(stop) => stop,
        _ => panic!("grouped collection without native access degraded into an offset window"),
    };
    assert_eq!(
        stop.denial().kind(),
        domain::WorthQueryCollectionCapabilityDenialKind::NativeAccessNotBound
    );
    assert_eq!(stop.denial().counters().native_layout_checks, 1);
    assert_eq!(stop.denial().counters().identity_rows_indexed, 0);
    assert!(!stop.into_projection().identity().is_empty());
}
