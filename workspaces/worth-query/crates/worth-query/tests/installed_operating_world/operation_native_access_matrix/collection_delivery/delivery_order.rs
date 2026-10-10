use worth_query::facade::{domain, runtime};

use super::super::collection_window::{bound_collection, first_window};
use super::super::fixture::{insert_matrix_value, matrix_workspace};
use super::super::samples::{matrix_aspect_key, matrix_value_with_order};
use super::{managed_collection_lease, required_patch};

#[test]
fn a_new_epoch_after_an_unapplied_patch_requires_reset() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut workspace = matrix_workspace(request_execution, "collection-unapplied-patch", 0, false);
    let first = insert_matrix_value(
        request_execution,
        &mut workspace,
        0,
        matrix_value_with_order(0, "10"),
    );
    let second = insert_matrix_value(
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
    let (collection, _) = bound_collection(&mut workspace);
    let window = first_window(&collection, 3);
    let baseline = window
        .rows()
        .iter()
        .map(|row| row.entity_identity().clone())
        .collect::<Vec<_>>();
    let mut consumer =
        domain::WorthQueryCollectionConsumerWindow::from_bound(collection, window).unwrap();
    let lease = managed_collection_lease(request_execution, &mut workspace);
    assert!(lease.drain(&mut workspace).unwrap().delivery().is_empty());

    update_order(request_execution, &mut workspace, first, "40");
    let first_delta = lease
        .consumer_invalidation_delta(lease.drain(&mut workspace).unwrap())
        .unwrap();
    let first_admitted = lease
        .admit_consumer_invalidation_delta(first_delta, &workspace)
        .unwrap_or_else(|stop| panic!("first invalidation stopped: {:?}", stop.kind()));
    consumer
        .bind_shared_target(&first_admitted, &workspace)
        .unwrap();
    let unapplied = required_patch(
        request_execution,
        &mut consumer,
        &first_admitted,
        &workspace,
    );
    drop(first_admitted);

    update_order(request_execution, &mut workspace, second, "05");
    let second_delta = lease
        .consumer_invalidation_delta(lease.drain(&mut workspace).unwrap())
        .unwrap();
    let second_admitted = lease
        .admit_consumer_invalidation_delta(second_delta, &workspace)
        .unwrap_or_else(|stop| panic!("second invalidation stopped: {:?}", stop.kind()));
    let reset = required_patch(
        request_execution,
        &mut consumer,
        &second_admitted,
        &workspace,
    );
    assert!(matches!(
        reset.operations(),
        [domain::WorthQueryCollectionPatchOperation::ResetRequired {
            reason: domain::WorthQueryCollectionResetReason::UnappliedPriorPatch,
            cost: domain::WorthQueryCollectionResetCost {
                fresh_execution_required: true,
                maximum_replacement_rows: 3
            }
        }]
    ));
    let denial = match consumer.apply_patch(unapplied) {
        Err(denial) => denial,
        Ok(_) => panic!("superseded patch mutated the consumer"),
    };
    assert_eq!(
        denial.kind(),
        domain::WorthQueryCollectionDeliveryDenialKind::SupersededPatch
    );
    assert_eq!(denial.counters().pending_patch_checks, 1);
    let receipt = consumer.apply_patch(reset).unwrap();
    assert!(receipt.reset_required());
    assert!(consumer.reset_pending());
    assert_eq!(
        consumer
            .rows()
            .iter()
            .map(|row| row.entity_identity().clone())
            .collect::<Vec<_>>(),
        baseline
    );
}

fn update_order(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    workspace: &mut runtime::WorthQueryWorkspace,
    entity: worth_query::facade::foundation::WorthQueryEntityIdentity,
    order: &str,
) {
    workspace
        .update(request_execution, entity, |mutation| {
            mutation.set_aspect(
                runtime::WorthQueryAspectTouch::whole_aspect(matrix_aspect_key()),
                matrix_value_with_order(0, order),
            )
        })
        .unwrap();
}
