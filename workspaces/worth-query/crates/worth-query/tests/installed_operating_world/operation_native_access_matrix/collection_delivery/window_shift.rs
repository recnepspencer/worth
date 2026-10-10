use worth_query::facade::domain;

use super::super::collection_window::{bound_collection, first_window};
use super::super::fixture::{insert_matrix_value, matrix_workspace};
use super::super::samples::matrix_value_with_order;
use super::{managed_collection_lease, required_patch};

#[test]
fn removing_a_tail_anchor_emits_an_explicit_window_shift() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);
    let mut workspace = matrix_workspace(
        request_execution,
        "collection-delivery-window-shift",
        0,
        false,
    );
    for (row, order) in ["10", "20", "30", "40"].into_iter().enumerate() {
        insert_matrix_value(
            request_execution,
            &mut workspace,
            row,
            matrix_value_with_order(row as u64, order),
        );
    }
    let (collection, _) = bound_collection(&mut workspace);
    let first = first_window(&collection, 2);
    let cursor = match first.continuation() {
        domain::WorthQueryCollectionContinuation::LiveMore(cursor) => cursor.clone(),
        _ => panic!("tail fixture omitted its continuation"),
    };
    let tail = collection
        .resolve_window(
            collection
                .declare_window(
                    cursor,
                    domain::WorthQueryCollectionWindowBreadth::new(2, 0, 0, 2).unwrap(),
                )
                .unwrap(),
        )
        .unwrap();
    let removed_anchor = tail.rows()[0].entity_identity().clone();
    let mut consumer =
        domain::WorthQueryCollectionConsumerWindow::from_bound(collection, tail).unwrap();
    let lease = managed_collection_lease(request_execution, &mut workspace);
    assert!(lease.drain(&mut workspace).unwrap().delivery().is_empty());

    workspace
        .delete(request_execution, removed_anchor.clone())
        .unwrap();
    let delta = lease
        .consumer_invalidation_delta(lease.drain(&mut workspace).unwrap())
        .unwrap();
    let admitted = match lease.admit_consumer_invalidation_delta(delta, &workspace) {
        Ok(admitted) => admitted,
        Err(stop) => panic!("window-shift invalidation stopped: {:?}", stop.kind()),
    };
    consumer.bind_shared_target(&admitted, &workspace).unwrap();
    let patch = required_patch(request_execution, &mut consumer, &admitted, &workspace);

    assert!(patch.operations().iter().any(|operation| matches!(
        operation,
        domain::WorthQueryCollectionPatchOperation::Remove { entity, .. }
            if entity == &removed_anchor
    )));
    assert!(patch.operations().iter().any(|operation| matches!(
        operation,
        domain::WorthQueryCollectionPatchOperation::WindowShift { .. }
    )));
    consumer.apply_patch(patch).unwrap();
    assert_eq!(consumer.rows().len(), 1);
    assert_ne!(consumer.rows()[0].entity_identity(), &removed_anchor);
}
