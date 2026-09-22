use super::super::support::{admit_retryable_request_response_identity, timeout_retry_request};
use super::runtime;
use crate::facade::{
    BridgeAsyncForwardCausalityRejection, BridgeAsyncForwardCausalityRejectionKind,
    BridgeAsyncRequestTruthViewBasis, BridgeAsyncRetryLineageRequest,
    BridgeAsyncRevalidationLineageRequest,
};
use crate::source::{runtime_storage_for_test, with_async_request_signal_runtime};
use worth_signal::facade::{
    NodeId, ResourceCancellationReason, ResourceNodeId, ResourceRevalidationIntent,
};

fn basis(label: &str) -> BridgeAsyncRequestTruthViewBasis {
    BridgeAsyncRequestTruthViewBasis::authoritative(
        crate::truth_identity_fixtures::truth_branch_fixture("truth-main"),
        crate::truth_identity_fixtures::truth_commit_fixture(label),
        crate::truth_identity_fixtures::truth_snapshot_fixture(label),
    )
}

fn assert_mismatch(error: BridgeAsyncForwardCausalityRejection) {
    assert_eq!(
        error.kind(),
        BridgeAsyncForwardCausalityRejectionKind::SignalRuntimeIdentityMismatch
    );
}

#[test]
fn timeout_lineage_rejects_foreign_owner_before_signal_access() {
    let owner = runtime();
    let foreign = runtime();
    let request = timeout_retry_request(&owner, NodeId::new(700, 0), basis("a"), 8);
    assert_mismatch(
        foreign
            .admit_async_retry_lineage_after_timeout(request.clone())
            .unwrap_err(),
    );
    assert_eq!(
        runtime_storage_for_test(foreign.signal_runtime_key),
        (false, false, false, false)
    );
    let wrong_newer =
        admit_retryable_request_response_identity(&foreign, NodeId::new(700, 0), basis("a"));
    assert_mismatch(
        crate::source::admit_owned_retry_lineage(
            request.prior.clone(),
            wrong_newer,
            request.timeout_report.as_ref().unwrap(),
            request.retry_schedule_report.as_ref().unwrap(),
            request.retry_admission_report.as_ref().unwrap(),
        )
        .unwrap_err(),
    );
    let lineage = owner
        .admit_async_retry_lineage_after_timeout(request)
        .unwrap();
    let key = owner.signal_runtime_key;
    let newer = lineage.newer_request().clone();
    drop(lineage);
    drop(owner);
    assert!(runtime_storage_for_test(key).0);
    drop(newer);
    assert_eq!(runtime_storage_for_test(key), (false, false, false, false));
}

#[test]
fn cancellation_lineage_rejects_foreign_prior_and_newer_without_panicking() {
    let owner = runtime();
    let foreign = runtime();
    let prior = admit_retryable_request_response_identity(&owner, NodeId::new(701, 0), basis("a"));
    let cancellation = with_async_request_signal_runtime(owner.signal_runtime_key, |signal| {
        signal
            .cancel_resource_request(
                prior.request_handle(),
                ResourceCancellationReason::HostRequested,
            )
            .unwrap()
    })
    .unwrap();
    let newer =
        admit_retryable_request_response_identity(&foreign, NodeId::new(701, 0), basis("a"));
    let request = BridgeAsyncRetryLineageRequest::after_cancellation(&prior, &cancellation, &newer);
    assert_mismatch(
        owner
            .admit_async_retry_lineage_after_cancellation(request.clone())
            .unwrap_err(),
    );
    assert_mismatch(
        foreign
            .admit_async_retry_lineage_after_cancellation(request)
            .unwrap_err(),
    );
    let correct =
        admit_retryable_request_response_identity(&owner, NodeId::new(701, 0), basis("a"));
    owner
        .admit_async_retry_lineage_after_cancellation(
            BridgeAsyncRetryLineageRequest::after_cancellation(&prior, &cancellation, &correct),
        )
        .unwrap();
}

#[test]
fn revalidation_rejects_foreign_owner_before_mutation_and_preserves_custody() {
    let owner = runtime();
    let foreign = runtime();
    let prior = admit_retryable_request_response_identity(&owner, NodeId::new(702, 0), basis("a"));
    assert_mismatch(
        foreign
            .revalidate_async_request(&prior, basis("b"))
            .unwrap_err(),
    );
    assert_eq!(
        runtime_storage_for_test(foreign.signal_runtime_key),
        (false, false, false, false)
    );
    let report = with_async_request_signal_runtime(owner.signal_runtime_key, |signal| {
        signal
            .revalidate_resource_node(ResourceRevalidationIntent::with_expected_active(
                ResourceNodeId::from_node(prior.in_flight_identity().in_flight().node().node()),
                prior.request_handle(),
            ))
            .unwrap()
    })
    .unwrap();
    let request =
        BridgeAsyncRevalidationLineageRequest::request_response(&prior, basis("b"), &report);
    assert_mismatch(
        foreign
            .admit_async_revalidation_lineage(request.clone())
            .unwrap_err(),
    );
    assert_eq!(
        runtime_storage_for_test(foreign.signal_runtime_key),
        (false, false, false, false)
    );
    let wrong_newer =
        admit_retryable_request_response_identity(&foreign, NodeId::new(702, 0), basis("b"));
    assert_mismatch(
        crate::source::admit_owned_revalidation_lineage(prior.clone(), wrong_newer, &report)
            .unwrap_err(),
    );
    let lineage = owner.admit_async_revalidation_lineage(request).unwrap();
    let newer = lineage.newer_request().clone();
    let key = owner.signal_runtime_key;
    drop(lineage);
    drop(prior);
    drop(owner);
    assert!(runtime_storage_for_test(key).0);
    drop(newer);
    assert_eq!(runtime_storage_for_test(key), (false, false, false, false));
}
