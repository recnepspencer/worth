use crate::facade::{
    AdmittedBridgeAsyncRequestIdentity, BridgeAsyncCompletionAdmissionReport,
    BridgeAsyncCompletionRejection, BridgeAsyncCompletionRejectionKind,
    BridgeAsyncRequestAdmissionRequest, BridgeAsyncRequestSubscriptionInstance,
    BridgeAsyncRequestTruthViewBasis, RuntimeBridge,
};
use crate::source::with_async_request_signal_runtime;
use worth_signal::facade::{
    ClockAdvanceRequest, ClockDomain, ClockTick, ResourceCancellationReason,
    ResourceRejectionReason, ResourceRequestIntent,
};
use worth_signal::facade::{NodeId, RawCompletionEnvelope, ResourcePayloadContractDigest};

use super::{
    activation_ready_for_snapshot, admit_request_response_identity,
    admit_timeout_request_response_identity, lowered_request_response, lowered_subscription_backed,
    preview_active_subscription,
};

pub(crate) fn request_response_raw_completion(
    request_identity: &AdmittedBridgeAsyncRequestIdentity,
    payload_byte_len: u64,
) -> RawCompletionEnvelope {
    RawCompletionEnvelope::new(
        request_identity.request_handle().request_id(),
        request_identity.request_handle().generation(),
        request_identity.request_handle().branch_epoch(),
        request_identity.attempt(),
        request_identity
            .lowered()
            .resource_descriptor()
            .expect("request-response identity should retain resource descriptor")
            .payload_contract_digest()
            .clone(),
        payload_byte_len,
    )
}

pub(crate) fn subscription_backed_raw_completion(
    request_identity: &AdmittedBridgeAsyncRequestIdentity,
    payload_byte_len: u64,
) -> RawCompletionEnvelope {
    RawCompletionEnvelope::new(
        request_identity.request_handle().request_id(),
        request_identity.request_handle().generation(),
        request_identity.request_handle().branch_epoch(),
        request_identity.attempt(),
        ResourcePayloadContractDigest::new(
            request_identity
                .lowered()
                .async_node_capability_bundle()
                .expect("subscription-backed identity should retain capability bundle")
                .payload_contract_digest()
                .as_str(),
        ),
        payload_byte_len,
    )
}

pub(crate) fn admit_request_response_completion(
    runtime: &RuntimeBridge,
    node: NodeId,
    truth_basis: BridgeAsyncRequestTruthViewBasis,
    payload_byte_len: u64,
) -> BridgeAsyncCompletionAdmissionReport {
    let request_identity = admit_request_response_identity(runtime, node, truth_basis);
    let validated = runtime
        .validate_async_completion_envelope(
            &request_identity,
            request_response_raw_completion(&request_identity, payload_byte_len),
        )
        .expect("request-response completion envelope should validate");
    runtime
        .admit_async_completion(&request_identity, &validated)
        .expect("request-response completion should admit or deny canonically")
}

pub(crate) fn admit_subscription_backed_completion(
    runtime: &RuntimeBridge,
    node: NodeId,
    truth_basis: BridgeAsyncRequestTruthViewBasis,
    payload_byte_len: u64,
    resource_request: worth_execution::ExecutionRequest<'_, '_>,
) -> Result<BridgeAsyncCompletionAdmissionReport, BridgeAsyncCompletionRejection> {
    let subscription_instance =
        BridgeAsyncRequestSubscriptionInstance::authoritative(&activation_ready_for_snapshot(
            runtime,
            crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
            resource_request,
        ));
    let request_identity = super::admit_subscription_backed_identity(
        runtime,
        node,
        truth_basis,
        subscription_instance,
    )
    .map_err(|error| {
        BridgeAsyncCompletionRejection::new(
            BridgeAsyncCompletionRejectionKind::SignalCompletionAdmissionUnavailable,
            error.detail().to_owned(),
        )
    })?;
    let validated = runtime.validate_async_completion_envelope(
        &request_identity,
        subscription_backed_raw_completion(&request_identity, payload_byte_len),
    )?;
    runtime.admit_async_completion(&request_identity, &validated)
}

pub(crate) fn mismatched_payload_completion(
    runtime: &RuntimeBridge,
) -> (AdmittedBridgeAsyncRequestIdentity, RawCompletionEnvelope) {
    let lowered = lowered_request_response(runtime, NodeId::new(37, 0));
    let binding = runtime.bind_async_request_basis(
        &lowered,
        BridgeAsyncRequestTruthViewBasis::authoritative(
            crate::truth_identity_fixtures::truth_branch_fixture("truth-main"),
            crate::truth_identity_fixtures::truth_commit_fixture("commit-a"),
            crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
        ),
    );
    let request =
        crate::facade::BridgeAsyncRequestAdmissionRequest::request_response(&lowered, &binding)
            .expect("request-response request should construct");
    let identity = runtime
        .admit_async_request_identity(request)
        .expect("request-response identity should admit");
    let raw = RawCompletionEnvelope::new(
        identity.request_handle().request_id(),
        identity.request_handle().generation(),
        identity.request_handle().branch_epoch(),
        identity.attempt(),
        ResourcePayloadContractDigest::new("payload-contract:999:1024"),
        64,
    );
    (identity, raw)
}

pub(crate) fn denied_request_response_completion_after_rejection(
    runtime: &RuntimeBridge,
    node: NodeId,
    truth_basis: BridgeAsyncRequestTruthViewBasis,
) -> BridgeAsyncCompletionAdmissionReport {
    let request_identity = admit_request_response_identity(runtime, node, truth_basis);
    let raw = request_response_raw_completion(&request_identity, 64);
    with_async_request_signal_runtime(runtime.signal_runtime_key, |signal_runtime| {
        signal_runtime
            .reject_resource_request(
                request_identity.request_handle(),
                ResourceRejectionReason::SemanticFailure,
            )
            .expect("request rejection should succeed");
    })
    .expect("signal runtime should stay on the owning thread");
    let validated = runtime
        .validate_async_completion_envelope(&request_identity, raw)
        .expect("late rejected completion envelope should validate");
    runtime
        .admit_async_completion(&request_identity, &validated)
        .expect("late rejected completion should admit or deny canonically")
}

pub(crate) fn denied_request_response_completion_after_cancellation(
    runtime: &RuntimeBridge,
    node: NodeId,
    truth_basis: BridgeAsyncRequestTruthViewBasis,
) -> BridgeAsyncCompletionAdmissionReport {
    let request_identity = admit_request_response_identity(runtime, node, truth_basis);
    let raw = request_response_raw_completion(&request_identity, 64);
    with_async_request_signal_runtime(runtime.signal_runtime_key, |signal_runtime| {
        signal_runtime
            .cancel_resource_request(
                request_identity.request_handle(),
                ResourceCancellationReason::HostRequested,
            )
            .expect("request cancellation should succeed");
    })
    .expect("signal runtime should stay on the owning thread");
    let validated = runtime
        .validate_async_completion_envelope(&request_identity, raw)
        .expect("late cancelled completion envelope should validate");
    runtime
        .admit_async_completion(&request_identity, &validated)
        .expect("late cancelled completion should admit or deny canonically")
}

pub(crate) fn denied_request_response_completion_after_supersession(
    runtime: &RuntimeBridge,
    node: NodeId,
    truth_basis: BridgeAsyncRequestTruthViewBasis,
) -> BridgeAsyncCompletionAdmissionReport {
    let lowered = lowered_request_response(runtime, node);
    let binding = runtime.bind_async_request_basis(&lowered, truth_basis);
    let request = BridgeAsyncRequestAdmissionRequest::request_response(&lowered, &binding)
        .expect("request-response basis binding should construct");
    let request_identity = runtime
        .admit_async_request_identity(request.clone())
        .expect("first request-response identity should admit");
    let raw = request_response_raw_completion(&request_identity, 64);
    let _replacement = runtime
        .admit_async_request_identity(request)
        .expect("second request-response identity should supersede the first");
    let validated = runtime
        .validate_async_completion_envelope(&request_identity, raw)
        .expect("late superseded completion envelope should validate");
    runtime
        .admit_async_completion(&request_identity, &validated)
        .expect("late superseded completion should admit or deny canonically")
}

pub(crate) fn denied_request_response_completion_after_timeout(
    runtime: &RuntimeBridge,
    node: NodeId,
    truth_basis: BridgeAsyncRequestTruthViewBasis,
    timeout_tick: u64,
) -> BridgeAsyncCompletionAdmissionReport {
    let request_identity =
        admit_timeout_request_response_identity(runtime, node, truth_basis, timeout_tick);
    let raw = request_response_raw_completion(&request_identity, 64);
    with_async_request_signal_runtime(runtime.signal_runtime_key, |signal_runtime| {
        let wake_id = signal_runtime
            .in_flight_resource_request(request_identity.request_handle())
            .and_then(|in_flight| in_flight.timeout_wake_id())
            .expect("timeout wake should attach");
        signal_runtime
            .advance_clock(ClockAdvanceRequest::new(
                ClockDomain::MonotonicExecution,
                ClockTick::new(timeout_tick),
            ))
            .expect("clock should advance");
        let ready = signal_runtime
            .promote_temporal_wake_ready(wake_id)
            .expect("timeout wake should promote");
        signal_runtime
            .admit_resource_timeout(request_identity.request_handle(), ready)
            .expect("timeout admission should succeed");
    })
    .expect("signal runtime should stay on the owning thread");
    let validated = runtime
        .validate_async_completion_envelope(&request_identity, raw)
        .expect("late timed out completion envelope should validate");
    runtime
        .admit_async_completion(&request_identity, &validated)
        .expect("late timed out completion should admit or deny canonically")
}

pub(crate) fn denied_request_response_completion_after_restore_staleness(
    runtime: &RuntimeBridge,
    node: NodeId,
    truth_basis: BridgeAsyncRequestTruthViewBasis,
) -> BridgeAsyncCompletionAdmissionReport {
    let request_identity = admit_request_response_identity(runtime, node, truth_basis);
    let raw = request_response_raw_completion(&request_identity, 64);
    with_async_request_signal_runtime(runtime.signal_runtime_key, |signal_runtime| {
        let branch = signal_runtime.current_branch();
        let basis = signal_runtime
            .observe_signal_branch_basis(branch.clone())
            .expect("active branch should admit an owner-bound basis");
        let (snapshot, _captured_basis) = signal_runtime
            .capture_signal_branch_snapshot(&basis)
            .expect("snapshot capture should succeed through the owner basis")
            .into_parts();
        let node = request_identity
            .lowered()
            .request_response_declaration()
            .expect("request-response identity should retain declaration")
            .node();
        signal_runtime
            .admit_resource_request(ResourceRequestIntent::new(node))
            .expect("post-snapshot request should mutate runtime before restore");
        let restore_basis = signal_runtime
            .observe_signal_branch_basis(branch)
            .expect("mutated branch should admit a current owner-bound basis");
        signal_runtime
            .restore_signal_branch(&restore_basis, &snapshot)
            .expect("restore should rekey resource request handles");
    })
    .expect("signal runtime should stay on the owning thread");
    let validated = runtime
        .validate_async_completion_envelope(&request_identity, raw)
        .expect("stale completion envelope should validate before lifecycle denial");
    runtime
        .admit_async_completion(&request_identity, &validated)
        .expect("stale completion should admit or deny canonically")
}

pub(crate) fn denied_request_response_completion_with_displacing_identity(
    runtime: &RuntimeBridge,
    node: NodeId,
    original_truth_basis: BridgeAsyncRequestTruthViewBasis,
    current_truth_basis: BridgeAsyncRequestTruthViewBasis,
) -> (
    crate::facade::BridgeAsyncDeniedCompletion,
    AdmittedBridgeAsyncRequestIdentity,
) {
    let lowered = lowered_request_response(runtime, node);
    let original_binding = runtime.bind_async_request_basis(&lowered, original_truth_basis);
    let original_request =
        BridgeAsyncRequestAdmissionRequest::request_response(&lowered, &original_binding)
            .expect("original request-response basis binding should construct");
    let original_identity = runtime
        .admit_async_request_identity(original_request)
        .expect("original request-response identity should admit");
    let raw = request_response_raw_completion(&original_identity, 64);
    let current_binding = runtime.bind_async_request_basis(&lowered, current_truth_basis);
    let current_request =
        BridgeAsyncRequestAdmissionRequest::request_response(&lowered, &current_binding)
            .expect("current request-response basis binding should construct");
    let current_identity = runtime
        .admit_async_request_identity(current_request)
        .expect("current request-response identity should admit");
    let validated = runtime
        .validate_async_completion_envelope(&original_identity, raw)
        .expect("late request-response completion envelope should validate");
    let report = runtime
        .admit_async_completion(&original_identity, &validated)
        .expect("late request-response completion should deny canonically");
    (
        report
            .denied_completion()
            .expect("superseded request-response completion should deny")
            .clone(),
        current_identity,
    )
}

pub(crate) fn denied_subscription_backed_completion_with_displacing_identity(
    runtime: &RuntimeBridge,
    node: NodeId,
    original_truth_basis: BridgeAsyncRequestTruthViewBasis,
    original_subscription_instance: BridgeAsyncRequestSubscriptionInstance,
    current_truth_basis: BridgeAsyncRequestTruthViewBasis,
    current_subscription_instance: BridgeAsyncRequestSubscriptionInstance,
) -> (
    crate::facade::BridgeAsyncDeniedCompletion,
    AdmittedBridgeAsyncRequestIdentity,
) {
    let lowered = lowered_subscription_backed(runtime, node);
    let original_binding = runtime.bind_async_request_basis(&lowered, original_truth_basis);
    let original_request = BridgeAsyncRequestAdmissionRequest::subscription_backed(
        &lowered,
        &original_binding,
        original_subscription_instance,
    )
    .expect("original subscription-backed request should construct");
    let original_identity = runtime
        .admit_async_request_identity(original_request)
        .expect("original subscription-backed identity should admit");
    let raw = subscription_backed_raw_completion(&original_identity, 64);
    let current_binding = runtime.bind_async_request_basis(&lowered, current_truth_basis);
    let current_request = BridgeAsyncRequestAdmissionRequest::subscription_backed(
        &lowered,
        &current_binding,
        current_subscription_instance,
    )
    .expect("current subscription-backed request should construct");
    let current_identity = runtime
        .admit_async_request_identity(current_request)
        .expect("current subscription-backed identity should admit");
    let validated = runtime
        .validate_async_completion_envelope(&original_identity, raw)
        .expect("late subscription-backed completion envelope should validate");
    let report = runtime
        .admit_async_completion(&original_identity, &validated)
        .expect("late subscription-backed completion should deny canonically");
    (
        report
            .denied_completion()
            .expect("superseded subscription-backed completion should deny")
            .clone(),
        current_identity,
    )
}

pub(crate) fn denied_preview_subscription_backed_completion_after_discard(
    runtime: &RuntimeBridge,
    node: NodeId,
    suffix: &str,
    resource_request: worth_execution::ExecutionRequest<'_, '_>,
) -> (
    crate::facade::BridgeAsyncDeniedCompletion,
    BridgeAsyncRequestTruthViewBasis,
    BridgeAsyncRequestSubscriptionInstance,
) {
    let preview_active = preview_active_subscription(runtime, suffix, resource_request);
    let preview_truth_basis = BridgeAsyncRequestTruthViewBasis::preview(&preview_active);
    let preview_subscription_instance =
        BridgeAsyncRequestSubscriptionInstance::preview(&preview_active);
    let (denied, _) = denied_subscription_backed_completion_with_displacing_identity(
        runtime,
        node,
        preview_truth_basis.clone(),
        preview_subscription_instance.clone(),
        preview_truth_basis.clone(),
        preview_subscription_instance.clone(),
    );
    (denied, preview_truth_basis, preview_subscription_instance)
}
