pub(crate) use worth_query_testkit_query::facade::query::suite::operation_native_access_matrix::collection_delivery::*;

#[test]
fn foreign_bound_window_cannot_construct_a_consumer() {
    let host_request = worth_query_testkit_installed_operation_fixture::facade::query::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut workspace = matrix_workspace(
        request_execution,
        "collection-consumer-foreign-window",
        3,
        false,
    );
    let (owner, _) = bound_collection(&mut workspace);
    let (foreign, _) = bound_collection(&mut workspace);
    let foreign_window = first_window(&foreign, 2);

    let denial = match domain::WorthQueryCollectionConsumerWindow::from_bound(owner, foreign_window)
    {
        Err(denial) => denial,
        Ok(_) => panic!("foreign bound window constructed a consumer"),
    };
    assert_eq!(
        denial.kind(),
        domain::WorthQueryCollectionDeliveryDenialKind::ForeignCollectionCapability
    );
    assert_eq!(denial.counters().generation_checks, 1);
    assert_eq!(denial.counters().semantic_contract_checks, 0);
    assert_eq!(denial.counters().full_collection_scans, 0);
}

#[test]
fn admitted_patch_converges_and_rejects_foreign_duplicate_and_empty_delivery() {
    let host_request = worth_query_testkit_installed_operation_fixture::facade::query::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut world = world::CollectionDeliveryWorld::new(request_execution);
    world.move_row_and_reject_reuse(request_execution);
    world.ignore_outside_update(request_execution);
    world.insert_then_remove_window_row(request_execution);
    world.remove_tail_and_complete_continuation(request_execution);
}

#[test]
fn patch_for_a_different_cursor_cannot_touch_the_consumer_window() {
    let host_request = worth_query_testkit_installed_operation_fixture::facade::query::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut workspace = matrix_workspace(
        request_execution,
        "collection-patch-cursor-isolation",
        0,
        false,
    );
    let (changed, mut beginning_consumer, mut tail_consumer) =
        cursor_isolated_consumers(request_execution, &mut workspace);
    let lease = managed_collection_lease(request_execution, &mut workspace);
    assert!(lease.drain(&mut workspace).unwrap().delivery().is_empty());

    workspace
        .update(request_execution, changed, |mutation| {
            mutation.set_aspect(
                runtime::WorthQueryAspectTouch::whole_aspect(matrix_aspect_key()),
                matrix_value_with_order(0, "35"),
            )
        })
        .unwrap();
    let delta = lease
        .consumer_invalidation_delta(lease.drain(&mut workspace).unwrap())
        .unwrap();
    let admitted = match lease.admit_consumer_invalidation_delta(delta, &workspace) {
        Ok(admitted) => admitted,
        Err(stop) => panic!(
            "cursor-isolation invalidation did not readmit: {:?}",
            stop.kind()
        ),
    };
    beginning_consumer
        .bind_shared_target(&admitted, &workspace)
        .unwrap();
    tail_consumer
        .bind_shared_target(&admitted, &workspace)
        .unwrap();

    let tail_patch = required_patch(request_execution, &mut tail_consumer, &admitted, &workspace);
    let denial = match beginning_consumer.apply_patch(tail_patch) {
        Err(denial) => denial,
        Ok(_) => panic!("patch for a different cursor touched the beginning window"),
    };
    assert_eq!(
        denial.kind(),
        domain::WorthQueryCollectionDeliveryDenialKind::CursorMismatch
    );
    assert_eq!(denial.counters().cursor_checks, 1);
}

#[test]
fn patch_for_a_different_admitted_breadth_cannot_touch_the_consumer_window() {
    let host_request = worth_query_testkit_installed_operation_fixture::facade::query::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut workspace = matrix_workspace(
        request_execution,
        "collection-patch-window-isolation",
        0,
        false,
    );
    let changed = insert_matrix_value(
        request_execution,
        &mut workspace,
        0,
        matrix_value_with_order(0, "10"),
    );
    insert_matrix_value(
        request_execution,
        &mut workspace,
        1,
        matrix_value_with_order(1, "20"),
    );
    insert_matrix_value(
        request_execution,
        &mut workspace,
        2,
        matrix_value_with_order(2, "30"),
    );
    let (narrow_collection, _) = bound_collection(&mut workspace);
    let narrow_window = first_window(&narrow_collection, 1);
    let (wide_collection, _) = bound_collection(&mut workspace);
    let wide_window = first_window(&wide_collection, 2);
    let mut narrow_consumer =
        domain::WorthQueryCollectionConsumerWindow::from_bound(narrow_collection, narrow_window)
            .unwrap();
    let mut wide_consumer =
        domain::WorthQueryCollectionConsumerWindow::from_bound(wide_collection, wide_window)
            .unwrap();
    let lease = managed_collection_lease(request_execution, &mut workspace);
    assert!(lease.drain(&mut workspace).unwrap().delivery().is_empty());

    workspace
        .update(request_execution, changed, |mutation| {
            mutation.set_aspect(
                runtime::WorthQueryAspectTouch::whole_aspect(matrix_aspect_key()),
                matrix_value_with_order(0, "15"),
            )
        })
        .unwrap();
    let delta = lease
        .consumer_invalidation_delta(lease.drain(&mut workspace).unwrap())
        .unwrap();
    let admitted = match lease.admit_consumer_invalidation_delta(delta, &workspace) {
        Ok(admitted) => admitted,
        Err(stop) => panic!(
            "window-isolation invalidation did not readmit: {:?}",
            stop.kind()
        ),
    };
    narrow_consumer
        .bind_shared_target(&admitted, &workspace)
        .unwrap();
    wide_consumer
        .bind_shared_target(&admitted, &workspace)
        .unwrap();

    let wide_patch = required_patch(request_execution, &mut wide_consumer, &admitted, &workspace);
    let denial = match narrow_consumer.apply_patch(wide_patch) {
        Err(denial) => denial,
        Ok(_) => panic!("patch for a wider admitted window touched the narrow window"),
    };
    assert_eq!(
        denial.kind(),
        domain::WorthQueryCollectionDeliveryDenialKind::WindowContractMismatch
    );
    assert_eq!(denial.counters().semantic_contract_checks, 1);
    assert_eq!(narrow_consumer.rows().len(), 1);
}

pub(crate) mod contract;

pub(crate) mod delivery_order;

pub(crate) mod reset;

pub(crate) mod shared_window;

pub(crate) mod window_shift;
