pub use worth_query::facade::{domain, foundation, runtime};

pub use super::collection_window::{bound_collection, first_window, settled_with_order_key};
pub use super::fixture::{insert_matrix_value, matrix_workspace, NativeMatrixRead};
pub use super::samples::{matrix_aspect_key, matrix_value_with_order};
pub use worth_query_testkit_installed_operation_fixture::facade::query::suite::installed_operation_fixture::{GeometryDomain, ReadFamily};

#[path = "collection_delivery/contract.rs"]
pub mod contract;
#[path = "collection_delivery/delivery_order.rs"]
pub mod delivery_order;
#[path = "collection_delivery/oracle.rs"]
pub mod oracle;
#[path = "collection_delivery/reset.rs"]
pub mod reset;
#[path = "collection_delivery/shared_window.rs"]
pub mod shared_window;
#[path = "collection_delivery/window_shift.rs"]
pub mod window_shift;
#[path = "collection_delivery/world.rs"]
pub mod world;

pub fn cursor_isolated_consumers(
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

pub fn bounded_patch_counters(
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

pub type CollectionLease = domain::WorthQuerySharedLiveProjectionLease<
    GeometryDomain,
    NativeMatrixRead,
    ReadFamily,
    foundation::ObservationLaneWitness,
>;

pub fn managed_collection_lease(
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

pub fn required_patch(
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
