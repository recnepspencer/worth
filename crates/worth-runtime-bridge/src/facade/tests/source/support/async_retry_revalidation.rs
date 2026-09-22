use crate::facade::{
    AdmittedBridgeAsyncRequestIdentity, BridgeAsyncRequestAdmissionRequest,
    BridgeAsyncRequestSubscriptionInstance, BridgeAsyncRequestTruthViewBasis,
    BridgeAsyncRetryLineage, BridgeAsyncRetryLineageRequest, BridgeAsyncRevalidationLineage,
    BridgeAsyncRevalidationLineageRequest, BridgeAsyncSourceDeclarationDraft,
    BridgeAsyncSourceDeclarationIdentity, BridgeAsyncSourceLegacyDeclarationIdentity,
    RuntimeBridge,
};
use crate::source::with_async_request_signal_runtime;
use worth_signal::facade::ResourceObservationPolicyDeclaration;
use worth_signal::facade::{
    ClockAdvanceRequest, ClockDomain, ClockTick, ResourceCancellationReason, ResourceNodeId,
    ResourceRetryPolicyDeclaration, ResourceRetryReason, ResourceRevalidationIntent,
    ResourceTimeoutPolicyDeclaration, TemporalDuration,
};
use worth_signal::facade::{
    NodeId, ResourceNodeDeclaration, ResourceNodeId as SignalResourceNodeId,
};
use worth_signal::facade::{ResourcePayloadContract, ResourcePayloadContractId};

use super::{admit_request_response_identity, admit_subscription_backed_identity};

pub(crate) fn retry_lineage_after_timeout(
    runtime: &RuntimeBridge,
    node: NodeId,
    truth_basis: BridgeAsyncRequestTruthViewBasis,
    timeout_tick: u64,
) -> BridgeAsyncRetryLineage {
    runtime
        .admit_async_retry_lineage_after_timeout(timeout_retry_request(
            runtime,
            node,
            truth_basis,
            timeout_tick,
        ))
        .expect("timeout retry lineage should admit")
}

pub(crate) fn timeout_retry_request(
    runtime: &RuntimeBridge,
    node: NodeId,
    truth_basis: BridgeAsyncRequestTruthViewBasis,
    timeout_tick: u64,
) -> BridgeAsyncRetryLineageRequest {
    let prior =
        admit_retryable_timeout_request_response_identity(runtime, node, truth_basis, timeout_tick);
    let (timeout_report, retry_schedule, retry_admission) =
        with_async_request_signal_runtime(runtime.signal_runtime_key, |signal_runtime| {
            let wake_id = signal_runtime
                .in_flight_resource_request(prior.request_handle())
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
            let timeout_report = signal_runtime
                .admit_resource_timeout(prior.request_handle(), ready)
                .expect("timeout admission should succeed");
            let retry_schedule = signal_runtime
                .schedule_resource_retry(prior.request_handle(), ResourceRetryReason::TimedOut)
                .expect("retry scheduling should succeed");
            let scheduled = retry_schedule
                .scheduled_retry()
                .expect("retry schedule should admit one retry");
            signal_runtime
                .advance_clock(ClockAdvanceRequest::new(
                    ClockDomain::MonotonicExecution,
                    ClockTick::new(timeout_tick.saturating_add(scheduled.scheduled_delay().get())),
                ))
                .expect("clock should advance to retry wake");
            let retry_ready = signal_runtime
                .promote_temporal_wake_ready(scheduled.backoff_wake_id())
                .expect("retry backoff wake should promote");
            let retry_admission = signal_runtime
                .admit_scheduled_resource_retry(prior.request_handle(), retry_ready)
                .expect("retry admission should succeed");
            (timeout_report, retry_schedule, retry_admission)
        })
        .expect("signal runtime should stay on the owning thread");
    BridgeAsyncRetryLineageRequest::after_timeout(
        &prior,
        &timeout_report,
        &retry_schedule,
        &retry_admission,
    )
}

pub(crate) fn retry_lineage_after_cancellation(
    runtime: &RuntimeBridge,
    node: NodeId,
    truth_basis: BridgeAsyncRequestTruthViewBasis,
) -> BridgeAsyncRetryLineage {
    let prior = admit_retryable_request_response_identity(runtime, node, truth_basis);
    let cancellation_report =
        with_async_request_signal_runtime(runtime.signal_runtime_key, |signal_runtime| {
            signal_runtime
                .cancel_resource_request(
                    prior.request_handle(),
                    ResourceCancellationReason::HostRequested,
                )
                .expect("cancellation should succeed")
        })
        .expect("signal runtime should stay on the owning thread");
    let newer = admit_retryable_request_response_identity(
        runtime,
        node,
        prior.basis_binding().truth_view_basis().clone(),
    );
    runtime
        .admit_async_retry_lineage_after_cancellation(
            BridgeAsyncRetryLineageRequest::after_cancellation(
                &prior,
                &cancellation_report,
                &newer,
            ),
        )
        .expect("cancellation retry lineage should admit")
}

pub(crate) fn cancellation_retry_rejection_for_cross_declaration(
    runtime: &RuntimeBridge,
    prior_node: NodeId,
    newer_node: NodeId,
    truth_basis: BridgeAsyncRequestTruthViewBasis,
) -> crate::facade::BridgeAsyncForwardCausalityRejection {
    let prior = admit_retryable_request_response_identity(runtime, prior_node, truth_basis.clone());
    let cancellation_report =
        with_async_request_signal_runtime(runtime.signal_runtime_key, |signal_runtime| {
            signal_runtime
                .cancel_resource_request(
                    prior.request_handle(),
                    ResourceCancellationReason::HostRequested,
                )
                .expect("cancellation should succeed")
        })
        .expect("signal runtime should stay on the owning thread");
    let newer = admit_retryable_request_response_identity_with_ids(
        runtime,
        newer_node,
        truth_basis,
        BridgeAsyncSourceDeclarationIdentity::admit_bridge_owned(
            "bridge-async:request-response-other",
        ),
        BridgeAsyncSourceLegacyDeclarationIdentity::admit_bridge_owned(
            "source:legacy-request-response-other",
        ),
    );
    runtime
        .admit_async_retry_lineage_after_cancellation(
            BridgeAsyncRetryLineageRequest::after_cancellation(
                &prior,
                &cancellation_report,
                &newer,
            ),
        )
        .expect_err("cross-declaration retry lineage should reject")
}

pub(crate) fn admit_retryable_request_response_identity(
    runtime: &RuntimeBridge,
    node: NodeId,
    truth_basis: BridgeAsyncRequestTruthViewBasis,
) -> AdmittedBridgeAsyncRequestIdentity {
    admit_retryable_request_response_identity_with_ids(
        runtime,
        node,
        truth_basis,
        BridgeAsyncSourceDeclarationIdentity::admit_bridge_owned(
            "bridge-async:request-response-retryable",
        ),
        BridgeAsyncSourceLegacyDeclarationIdentity::admit_bridge_owned(
            "source:legacy-request-response-retryable",
        ),
    )
}

fn admit_retryable_request_response_identity_with_ids(
    runtime: &RuntimeBridge,
    node: NodeId,
    truth_basis: BridgeAsyncRequestTruthViewBasis,
    declaration_identity: BridgeAsyncSourceDeclarationIdentity,
    legacy_identity: BridgeAsyncSourceLegacyDeclarationIdentity,
) -> AdmittedBridgeAsyncRequestIdentity {
    let lowered = runtime
        .lower_async_source_declaration(
            &runtime
                .validate_async_source_declaration(retryable_request_response_draft(
                    node,
                    None,
                    declaration_identity,
                    legacy_identity,
                ))
                .expect("retryable request-response declaration should validate"),
        )
        .expect("retryable request-response declaration should lower");
    let binding = runtime.bind_async_request_basis(&lowered, truth_basis);
    let request = BridgeAsyncRequestAdmissionRequest::request_response(&lowered, &binding)
        .expect("retryable request-response admission request should construct");
    runtime
        .admit_async_request_identity(request)
        .expect("retryable request-response identity should admit")
}

fn admit_retryable_timeout_request_response_identity(
    runtime: &RuntimeBridge,
    node: NodeId,
    truth_basis: BridgeAsyncRequestTruthViewBasis,
    timeout_ms: u64,
) -> AdmittedBridgeAsyncRequestIdentity {
    let lowered = runtime
        .lower_async_source_declaration(
            &runtime
                .validate_async_source_declaration(retryable_request_response_draft(
                    node,
                    Some(timeout_ms),
                    BridgeAsyncSourceDeclarationIdentity::admit_bridge_owned(
                        "bridge-async:request-response-retryable",
                    ),
                    BridgeAsyncSourceLegacyDeclarationIdentity::admit_bridge_owned(
                        "source:legacy-request-response-retryable",
                    ),
                ))
                .expect("retryable timeout request-response declaration should validate"),
        )
        .expect("retryable timeout request-response declaration should lower");
    let binding = runtime.bind_async_request_basis(&lowered, truth_basis);
    let request = BridgeAsyncRequestAdmissionRequest::request_response(&lowered, &binding)
        .expect("retryable timeout request-response admission request should construct");
    runtime
        .admit_async_request_identity(request)
        .expect("retryable timeout request-response identity should admit")
}

fn retryable_request_response_draft(
    node: NodeId,
    timeout_ms: Option<u64>,
    declaration_identity: BridgeAsyncSourceDeclarationIdentity,
    legacy_identity: BridgeAsyncSourceLegacyDeclarationIdentity,
) -> BridgeAsyncSourceDeclarationDraft {
    let declaration = ResourceNodeDeclaration::new(
        SignalResourceNodeId::from_node(node),
        ResourcePayloadContract::new(ResourcePayloadContractId::new(142))
            .with_max_payload_bytes(512),
    )
    .with_observation_policy(ResourceObservationPolicyDeclaration::LifecycleOnly)
    .with_retry_policy(ResourceRetryPolicyDeclaration::FixedDelay {
        delay: TemporalDuration::temporal_duration(3).expect("retry delay should validate"),
    })
    .with_retry_max_attempts(3);
    let declaration = match timeout_ms {
        Some(timeout_ms) => declaration.with_timeout_policy(
            ResourceTimeoutPolicyDeclaration::RevalidationEligibleTimeout {
                timeout: TemporalDuration::temporal_duration(timeout_ms)
                    .expect("timeout duration should validate"),
            },
        ),
        None => declaration,
    };
    BridgeAsyncSourceDeclarationDraft::request_response(
        declaration_identity,
        legacy_identity,
        declaration,
    )
}

pub(crate) fn request_response_revalidation_lineage(
    runtime: &RuntimeBridge,
    node: NodeId,
    prior_truth_basis: BridgeAsyncRequestTruthViewBasis,
    current_truth_basis: BridgeAsyncRequestTruthViewBasis,
) -> BridgeAsyncRevalidationLineage {
    let prior = admit_request_response_identity(runtime, node, prior_truth_basis);
    let report = with_async_request_signal_runtime(runtime.signal_runtime_key, |signal_runtime| {
        signal_runtime
            .revalidate_resource_node(ResourceRevalidationIntent::with_expected_active(
                ResourceNodeId::from_node(prior.in_flight_identity().in_flight().node().node()),
                prior.request_handle(),
            ))
            .expect("revalidation should succeed")
    })
    .expect("signal runtime should stay on the owning thread");
    runtime
        .admit_async_revalidation_lineage(BridgeAsyncRevalidationLineageRequest::request_response(
            &prior,
            current_truth_basis,
            &report,
        ))
        .expect("request-response revalidation lineage should admit")
}

pub(crate) fn request_response_revalidation_rejection_for_stale_signal_generation(
    runtime: &RuntimeBridge,
    node: NodeId,
    truth_basis: BridgeAsyncRequestTruthViewBasis,
) -> crate::facade::BridgeAsyncForwardCausalityRejection {
    let prior = admit_request_response_identity(runtime, node, truth_basis.clone());
    let _newer = admit_request_response_identity(runtime, node, truth_basis.clone());
    let report = with_async_request_signal_runtime(runtime.signal_runtime_key, |signal_runtime| {
        signal_runtime
            .revalidate_resource_node(ResourceRevalidationIntent::with_expected_active(
                ResourceNodeId::from_node(prior.in_flight_identity().in_flight().node().node()),
                prior.request_handle(),
            ))
            .expect("stale-handle revalidation should still produce a signal report")
    })
    .expect("signal runtime should stay on the owning thread");
    runtime
        .admit_async_revalidation_lineage(BridgeAsyncRevalidationLineageRequest::request_response(
            &prior,
            truth_basis,
            &report,
        ))
        .expect_err("stale signal generation should reject at bridge forward causality")
}

pub(crate) fn subscription_backed_revalidation_lineage(
    runtime: &RuntimeBridge,
    node: NodeId,
    prior_truth_basis: BridgeAsyncRequestTruthViewBasis,
    prior_subscription_instance: BridgeAsyncRequestSubscriptionInstance,
    current_truth_basis: BridgeAsyncRequestTruthViewBasis,
    current_subscription_instance: BridgeAsyncRequestSubscriptionInstance,
) -> BridgeAsyncRevalidationLineage {
    let prior = admit_subscription_backed_identity(
        runtime,
        node,
        prior_truth_basis,
        prior_subscription_instance,
    )
    .expect("subscription-backed identity should admit");
    let report = with_async_request_signal_runtime(runtime.signal_runtime_key, |signal_runtime| {
        signal_runtime
            .revalidate_resource_node(ResourceRevalidationIntent::with_expected_active(
                ResourceNodeId::from_node(prior.in_flight_identity().in_flight().node().node()),
                prior.request_handle(),
            ))
            .expect("resource-side revalidation should succeed")
    })
    .expect("signal runtime should stay on the owning thread");
    runtime
        .admit_async_revalidation_lineage(
            BridgeAsyncRevalidationLineageRequest::subscription_backed_resource_only(
                &prior,
                current_truth_basis,
                current_subscription_instance,
                &report,
            ),
        )
        .expect("subscription-backed revalidation lineage should admit")
}
