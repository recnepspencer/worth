use super::fixture::{completion, source_commit, AsyncSourceCourt};
use std::num::NonZeroU64;
use worth_runtime_bridge::facade::{
    BridgeOwnedAsyncManagedDenial, BridgeOwnedAsyncManagedRequestAdmission, TruthCommitIdentity,
};
use worth_runtime_world::facade::ProductBranchObservation;
use worth_signal::facade::{ResourceManagedQueueDenialClass, ResourceManagedQueueMutationReport};

fn assert_queue_denial(
    result: Result<ResourceManagedQueueMutationReport, BridgeOwnedAsyncManagedDenial>,
    expected: ResourceManagedQueueDenialClass,
) {
    let Err(BridgeOwnedAsyncManagedDenial::Signal(
        worth_signal::facade::branch::SignalOwnedAsyncQueueDenial::Queue(denied),
    )) = result
    else {
        panic!("actual native queue must supply the refusal");
    };
    assert_eq!(denied.class(), expected);
}

fn selected_fork(court: &AsyncSourceCourt) -> ProductBranchObservation {
    let host = crate::test_execution_policy::serial_request();
    let execution = worth_execution::ExecutionRequest::serial(&host);
    let committed = court.publish(execution, &court.empty);
    let fork = court.fork(&committed, "managed-async-fork");
    drop(court.prepare(&execution, &committed));
    let sibling = court.publish(execution, &committed);
    assert!(
        sibling
            .basis()
            .relational_basis()
            .observation()
            .commit_id()
            .unwrap()
            .0
            > fork
                .basis()
                .relational_basis()
                .observation()
                .commit_id()
                .unwrap()
                .0
                + 1
    );
    fork
}

fn admit(
    court: &AsyncSourceCourt,
    selected: &ProductBranchObservation,
) -> BridgeOwnedAsyncManagedRequestAdmission {
    let lease = court.lease(selected);
    let owner = court
        .world
        .admit_managed_owned_async_request(&court.bridge, &court.declaration, selected, &lease)
        .expect("actual selected source admits a managed native request");
    assert!(!owner.cleanup_required());
    assert!(owner.request().is_some());
    owner
}

#[test]
fn managed_queue_saturates_and_cancellation_leaves_exact_drain_custody() {
    let court = AsyncSourceCourt::managed(NonZeroU64::MIN);
    let selected = selected_fork(&court);
    let owner = admit(&court, &selected);
    let request = owner.request().unwrap();
    let source = selected
        .basis()
        .relational_basis()
        .observation()
        .commit_id()
        .unwrap();
    assert_eq!(
        source_commit(request),
        &TruthCommitIdentity::from_relational_commit_id(source.0)
    );
    assert!(
        court
            .world
            .admit_managed_owned_async_request(
                &court.bridge,
                &court.declaration,
                &selected,
                &court.lease(&selected),
            )
            .is_err(),
        "a busy native node cannot coalesce another managed occurrence"
    );
    assert!(
        court
            .world
            .admit_owned_async_request(
                &court.bridge,
                &court.declaration,
                &selected,
                &court.lease(&selected),
            )
            .is_err(),
        "legacy admission cannot discard the installed queue contract"
    );
    assert_eq!(
        court
            .bridge
            .enqueue_managed_owned_async_queue(&owner, 1)
            .unwrap()
            .pressure()
            .queue_depth(),
        1
    );
    assert_queue_denial(
        court.bridge.enqueue_managed_owned_async_queue(&owner, 1),
        ResourceManagedQueueDenialClass::CapacityExceeded,
    );
    assert_eq!(
        court
            .bridge
            .observe_managed_owned_async_safe_point(&owner)
            .unwrap()
            .pressure()
            .queue_depth(),
        1
    );
    let cancelled = court
        .bridge
        .cancel_managed_owned_async_request(&owner)
        .unwrap();
    assert!(cancelled.cancelled_request().is_some());
    assert_eq!(
        court
            .bridge
            .owned_async_active_request_count(request)
            .unwrap(),
        0
    );
    assert_eq!(
        court
            .bridge
            .observe_managed_owned_async_safe_point(&owner)
            .unwrap()
            .pressure()
            .queue_depth(),
        1,
        "native cancellation does not drain the admitted queue"
    );
    assert_queue_denial(
        court.bridge.enqueue_managed_owned_async_queue(&owner, 1),
        ResourceManagedQueueDenialClass::RequestNotActive,
    );
    assert_eq!(
        court
            .bridge
            .dequeue_managed_owned_async_queue(&owner, 1)
            .unwrap()
            .pressure()
            .queue_depth(),
        0
    );
    assert_queue_denial(
        court.bridge.dequeue_managed_owned_async_queue(&owner, 1),
        ResourceManagedQueueDenialClass::DepthUnderflow,
    );
}

#[test]
fn actual_completion_commit_releases_active_index_and_old_cleanup_cannot_cancel_reused_node() {
    let court = AsyncSourceCourt::managed(NonZeroU64::MIN);
    let selected = selected_fork(&court);
    let owner = admit(&court, &selected);
    let request = owner.request().unwrap();
    court
        .bridge
        .enqueue_managed_owned_async_queue(&owner, 1)
        .unwrap();
    let validated = court
        .bridge
        .validate_owned_async_completion_envelope(request, completion(request))
        .unwrap();
    let admitted = court
        .bridge
        .admit_owned_async_completion(request, &validated)
        .unwrap();
    assert!(admitted.report().admitted_completion().is_some());
    assert_eq!(
        court
            .bridge
            .owned_async_active_request_count(request)
            .unwrap(),
        1,
        "admission is not native fulfillment"
    );
    court
        .bridge
        .dequeue_managed_owned_async_queue(&owner, 1)
        .unwrap();
    let mut committed = court
        .bridge
        .commit_owned_async_completion(request, &admitted)
        .unwrap();
    assert_eq!(
        committed.report().lifecycle().lifecycle(),
        worth_signal::facade::ResourceLifecycleClass::Fulfilled
    );
    assert!(committed.timeout_cleanup_complete());
    assert!(committed.timeout_cleanup_denial().is_none());
    assert_eq!(
        court
            .bridge
            .owned_async_active_request_count(request)
            .unwrap(),
        0
    );
    let successor = admit(&court, &selected);
    let newer = successor.request().unwrap();
    assert_ne!(owner.request_handle(), successor.request_handle());
    court
        .bridge
        .finish_owned_async_completion_timeout_cleanup(&mut committed)
        .unwrap();
    assert!(court
        .bridge
        .cancel_managed_owned_async_request(&owner)
        .unwrap()
        .cancelled_request()
        .is_none());
    assert!(court
        .bridge
        .commit_owned_async_completion(request, &admitted)
        .is_err());
    assert!(court
        .bridge
        .enqueue_managed_owned_async_queue(&owner, 1)
        .is_err());
    assert_eq!(
        court
            .bridge
            .owned_async_active_request_count(newer)
            .unwrap(),
        1,
        "old retained result cleanup must leave the successor active"
    );
    court
        .bridge
        .cancel_managed_owned_async_request(&successor)
        .unwrap();
}

#[test]
fn managed_replacement_is_refused_before_legacy_revalidation_or_retry_effects() {
    let court = AsyncSourceCourt::managed(NonZeroU64::MIN);
    let selected = selected_fork(&court);
    let owner = admit(&court, &selected);
    let request = owner.request().unwrap();
    court
        .bridge
        .enqueue_managed_owned_async_queue(&owner, 1)
        .unwrap();
    let host = crate::test_execution_policy::serial_request();
    let execution = worth_execution::ExecutionRequest::serial(&host);
    let newer_source = court.publish(execution, &selected);
    assert!(court
        .world
        .revalidate_owned_async_request(
            &court.bridge,
            request,
            &newer_source,
            &court.lease(&newer_source),
        )
        .is_err());
    assert_eq!(
        court
            .bridge
            .owned_async_active_request_count(request)
            .unwrap(),
        1
    );
    assert_eq!(
        court
            .bridge
            .observe_managed_owned_async_safe_point(&owner)
            .unwrap()
            .pressure()
            .queue_depth(),
        1
    );
    let timeout = court
        .bridge
        .advance_owned_async_request_to_timeout(request, 5)
        .unwrap();
    assert!(court
        .bridge
        .schedule_owned_async_retry(request, &timeout)
        .is_err());
    assert_eq!(
        court
            .bridge
            .observe_managed_owned_async_safe_point(&owner)
            .unwrap()
            .pressure()
            .queue_depth(),
        1
    );
    court
        .bridge
        .dequeue_managed_owned_async_queue(&owner, 1)
        .unwrap();
}

#[test]
fn foreign_and_effect_indeterminate_admissions_cannot_fulfill_an_ordinary_resource() {
    let court = AsyncSourceCourt::managed(NonZeroU64::MIN);
    let selected = selected_fork(&court);
    let owner = admit(&court, &selected);
    let request = owner.request().unwrap();
    let indeterminate = court
        .bridge
        .admit_owned_async_effects_indeterminate(request.effects_indeterminate_issuer().certify(64))
        .unwrap();
    assert_eq!(
        indeterminate
            .report()
            .admitted_completion()
            .unwrap()
            .completion_class(),
        worth_runtime_bridge::facade::BridgeAsyncCompletionClass::EffectsIndeterminate
    );
    assert!(court
        .bridge
        .commit_owned_async_completion(request, &indeterminate)
        .is_err());
    let validated = court
        .bridge
        .validate_owned_async_completion_envelope(request, completion(request))
        .unwrap();
    let admitted = court
        .bridge
        .admit_owned_async_completion(request, &validated)
        .unwrap();
    let foreign = AsyncSourceCourt::managed(NonZeroU64::MIN);
    assert!(admitted.report().admitted_completion().is_some());
    assert!(foreign
        .bridge
        .commit_owned_async_completion(request, &admitted)
        .is_err());
    assert!(foreign
        .bridge
        .cancel_managed_owned_async_request(&owner)
        .is_err());
    assert_eq!(
        court
            .bridge
            .owned_async_active_request_count(request)
            .unwrap(),
        1
    );
    court
        .bridge
        .cancel_managed_owned_async_request(&owner)
        .unwrap();
}
