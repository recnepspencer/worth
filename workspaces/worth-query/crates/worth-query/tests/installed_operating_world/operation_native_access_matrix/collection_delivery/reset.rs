use worth_query::facade::{domain, runtime};

use super::super::collection_window::{bound_collection, first_window};
use super::super::fixture::{insert_matrix_value, matrix_workspace_without_collection_lookup};
use super::super::samples::{matrix_aspect_key, matrix_value_with_order};
use super::managed_collection_lease;

#[test]
fn unsupported_point_lookup_yields_one_explicit_reset_then_blocks_patch_work() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut workspace = matrix_workspace_without_collection_lookup(
        request_execution,
        "collection-delivery-reset",
        &[],
    );
    let row = insert_matrix_value(
        request_execution,
        &mut workspace,
        0,
        matrix_value_with_order(0, "10"),
    );
    let (collection, _) = bound_collection(&mut workspace);
    let window = first_window(&collection, 1);
    let mut consumer =
        domain::WorthQueryCollectionConsumerWindow::from_bound(collection, window).unwrap();
    let lease = managed_collection_lease(request_execution, &mut workspace);
    assert!(lease.drain(&mut workspace).unwrap().delivery().is_empty());

    workspace
        .update(request_execution, row, |mutation| {
            mutation.set_aspect(
                runtime::WorthQueryAspectTouch::whole_aspect(matrix_aspect_key()),
                matrix_value_with_order(0, "20"),
            )
        })
        .unwrap();
    let delta = lease
        .consumer_invalidation_delta(lease.drain(&mut workspace).unwrap())
        .unwrap();
    let admitted = match lease.admit_consumer_invalidation_delta(delta, &workspace) {
        Ok(admitted) => admitted,
        Err(stop) => panic!("reset-path invalidation stopped: {:?}", stop.kind()),
    };
    consumer.bind_shared_target(&admitted, &workspace).unwrap();
    let expected_foundational = admitted.delta().foundational_projection();

    let patch = match consumer.plan_patch(request_execution, &admitted, &workspace) {
        domain::WorthQueryCollectionDeliveryOutcome::Patch(patch) => patch,
        domain::WorthQueryCollectionDeliveryOutcome::NoDelivery(denial) => {
            panic!("unsupported point lookup hid reset: {:?}", denial.kind())
        }
    };
    assert_unsupported_reset(&patch);
    assert_eq!(patch.foundational_invalidation(), &expected_foundational);
    let receipt = consumer.apply_patch(patch).unwrap();
    assert!(receipt.reset_required());
    assert!(consumer.reset_pending());

    let outcome = consumer.plan_patch(request_execution, &admitted, &workspace);
    let domain::WorthQueryCollectionDeliveryOutcome::NoDelivery(denial) = outcome else {
        panic!("reset-pending consumer continued incremental work")
    };
    assert_eq!(
        denial.kind(),
        domain::WorthQueryCollectionDeliveryDenialKind::ResetPending
    );
    assert_eq!(denial.counters().invalidation_authority_checks, 0);
    assert_eq!(denial.counters().entity_point_lookups, 0);
    assert_eq!(denial.counters().full_collection_scans, 0);
}

fn assert_unsupported_reset(patch: &domain::WorthQueryCollectionPatch) {
    assert!(matches!(
        patch.operations(),
        [domain::WorthQueryCollectionPatchOperation::ResetRequired {
            reason: domain::WorthQueryCollectionResetReason::UnsupportedIncrementalMeaning,
            cost: domain::WorthQueryCollectionResetCost {
                fresh_execution_required: true,
                maximum_replacement_rows: 1,
            },
        }]
    ));
}
