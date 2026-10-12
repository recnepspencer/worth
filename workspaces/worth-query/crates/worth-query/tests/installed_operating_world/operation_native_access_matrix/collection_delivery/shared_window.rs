use worth_query::facade::{domain, runtime};

use super::super::collection_window::{bound_collection, first_window, settled_with_order_key};
use super::super::fixture::{insert_matrix_value, matrix_workspace};
use super::super::samples::{matrix_aspect_key, matrix_value_with_order};
use super::required_patch;

#[test]
fn one_shared_epoch_yields_two_lease_specific_collection_patches() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut workspace = matrix_workspace(request_execution, "collection-shared-window", 0, false);
    let changed = insert_matrix_value(
        request_execution,
        &mut workspace,
        0,
        matrix_value_with_order(0, "30"),
    );
    insert_matrix_value(
        request_execution,
        &mut workspace,
        1,
        matrix_value_with_order(1, "10"),
    );
    insert_matrix_value(
        request_execution,
        &mut workspace,
        2,
        matrix_value_with_order(2, "20"),
    );

    let (subject_collection, _) = bound_collection(&mut workspace);
    let subject_window = first_window(&subject_collection, 3);
    let mut subject_consumer =
        domain::WorthQueryCollectionConsumerWindow::from_bound(subject_collection, subject_window)
            .unwrap();
    let (candidate_collection, _) = bound_collection(&mut workspace);
    let candidate_window = first_window(&candidate_collection, 3);
    let mut candidate_consumer = domain::WorthQueryCollectionConsumerWindow::from_bound(
        candidate_collection,
        candidate_window,
    )
    .unwrap();

    let (subject, candidate) = shared_leases(request_execution, &mut workspace);
    assert!(subject.drain(&mut workspace).unwrap().delivery().is_empty());
    assert!(candidate
        .drain(&mut workspace)
        .unwrap()
        .delivery()
        .is_empty());
    workspace
        .update(request_execution, changed, |mutation| {
            mutation.set_aspect(
                runtime::WorthQueryAspectTouch::whole_aspect(matrix_aspect_key()),
                matrix_value_with_order(0, "05"),
            )
        })
        .unwrap();

    let subject_delivery = subject.drain(&mut workspace).unwrap();
    let candidate_delivery = candidate.drain(&mut workspace).unwrap();
    assert!(subject_delivery.shares_invalidation_epoch_with(&candidate_delivery));
    assert_eq!(subject_delivery.counters().underlying_maintenance_passes, 1);
    assert_eq!(subject_delivery.counters().fanout_targets, 2);
    let subject_delta = subject
        .consumer_invalidation_delta(subject_delivery)
        .unwrap();
    let candidate_delta = candidate
        .consumer_invalidation_delta(candidate_delivery)
        .unwrap();
    assert!(subject_delta.shares_epoch_with(&candidate_delta));
    assert!(subject_delta.retains_same_impact_as(&candidate_delta));
    let subject_admitted = subject
        .admit_consumer_invalidation_delta(subject_delta, &workspace)
        .unwrap_or_else(|stop| panic!("subject delta stopped: {:?}", stop.kind()));
    let candidate_admitted = candidate
        .admit_consumer_invalidation_delta(candidate_delta, &workspace)
        .unwrap_or_else(|stop| panic!("candidate delta stopped: {:?}", stop.kind()));
    subject_consumer
        .bind_shared_target(&subject_admitted, &workspace)
        .unwrap();
    candidate_consumer
        .bind_shared_target(&candidate_admitted, &workspace)
        .unwrap();

    let subject_patch = required_patch(
        request_execution,
        &mut subject_consumer,
        &subject_admitted,
        &workspace,
    );
    let candidate_patch = required_patch(
        request_execution,
        &mut candidate_consumer,
        &candidate_admitted,
        &workspace,
    );
    let subject_ordinal = subject_patch.maintenance_ordinal();
    let candidate_ordinal = candidate_patch.maintenance_ordinal();
    let denial = match candidate_consumer.apply_patch(subject_patch) {
        Err(denial) => denial,
        Ok(_) => panic!("subject lease patch applied through the candidate lease"),
    };
    assert_eq!(
        denial.kind(),
        domain::WorthQueryCollectionDeliveryDenialKind::WrongLease
    );
    let subject_patch = required_patch(
        request_execution,
        &mut subject_consumer,
        &subject_admitted,
        &workspace,
    );
    subject_consumer.apply_patch(subject_patch).unwrap();
    candidate_consumer.apply_patch(candidate_patch).unwrap();
    assert_eq!(subject_ordinal, candidate_ordinal);
    assert_eq!(
        subject_consumer
            .rows()
            .iter()
            .map(|row| (row.entity_identity(), row.view_local_identity()))
            .collect::<Vec<_>>(),
        candidate_consumer
            .rows()
            .iter()
            .map(|row| (row.entity_identity(), row.view_local_identity()))
            .collect::<Vec<_>>()
    );
}

fn shared_leases(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    workspace: &mut runtime::WorthQueryWorkspace,
) -> (super::CollectionLease, super::CollectionLease) {
    let (subject, _) = settled_with_order_key(workspace);
    let live = match subject
        .into_lifecycle()
        .promote(request_execution, workspace)
    {
        domain::WorthQueryProjectionPromotionOutcome::Promoted(live) => live,
        _ => panic!("shared collection subject did not promote"),
    };
    let (candidate, _) = settled_with_order_key(workspace);
    let shared = match live.share_with(candidate.into_lifecycle(), workspace) {
        domain::WorthQueryProjectionSharingOutcome::Shared(shared) => shared,
        domain::WorthQueryProjectionSharingOutcome::Stopped(stop) => {
            panic!("shared collection admission stopped: {}", stop.detail())
        }
    };
    shared.into_leases()
}
