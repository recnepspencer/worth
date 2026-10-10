use worth_foundational::facade::{
    AspectKey, AspectLocator, CanonicalFieldPath, FieldKey, LocatorAuthority,
};
use worth_proof::TransitionOutcome;
use worth_runtime_bridge::facade::{
    AdmittedBridgeAsyncCompletion, AdmittedBridgeAsyncRequestIdentity, AdmittedBridgeSubscription,
    AdmittedBridgeTemporalBasis, BridgeAsyncRequestAdmissionRequest,
    BridgeAsyncRequestTruthViewBasis, BridgeAsyncSourceDeclarationDraft,
    BridgeAsyncSourceDeclarationIdentity, BridgeAsyncSourceLegacyDeclarationIdentity,
    BridgeCommittedPatchEnvelope, BridgeCommittedPatchEnvelopeIdentity, BridgeCommittedPatchItem,
    BridgeCommittedPatchTarget, BridgePreviewSessionIdentity, BridgeSubscriptionBasisRequest,
    BridgeSubscriptionDeclarationFamilyKind, BridgeSubscriptionDeliveryIntentClass,
    BridgeSubscriptionPreviewBasisBinding, BridgeTemporalCauseRecord, BridgeTemporalSignalBasis,
    BridgeTemporalSubscriptionFamilyKind, BridgeTemporalTruthViewBasis, BridgeTemporalWakeEvidence,
    NormalizedSubscriptionSliceIntent, RuntimeBridge, SubscriptionSliceKind,
};
use worth_signal::facade::{
    ClockAdvanceOrdinal, ClockDomain, ClockTick, NodeId, ResourceNodeDeclaration,
    ResourceObservationPolicyDeclaration, ResourcePayloadContract, ResourcePayloadContractId,
    TemporalWakeId, WakeOrdinal,
};

use super::*;

pub(in crate::runtime::tests) fn canonical_truth_patch(
    branch_identity: &str,
    snapshot_identity: &str,
    commit_identity: &str,
    patch_identity: &str,
) -> BridgeCommittedPatchEnvelope {
    BridgeCommittedPatchEnvelope::new(
        BridgeCommittedPatchEnvelopeIdentity::new(
            TruthCommitIdentity::from_bridge_harness_label(commit_identity),
            TruthPatchIdentity::from_bridge_harness_label(patch_identity),
            TruthSnapshotIdentity::from_bridge_harness_label(snapshot_identity),
            TruthBranchIdentity::from_bridge_harness_label(branch_identity),
        ),
        vec![BridgeCommittedPatchItem::with_target(
            "entity-1",
            BridgeCommittedPatchTarget::entity_field_path(
                AspectLocator::new(
                    LocatorAuthority::Authoritative,
                    AspectKey::new("profile").expect("valid test aspect key"),
                ),
                CanonicalFieldPath::single(
                    FieldKey::new("name".to_owned()).expect("valid test field key"),
                ),
            ),
        )],
    )
    .expect("mixed-cause truth patch should construct")
}

pub(in crate::runtime::tests) fn authoritative_time_only_cause(
    runtime: &RuntimeBridge,
    resource_request: worth_execution::ExecutionRequest<'_, '_>,
) -> BridgeTemporalCauseRecord {
    let admitted = admitted_detail_subscription(runtime, resource_request);
    let temporal = runtime
        .admit_temporal_subscription(
            &admitted,
            admitted_temporal_basis(BridgeTemporalTruthViewBasis::authoritative(
                TruthBranchIdentity::from_bridge_harness_label("truth-main"),
                TruthCommitIdentity::from_bridge_harness_label("commit-a"),
                TruthSnapshotIdentity::from_bridge_harness_label("snapshot-a"),
            )),
            BridgeTemporalSubscriptionFamilyKind::WakeDriven,
        )
        .expect("temporal subscription should admit");
    let ready = runtime.prepare_temporal_subscription_activation(&temporal);
    let request = runtime.prepare_temporal_wake_routing(&ready);
    runtime
        .route_temporal_wake(&request, None)
        .expect("time-only cause should route")
}

pub(in crate::runtime::tests) fn authoritative_truth_plus_time_cause(
    runtime: &RuntimeBridge,
    truth_patch: &BridgeCommittedPatchEnvelope,
    resource_request: worth_execution::ExecutionRequest<'_, '_>,
) -> BridgeTemporalCauseRecord {
    let admitted = admitted_detail_subscription(runtime, resource_request);
    let temporal = runtime
        .admit_temporal_subscription(
            &admitted,
            admitted_temporal_basis(BridgeTemporalTruthViewBasis::authoritative(
                TruthBranchIdentity::from_bridge_harness_label("truth-main"),
                TruthCommitIdentity::from_bridge_harness_label("commit-a"),
                TruthSnapshotIdentity::from_bridge_harness_label("snapshot-a"),
            )),
            BridgeTemporalSubscriptionFamilyKind::WakeDriven,
        )
        .expect("temporal subscription should admit");
    let ready = runtime.prepare_temporal_subscription_activation(&temporal);
    let request = runtime.prepare_temporal_wake_routing(&ready);
    runtime
        .route_temporal_wake_with_truth_patch(&request, truth_patch, None)
        .expect("truth-plus-time cause should route")
}

pub(in crate::runtime::tests) fn preview_time_only_cause(
    runtime: &RuntimeBridge,
    suffix: &str,
    resource_request: worth_execution::ExecutionRequest<'_, '_>,
) -> BridgeTemporalCauseRecord {
    let admitted = admitted_detail_subscription(runtime, resource_request);
    let preview_basis = admitted_preview_basis_for_truth(
        runtime,
        suffix,
        TruthBranchIdentity::from_bridge_harness_label("truth-preview"),
        TruthSnapshotIdentity::from_bridge_harness_label("snapshot-a"),
    );
    let preview_temporal = runtime
        .admit_preview_temporal_subscription(
            &admitted,
            &preview_basis,
            admitted_temporal_basis(BridgeTemporalTruthViewBasis::authoritative(
                TruthBranchIdentity::from_bridge_harness_label("truth-preview"),
                TruthCommitIdentity::from_bridge_harness_label("commit-preview"),
                TruthSnapshotIdentity::from_bridge_harness_label("snapshot-a"),
            )),
            BridgeTemporalSubscriptionFamilyKind::WakeDriven,
        )
        .expect("preview temporal subscription should admit");
    let ready = runtime.prepare_preview_temporal_subscription_activation(&preview_temporal);
    let request = runtime.prepare_preview_temporal_wake_routing(&ready);
    runtime
        .route_temporal_wake(&request, None)
        .expect("preview time-only cause should route")
}

pub(in crate::runtime::tests) fn admitted_async_completion(
    runtime: &RuntimeBridge,
    node: NodeId,
    truth_basis: BridgeAsyncRequestTruthViewBasis,
    payload_byte_len: u64,
) -> AdmittedBridgeAsyncCompletion {
    admitted_async_request_and_completion(runtime, node, truth_basis, payload_byte_len).1
}

pub(in crate::runtime::tests) fn admitted_async_request_and_completion(
    runtime: &RuntimeBridge,
    node: NodeId,
    truth_basis: BridgeAsyncRequestTruthViewBasis,
    payload_byte_len: u64,
) -> (
    AdmittedBridgeAsyncRequestIdentity,
    AdmittedBridgeAsyncCompletion,
) {
    let request_identity = admitted_async_request(runtime, node, truth_basis);
    let completion =
        admitted_async_completion_for_request(runtime, &request_identity, payload_byte_len);
    (request_identity, completion)
}

pub(in crate::runtime::tests) fn admitted_async_completion_for_request(
    runtime: &RuntimeBridge,
    request_identity: &AdmittedBridgeAsyncRequestIdentity,
    payload_byte_len: u64,
) -> AdmittedBridgeAsyncCompletion {
    let validated = runtime
        .validate_async_completion_envelope(
            request_identity,
            worth_signal::facade::RawCompletionEnvelope::new(
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
            ),
        )
        .expect("request-response completion envelope should validate");
    let completion = runtime
        .admit_async_completion(request_identity, &validated)
        .expect("request-response completion should admit or deny canonically")
        .admitted_completion()
        .expect("completion should admit")
        .clone();
    completion
}

fn admitted_detail_subscription(
    runtime: &RuntimeBridge,
    resource_request: worth_execution::ExecutionRequest<'_, '_>,
) -> AdmittedBridgeSubscription {
    let declaration = runtime
        .declare_subscription(
            BridgeSubscriptionDeclarationFamilyKind::DetailExact,
            vec![NormalizedSubscriptionSliceIntent::try_new_entity_field(
                "entity-1",
                AspectKey::new("profile").expect("valid native subscription aspect key"),
                FieldKey::new("name".to_owned()).expect("valid native subscription field key"),
                SubscriptionSliceKind::SignalField,
            )
            .expect("slice intent should validate")],
            BridgeSubscriptionDeliveryIntentClass::None,
        )
        .expect("subscription declaration should succeed");
    runtime
        .admit_subscription(
            &declaration,
            BridgeSubscriptionBasisRequest::snapshot(
                TruthSnapshotIdentity::from_bridge_harness_label("snapshot-a"),
            ),
            resource_request,
        )
        .expect("bridge subscription should admit")
}

fn admitted_temporal_basis(
    truth_basis: BridgeTemporalTruthViewBasis,
) -> AdmittedBridgeTemporalBasis {
    let signal_basis = BridgeTemporalSignalBasis::new(
        truth_basis.branch_identity().clone(),
        ClockDomain::MonotonicExecution,
        ClockTick::new(5),
        ClockAdvanceOrdinal::new(3),
        None,
    );
    let wake = BridgeTemporalWakeEvidence::new(
        TemporalWakeId::new(11),
        WakeOrdinal::new(7),
        ClockTick::new(5),
    );
    match AdmittedBridgeTemporalBasis::admit(truth_basis, signal_basis, Some(wake)) {
        TransitionOutcome::Success(admitted) => admitted,
        outcome => panic!("expected admitted temporal basis, got {outcome:?}"),
    }
}

fn admitted_preview_basis_for_truth(
    runtime: &RuntimeBridge,
    suffix: &str,
    truth_branch_identity: TruthBranchIdentity,
    snapshot_identity: TruthSnapshotIdentity,
) -> BridgeSubscriptionPreviewBasisBinding {
    let admitted_preview = runtime
        .admit_preview_session(
            BridgePreviewSessionIdentity::from_stable_name(format!("preview-session:{suffix}")),
            preview_declaration_for_truth(suffix, truth_branch_identity, snapshot_identity),
        )
        .expect("preview session should admit");
    let (active_preview, execution_record) =
        runtime.activate_preview_session(admitted_preview, 3, 1, 2);
    runtime
        .admit_subscription_preview_basis(&active_preview, &execution_record)
        .expect("preview basis should admit")
}

fn preview_declaration_for_truth(
    suffix: &str,
    truth_branch_identity: TruthBranchIdentity,
    snapshot_identity: TruthSnapshotIdentity,
) -> worth_runtime_bridge::facade::BridgePreviewSessionDeclaration {
    let selector_branch_identity = truth_branch_identity.clone();
    worth_runtime_bridge::facade::BridgePreviewSessionDeclaration::new(
        worth_runtime_bridge::facade::BridgePreviewSessionDeclarationIdentity::from_stable_name(format!(
            "preview-declaration:{suffix}"
        )),
        worth_runtime_bridge::facade::BridgeRequestKind::Preview,
        worth_runtime_bridge::facade::BridgeSpeculativeBranchBinding::new(
            worth_runtime_bridge::facade::BridgeSpeculativeBranchBindingIdentity::from_stable_name(format!(
                "preview-binding:{suffix}"
            )),
            truth_branch_identity,
            worth_runtime_bridge::facade::BridgeSignalBranchIdentity::from_stable_name(format!(
                "signal-branch:{suffix}"
            )),
        ),
        worth_runtime_bridge::facade::BridgePreviewSessionBasis::new(
            worth_runtime_bridge::facade::BridgeTruthViewSelector::branch_snapshot(
                selector_branch_identity,
                snapshot_identity,
            ),
            worth_runtime_bridge::facade::BridgeSourceCapabilitySet::new(vec![
                worth_runtime_bridge::facade::BridgeSourceCapability::SnapshotRead,
                worth_runtime_bridge::facade::BridgeSourceCapability::BranchRead,
            ]),
            worth_runtime_bridge::facade::BridgePreviewRetainedArtifactSchema::PreviewLifecycleArtifactsV1,
        ),
    )
}

pub(in crate::runtime::tests) fn admitted_async_request(
    runtime: &RuntimeBridge,
    node: NodeId,
    truth_basis: BridgeAsyncRequestTruthViewBasis,
) -> AdmittedBridgeAsyncRequestIdentity {
    let lowered = runtime
        .lower_async_source_declaration(
            &runtime
                .validate_async_source_declaration(request_response_draft(node))
                .expect("request-response declaration should validate"),
        )
        .expect("request-response declaration should lower");
    let binding = runtime.bind_async_request_basis(&lowered, truth_basis);
    let request = BridgeAsyncRequestAdmissionRequest::request_response(&lowered, &binding)
        .expect("request-response admission request should construct");
    runtime
        .admit_async_request_identity(request)
        .expect("request-response identity should admit")
}

fn request_response_draft(node: NodeId) -> BridgeAsyncSourceDeclarationDraft {
    BridgeAsyncSourceDeclarationDraft::request_response(
        BridgeAsyncSourceDeclarationIdentity::from_stable_name("bridge-async:request-response"),
        BridgeAsyncSourceLegacyDeclarationIdentity::from_stable_name(
            "source:legacy-request-response",
        ),
        ResourceNodeDeclaration::new(
            worth_signal::facade::ResourceNodeId::from_node(node),
            ResourcePayloadContract::new(ResourcePayloadContractId::new(142))
                .with_max_payload_bytes(512),
        )
        .with_observation_policy(ResourceObservationPolicyDeclaration::LifecycleOnly),
    )
}
