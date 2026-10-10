use worth_foundational::facade::{AspectKey, FieldKey, ScalarAspectType};
use worth_query::facade::{foundation, read, runtime};
use worth_runtime_bridge::facade::{
    AdmittedBridgeAsyncCompletion, AdmittedBridgeAsyncRequestIdentity,
    BridgeAsyncRequestAdmissionRequest, BridgeAsyncRequestTruthViewBasis,
    BridgeAsyncSourceDeclarationDraft, BridgeAsyncSourceDeclarationIdentity,
    BridgeAsyncSourceLegacyDeclarationIdentity, BridgeDeliveryReceipt, BridgeMappingId,
    BridgeMappingRegistration, BridgeMixedCauseOrderingInput, BridgeMixedCauseOrderingLaneKind,
    BridgeMixedCauseOrderingRequest, CoarseRoutingMode, CommittedPatchSource, InvalidationSink,
    MappingSelector, RelationalBridgeSourceError, RelationalCommittedPatchRequest, RuntimeBridge,
    RuntimeBridgeBuilder, SignalBridgeSinkError, SignalInvalidationScope, SnapshotReadContract,
    SnapshotReadSource, TruthBranchIdentity, TruthCommitIdentity, TruthPatchScope,
    TruthSnapshotIdentity, TruthSnapshotReader,
};
use worth_signal::facade::{
    NodeId, ResourceNodeDeclaration, ResourceObservationPolicyDeclaration, ResourcePayloadContract,
    ResourcePayloadContractId,
};

#[derive(Clone, Debug)]
struct UncontactedTruthSource;

impl CommittedPatchSource for UncontactedTruthSource {
    fn load_committed_patch(
        &self,
        _request: RelationalCommittedPatchRequest,
        _execution: worth_runtime_bridge::facade::ExecutionRequest<'_, '_>,
    ) -> Result<
        worth_runtime_bridge::facade::BridgeCommittedPatchEnvelope,
        RelationalBridgeSourceError,
    > {
        panic!("the async lifecycle fixture must not contact relational patch IO")
    }
}

impl SnapshotReadSource for UncontactedTruthSource {
    fn open_snapshot(
        &self,
        _identity: &TruthSnapshotIdentity,
        _execution: worth_runtime_bridge::facade::ExecutionRequest<'_, '_>,
    ) -> Result<Box<dyn TruthSnapshotReader>, RelationalBridgeSourceError> {
        panic!("the async lifecycle fixture must not contact snapshot IO")
    }
}

struct RecordingSignalSink;

impl InvalidationSink for RecordingSignalSink {
    fn deliver_invalidation(
        &self,
        delivery: worth_runtime_bridge::facade::BridgeSignalInvalidationDelivery,
        _lease: worth_runtime_bridge::facade::ExecutionRequest<'_, '_>,
    ) -> Result<BridgeDeliveryReceipt, SignalBridgeSinkError> {
        Ok(BridgeDeliveryReceipt::new(
            delivery.invalidation_targets().len(),
            delivery.source_snapshot().clone(),
        ))
    }
}

pub(crate) fn projection_bridge() -> RuntimeBridge {
    RuntimeBridgeBuilder::new()
        .with_relational_source(UncontactedTruthSource)
        .with_signal_sink(RecordingSignalSink)
        .register_mapping(BridgeMappingRegistration::new(
            BridgeMappingId::from_stable_name("worth-ui-projection"),
            TruthPatchScope::for_entity_field(
                MappingSelector::any(),
                AspectKey::new("query_text").expect("valid projection aspect"),
                FieldKey::new("status".to_owned()).expect("valid projection field"),
            ),
            SnapshotReadContract::scalar(
                AspectKey::new("query_text").expect("valid projection aspect"),
                ScalarAspectType::String,
            ),
            SignalInvalidationScope::from_stable_name("worth-ui-projection"),
            CoarseRoutingMode::Direct,
        ))
        .build()
        .expect("projection Bridge must build")
}

pub(crate) fn scalar_async_view(
    workspace: &mut runtime::WorthQueryWorkspace,
    request: &AdmittedBridgeAsyncRequestIdentity,
) -> runtime::WorthQueryLiveView<runtime::WorthQueryUnrefinedLiveShape> {
    workspace
        .declare_bridge_async_live_view(
            "platform.pulse.status",
            foundation::DeclarativeLiveQueryRequest::new(
                "WorthUiProjectionText",
                foundation::DeclarativeLiveViewShape::table(),
            )
            .project(
                foundation::DeclarativeProjectionField::new(
                    foundation::AspectFieldKey::from_authoring_parts("identity", "id")
                        .expect("valid identity projection field"),
                )
                .delivered_as("identity.id"),
            )
            .project(
                foundation::DeclarativeProjectionField::new(
                    foundation::AspectFieldKey::from_authoring_parts("query_text", "status")
                        .expect("valid status projection field"),
                )
                .delivered_as("query_text.status"),
            ),
            read::QuerySchemaView::new(
                "worth-ui-scalar-async",
                [
                    read::SchemaFieldView::new(
                        read::AspectName::new("identity").expect("valid identity aspect"),
                        read::FieldName::new("id").expect("valid identity field"),
                        ScalarAspectType::String,
                    ),
                    read::SchemaFieldView::new(
                        read::AspectName::new("query_text").expect("valid text aspect"),
                        read::FieldName::new("status").expect("valid status field"),
                        ScalarAspectType::String,
                    ),
                ],
                [],
            ),
            request,
        )
        .expect("Bridge-backed scalar async view must declare")
}

pub(crate) fn admitted_async_request_and_completion(
    bridge: &RuntimeBridge,
    node: NodeId,
    truth_basis: BridgeAsyncRequestTruthViewBasis,
    payload_byte_len: u64,
) -> (
    AdmittedBridgeAsyncRequestIdentity,
    AdmittedBridgeAsyncCompletion,
) {
    let request = admitted_async_request(bridge, node, truth_basis);
    let completion = admitted_async_completion_for_request(bridge, &request, payload_byte_len);
    (request, completion)
}

pub(crate) fn admitted_async_completion_for_request(
    bridge: &RuntimeBridge,
    request: &AdmittedBridgeAsyncRequestIdentity,
    payload_byte_len: u64,
) -> AdmittedBridgeAsyncCompletion {
    let validated = bridge
        .validate_async_completion_envelope(
            request,
            worth_signal::facade::RawCompletionEnvelope::new(
                request.request_handle().request_id(),
                request.request_handle().generation(),
                request.request_handle().branch_epoch(),
                request.attempt(),
                request
                    .lowered()
                    .resource_descriptor()
                    .expect("request-response identity retains its descriptor")
                    .payload_contract_digest()
                    .clone(),
                payload_byte_len,
            ),
        )
        .expect("async completion envelope must validate");
    bridge
        .admit_async_completion(request, &validated)
        .expect("async completion must classify")
        .admitted_completion()
        .expect("async completion must admit")
        .clone()
}

pub(crate) fn admitted_async_request(
    bridge: &RuntimeBridge,
    node: NodeId,
    truth_basis: BridgeAsyncRequestTruthViewBasis,
) -> AdmittedBridgeAsyncRequestIdentity {
    let draft = BridgeAsyncSourceDeclarationDraft::request_response(
        BridgeAsyncSourceDeclarationIdentity::from_stable_name("worth-ui:projection-source"),
        BridgeAsyncSourceLegacyDeclarationIdentity::from_stable_name(
            "worth-ui:legacy-projection-source",
        ),
        ResourceNodeDeclaration::new(
            worth_signal::facade::ResourceNodeId::from_node(node),
            ResourcePayloadContract::new(ResourcePayloadContractId::new(313))
                .with_max_payload_bytes(512),
        )
        .with_observation_policy(ResourceObservationPolicyDeclaration::LifecycleOnly),
    );
    let lowered = bridge
        .lower_async_source_declaration(
            &bridge
                .validate_async_source_declaration(draft)
                .expect("projection source declaration must validate"),
        )
        .expect("projection source declaration must lower");
    let binding = bridge.bind_async_request_basis(&lowered, truth_basis);
    let request = BridgeAsyncRequestAdmissionRequest::request_response(&lowered, &binding)
        .expect("projection request admission must construct");
    bridge
        .admit_async_request_identity(request)
        .expect("projection request identity must admit")
}

pub(crate) fn authoritative_async_basis(
    commit: &str,
    snapshot: &str,
) -> BridgeAsyncRequestTruthViewBasis {
    BridgeAsyncRequestTruthViewBasis::authoritative(
        TruthBranchIdentity::from_bridge_harness_label("truth-main"),
        TruthCommitIdentity::from_bridge_harness_label(commit),
        TruthSnapshotIdentity::from_bridge_harness_label(snapshot),
    )
}

pub(crate) fn async_ordering(
    input: BridgeMixedCauseOrderingInput,
) -> BridgeMixedCauseOrderingRequest {
    BridgeMixedCauseOrderingRequest::new(
        BridgeMixedCauseOrderingLaneKind::Authoritative,
        vec![input],
    )
}
