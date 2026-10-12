use worth_query::facade::{domain, runtime};

use super::super::collection_window::{first_window, settled_with_native_field};
use super::super::fixture::{insert_matrix_value, matrix_workspace};
use super::super::samples::{matrix_aspect_key, matrix_value_with_order};
use super::managed_collection_lease;

#[test]
fn a_lease_with_a_different_native_layout_cannot_target_the_consumer() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut workspace = matrix_workspace(
        request_execution,
        "collection-delivery-contract-affinity",
        0,
        false,
    );
    let changed = insert_matrix_value(
        request_execution,
        &mut workspace,
        0,
        matrix_value_with_order(0, "10"),
    );
    let (settled, _) = settled_with_native_field(&mut workspace, 14);
    let collection = settled.into_bound_collection().unwrap();
    let window = first_window(&collection, 1);
    let mut consumer =
        domain::WorthQueryCollectionConsumerWindow::from_bound(collection, window).unwrap();
    let lease = managed_collection_lease(request_execution, &mut workspace);
    assert!(lease.drain(&mut workspace).unwrap().delivery().is_empty());

    workspace
        .update(request_execution, changed, |mutation| {
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
        Err(stop) => panic!("contract-affinity invalidation stopped: {:?}", stop.kind()),
    };
    let denial = consumer
        .bind_shared_target(&admitted, &workspace)
        .expect_err("a different selected native layout joined collection delivery");

    assert_eq!(
        denial.kind(),
        domain::WorthQueryCollectionDeliveryDenialKind::CollectionContractMismatch
    );
    assert_eq!(denial.counters().entity_point_lookups, 0);
    assert_eq!(denial.counters().ordering_index_updates, 0);
    assert_eq!(denial.counters().full_collection_scans, 0);
}
