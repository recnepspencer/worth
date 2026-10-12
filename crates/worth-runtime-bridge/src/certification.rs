//! Always-compiled, boundary-sealed drivers for certification against the real Bridge/Signal
//! async lifecycle.
//!
//! These functions create external lifecycle conditions and return ordinary
//! Bridge evidence. They do not construct mixed-cause transitions or Query
//! result states.

use crate::facade::{
    AdmittedBridgeAsyncRequestIdentity, BridgeAsyncDeniedCompletion,
    BridgeAsyncRequestAdmissionRequest, BridgeAsyncRequestTruthViewBasis, BridgeAsyncRetryLineage,
    BridgeAsyncRetryLineageRequest, BridgeAsyncSourceDeclarationDraft,
    BridgeAsyncSourceDeclarationIdentity, BridgeAsyncSourceLegacyDeclarationIdentity,
    RuntimeBridge,
};
use worth_signal::facade::{
    NodeId, RawCompletionEnvelope, ResourceCancellationReason, ResourceNodeDeclaration,
    ResourceObservationPolicyDeclaration, ResourcePayloadContract, ResourcePayloadContractId,
    ResourceRejectionReason, ResourceRetryPolicyDeclaration, TemporalDuration,
};

pub fn reject_async_request_for_test(
    bridge: &RuntimeBridge,
    request: &AdmittedBridgeAsyncRequestIdentity,
) -> BridgeAsyncDeniedCompletion {
    crate::source::with_async_request_signal_runtime(bridge.signal_runtime_key, |signal_runtime| {
        signal_runtime
            .reject_resource_request(
                request.request_handle(),
                ResourceRejectionReason::SemanticFailure,
            )
            .expect("certification rejection must reach the owning Signal request")
    })
    .expect("certification rejection must remain on the Bridge owner thread");
    denied_completion(bridge, request)
}

pub fn cancel_async_request_for_test(
    bridge: &RuntimeBridge,
    request: &AdmittedBridgeAsyncRequestIdentity,
) -> BridgeAsyncDeniedCompletion {
    cancel_request(bridge, request);
    denied_completion(bridge, request)
}

pub fn deny_oversized_async_completion_for_test(
    bridge: &RuntimeBridge,
    request: &AdmittedBridgeAsyncRequestIdentity,
) -> BridgeAsyncDeniedCompletion {
    completion_report_with_bytes(bridge, request, 4_096)
        .denied_completion()
        .expect("oversized certification completion must deny")
        .clone()
}

pub fn observe_late_async_completion_for_test(
    bridge: &RuntimeBridge,
    request: &AdmittedBridgeAsyncRequestIdentity,
) -> BridgeAsyncDeniedCompletion {
    denied_completion(bridge, request)
}

pub fn supersede_async_request_for_test(
    bridge: &RuntimeBridge,
    request: &AdmittedBridgeAsyncRequestIdentity,
) -> (
    BridgeAsyncDeniedCompletion,
    AdmittedBridgeAsyncRequestIdentity,
) {
    let replacement = bridge
        .admit_async_request_identity(
            BridgeAsyncRequestAdmissionRequest::request_response(
                request.lowered(),
                request.basis_binding(),
            )
            .expect("certification replacement request must preserve request-response shape"),
        )
        .expect("certification replacement request must admit");
    (denied_completion(bridge, request), replacement)
}

pub fn retryable_async_request_for_test(
    bridge: &RuntimeBridge,
    node: NodeId,
    truth_basis: BridgeAsyncRequestTruthViewBasis,
) -> AdmittedBridgeAsyncRequestIdentity {
    let declaration = ResourceNodeDeclaration::new(
        worth_signal::facade::ResourceNodeId::from_node(node),
        ResourcePayloadContract::new(ResourcePayloadContractId::new(313))
            .with_max_payload_bytes(512),
    )
    .with_observation_policy(ResourceObservationPolicyDeclaration::LifecycleOnly)
    .with_retry_policy(ResourceRetryPolicyDeclaration::FixedDelay {
        delay: TemporalDuration::temporal_duration(3)
            .expect("static certification retry delay must admit"),
    })
    .with_retry_max_attempts(3);
    let draft = BridgeAsyncSourceDeclarationDraft::request_response(
        BridgeAsyncSourceDeclarationIdentity::from_stable_name("worth-ui:projection-retry-source"),
        BridgeAsyncSourceLegacyDeclarationIdentity::from_stable_name(
            "worth-ui:legacy-projection-retry-source",
        ),
        declaration,
    );
    let lowered = bridge
        .lower_async_source_declaration(
            &bridge
                .validate_async_source_declaration(draft)
                .expect("certification retry source declaration must validate"),
        )
        .expect("certification retry source declaration must lower");
    let binding = bridge.bind_async_request_basis(&lowered, truth_basis);
    bridge
        .admit_async_request_identity(
            BridgeAsyncRequestAdmissionRequest::request_response(&lowered, &binding)
                .expect("certification retry request must construct"),
        )
        .expect("certification retry request must admit")
}

pub fn cancel_and_retry_async_request_for_test(
    bridge: &RuntimeBridge,
    prior: &AdmittedBridgeAsyncRequestIdentity,
) -> (BridgeAsyncDeniedCompletion, BridgeAsyncRetryLineage) {
    let cancellation = cancel_request(bridge, prior);
    let denied = denied_completion(bridge, prior);
    let newer = bridge
        .admit_async_request_identity(
            BridgeAsyncRequestAdmissionRequest::request_response(
                prior.lowered(),
                prior.basis_binding(),
            )
            .expect("certification retry successor must preserve request-response shape"),
        )
        .expect("certification retry successor must admit");
    let lineage = bridge
        .admit_async_retry_lineage_after_cancellation(
            BridgeAsyncRetryLineageRequest::after_cancellation(prior, &cancellation, &newer),
        )
        .expect("real cancellation evidence must admit retry lineage");
    (denied, lineage)
}

fn cancel_request(
    bridge: &RuntimeBridge,
    request: &AdmittedBridgeAsyncRequestIdentity,
) -> worth_signal::facade::ResourceCancellationReport {
    crate::source::with_async_request_signal_runtime(bridge.signal_runtime_key, |signal_runtime| {
        signal_runtime
            .cancel_resource_request(
                request.request_handle(),
                ResourceCancellationReason::HostRequested,
            )
            .expect("certification cancellation must reach the owning Signal request")
    })
    .expect("certification cancellation must remain on the Bridge owner thread")
}

fn denied_completion(
    bridge: &RuntimeBridge,
    request: &AdmittedBridgeAsyncRequestIdentity,
) -> BridgeAsyncDeniedCompletion {
    completion_report(bridge, request)
        .denied_completion()
        .expect("certification lifecycle condition must deny completion")
        .clone()
}

fn completion_report(
    bridge: &RuntimeBridge,
    request: &AdmittedBridgeAsyncRequestIdentity,
) -> crate::facade::BridgeAsyncCompletionAdmissionReport {
    completion_report_with_bytes(bridge, request, 64)
}

fn completion_report_with_bytes(
    bridge: &RuntimeBridge,
    request: &AdmittedBridgeAsyncRequestIdentity,
    payload_bytes: u64,
) -> crate::facade::BridgeAsyncCompletionAdmissionReport {
    let validated = bridge
        .validate_async_completion_envelope(request, raw_completion(request, payload_bytes))
        .expect("certification completion envelope must validate");
    bridge
        .admit_async_completion(request, &validated)
        .expect("certification completion must classify through Bridge")
}

fn raw_completion(
    request: &AdmittedBridgeAsyncRequestIdentity,
    payload_bytes: u64,
) -> RawCompletionEnvelope {
    RawCompletionEnvelope::new(
        request.request_handle().request_id(),
        request.request_handle().generation(),
        request.request_handle().branch_epoch(),
        request.attempt(),
        request
            .lowered()
            .resource_descriptor()
            .expect("request-response identity must retain its descriptor")
            .payload_contract_digest()
            .clone(),
        payload_bytes,
    )
}
