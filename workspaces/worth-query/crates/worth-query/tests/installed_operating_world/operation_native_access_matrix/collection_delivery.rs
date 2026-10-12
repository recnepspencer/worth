use worth_query::facade::{domain, foundation, runtime};

use super::collection_window::{bound_collection, first_window, settled_with_order_key};
use super::fixture::{insert_matrix_value, matrix_workspace, NativeMatrixRead};
use super::samples::{matrix_aspect_key, matrix_value_with_order};
use crate::suite::installed_operation_fixture::{GeometryDomain, ReadFamily};

#[path = "collection_delivery/contract.rs"]
mod contract;
#[path = "collection_delivery/delivery_order.rs"]
mod delivery_order;
#[path = "collection_delivery/oracle.rs"]
mod oracle;
#[path = "collection_delivery/reset.rs"]
mod reset;
#[path = "collection_delivery/shared_window.rs"]
mod shared_window;
#[path = "collection_delivery/window_shift.rs"]
mod window_shift;
#[path = "collection_delivery/world.rs"]
mod world;

#[test]
fn foreign_bound_window_cannot_construct_a_consumer() {
    let host_request = crate::test_execution_policy::serial_request();
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
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut world = world::CollectionDeliveryWorld::new(request_execution);
    world.move_row_and_reject_reuse(request_execution);
    world.ignore_outside_update(request_execution);
    world.insert_then_remove_window_row(request_execution);
    world.remove_tail_and_complete_continuation(request_execution);
}

#[test]
fn patch_cost_is_independent_of_total_collection_width() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let small = bounded_patch_counters(request_execution, "collection-patch-scale-small", 4);
    let large = bounded_patch_counters(request_execution, "collection-patch-scale-large", 512);

    assert_eq!(small, large);
    assert_eq!(large.entity_point_lookups, 1);
    assert_eq!(large.ordering_index_updates, 1);
    assert_eq!(large.fresh_window_rows_visited, 7);
    assert_eq!(large.full_collection_scans, 0);
    assert_eq!(large.unrelated_consumer_scans, 0);
}

#[test]
fn patch_for_a_different_cursor_cannot_touch_the_consumer_window() {
    let host_request = crate::test_execution_policy::serial_request();
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
    let host_request = crate::test_execution_policy::serial_request();
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

fn cursor_isolated_consumers(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    workspace: &mut runtime::WorthQueryWorkspace,
) -> (
    foundation::WorthQueryEntityIdentity,
    domain::WorthQueryCollectionConsumerWindow,
    domain::WorthQueryCollectionConsumerWindow,
) {
    let changed = insert_matrix_value(
        request_execution,
        workspace,
        0,
        matrix_value_with_order(0, "10"),
    );
    for (row, order) in ["20", "30", "40"].into_iter().enumerate() {
        insert_matrix_value(
            request_execution,
            workspace,
            row + 1,
            matrix_value_with_order((row + 1) as u64, order),
        );
    }
    let (beginning_collection, _) = bound_collection(workspace);
    let beginning = first_window(&beginning_collection, 2);
    let (tail_collection, _) = bound_collection(workspace);
    let tail_beginning = first_window(&tail_collection, 2);
    let cursor = match tail_beginning.continuation() {
        domain::WorthQueryCollectionContinuation::LiveMore(cursor) => cursor.clone(),
        _ => panic!("cursor-isolation fixture omitted its second window"),
    };
    let admission = tail_collection
        .declare_window(
            cursor,
            domain::WorthQueryCollectionWindowBreadth::new(2, 0, 0, 2).unwrap(),
        )
        .unwrap();
    let tail = tail_collection.resolve_window(admission).unwrap();
    (
        changed,
        domain::WorthQueryCollectionConsumerWindow::from_bound(beginning_collection, beginning)
            .unwrap(),
        domain::WorthQueryCollectionConsumerWindow::from_bound(tail_collection, tail).unwrap(),
    )
}

fn bounded_patch_counters(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    name: &str,
    row_count: usize,
) -> domain::WorthQueryCollectionDeliveryCounters {
    let mut workspace = matrix_workspace(request_execution, name, 0, false);
    let changed = insert_matrix_value(
        request_execution,
        &mut workspace,
        0,
        matrix_value_with_order(0, "0000"),
    );
    for row in 1..row_count {
        insert_matrix_value(
            request_execution,
            &mut workspace,
            row,
            matrix_value_with_order(row as u64, &format!("{:04}", row + 100)),
        );
    }
    let (baseline, _) = bound_collection(&mut workspace);
    let window = first_window(&baseline, 3);
    let mut consumer =
        domain::WorthQueryCollectionConsumerWindow::from_bound(baseline, window).unwrap();
    let lease = managed_collection_lease(request_execution, &mut workspace);
    assert!(lease.drain(&mut workspace).unwrap().delivery().is_empty());
    workspace
        .update(request_execution, changed, |mutation| {
            mutation.set_aspect(
                runtime::WorthQueryAspectTouch::whole_aspect(matrix_aspect_key()),
                matrix_value_with_order(0, "0001"),
            )
        })
        .unwrap();
    let delta = lease
        .consumer_invalidation_delta(lease.drain(&mut workspace).unwrap())
        .unwrap();
    let admitted = match lease.admit_consumer_invalidation_delta(delta, &workspace) {
        Ok(admitted) => admitted,
        Err(stop) => panic!("scale invalidation did not readmit: {:?}", stop.kind()),
    };
    consumer.bind_shared_target(&admitted, &workspace).unwrap();
    required_patch(request_execution, &mut consumer, &admitted, &workspace).counters()
}

type CollectionLease = domain::WorthQuerySharedLiveProjectionLease<
    GeometryDomain,
    NativeMatrixRead,
    ReadFamily,
    foundation::ObservationLaneWitness,
>;

fn managed_collection_lease(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    workspace: &mut runtime::WorthQueryWorkspace,
) -> CollectionLease {
    let (settled, _) = settled_with_order_key(workspace);
    let live = match settled
        .into_lifecycle()
        .promote(request_execution, workspace)
    {
        domain::WorthQueryProjectionPromotionOutcome::Promoted(live) => live,
        _ => panic!("collection projection did not promote"),
    };
    match live.into_managed_lease(workspace) {
        domain::WorthQueryProjectionLeaseAdmissionOutcome::Admitted(lease) => lease,
        domain::WorthQueryProjectionLeaseAdmissionOutcome::Stopped(stop) => {
            panic!("collection lease admission stopped: {}", stop.detail())
        }
    }
}

fn required_patch(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    consumer: &mut domain::WorthQueryCollectionConsumerWindow,
    admitted: &domain::WorthQueryAdmittedConsumerInvalidation<'_>,
    workspace: &runtime::WorthQueryWorkspace,
) -> domain::WorthQueryCollectionPatch {
    match consumer.plan_patch(request_execution, admitted, workspace) {
        domain::WorthQueryCollectionDeliveryOutcome::Patch(patch) => patch,
        domain::WorthQueryCollectionDeliveryOutcome::NoDelivery(denial) => {
            panic!(
                "semantic collection change did not deliver: {:?}",
                denial.kind()
            )
        }
    }
}
