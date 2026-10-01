use worth_foundational::facade::{AspectKey, FieldKey, ScalarAspectType};
use worth_query::facade::{foundation, read, runtime};
use worth_runtime_bridge::facade::{
    AdmittedBridgeAsyncCompletion, AdmittedBridgeAsyncRequestIdentity,
    BridgeAsyncRequestAdmissionRequest, BridgeAsyncRequestTruthViewBasis,
    BridgeAsyncSourceDeclarationDraft, BridgeAsyncSourceDeclarationIdentity,
    BridgeAsyncSourceLegacyDeclarationIdentity, BridgeDeliveryReceipt, BridgeMappingId,
    BridgeMappingRegistration, CoarseRoutingMode, CommittedPatchSource, InvalidationSink,
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
    ) -> Result<
        worth_runtime_bridge::facade::BridgeCommittedPatchEnvelope,
        RelationalBridgeSourceError,
    > {
        panic!("QP02 async lifecycle must not contact relational patch IO")
    }
}

impl SnapshotReadSource for UncontactedTruthSource {
    fn open_snapshot(
        &self,
        _identity: &TruthSnapshotIdentity,
    ) -> Result<Box<dyn TruthSnapshotReader>, RelationalBridgeSourceError> {
        panic!("QP02 async lifecycle must not contact snapshot IO")
    }
}

struct RecordingSignalSink;

impl InvalidationSink for RecordingSignalSink {
    fn deliver_invalidation(
        &self,
        delivery: worth_runtime_bridge::facade::BridgeSignalInvalidationDelivery,
        _lease: Option<&worth_runtime_bridge::facade::ExecutionResourceLease<'_>>,
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
            BridgeMappingId::from_stable_name("worth-ui-certification-projection"),
            TruthPatchScope::for_entity_field(
                MappingSelector::any(),
                AspectKey::new("query_text").expect("valid projection aspect"),
                FieldKey::new("status".to_owned()).expect("valid projection field"),
            ),
            SnapshotReadContract::scalar(
                AspectKey::new("query_text").expect("valid projection aspect"),
                ScalarAspectType::String,
            ),
            SignalInvalidationScope::from_stable_name("worth-ui-certification-projection"),
            CoarseRoutingMode::Direct,
        ))
        .build()
        .expect("QP02 projection Bridge must build")
}

pub(crate) fn scalar_async_view(
    workspace: &mut runtime::WorthQueryWorkspace,
    request: &AdmittedBridgeAsyncRequestIdentity,
) -> runtime::WorthQueryLiveView<runtime::WorthQueryUnrefinedLiveShape> {
    scalar_async_view_named(workspace, request, "platform.pulse.status")
}

pub(crate) fn scalar_async_view_named(
    workspace: &mut runtime::WorthQueryWorkspace,
    request: &AdmittedBridgeAsyncRequestIdentity,
    identity: &str,
) -> runtime::WorthQueryLiveView<runtime::WorthQueryUnrefinedLiveShape> {
    workspace
        .declare_bridge_async_live_view(
            identity,
            foundation::DeclarativeLiveQueryRequest::new(
                "WorthUiProjectionText",
                foundation::DeclarativeLiveViewShape::table(),
            )
            .project(
                foundation::DeclarativeProjectionField::new(
                    foundation::AspectFieldKey::from_authoring_parts("identity", "id")
                        .expect("valid identity field"),
                )
                .delivered_as("identity.id"),
            )
            .project(
                foundation::DeclarativeProjectionField::new(
                    foundation::AspectFieldKey::from_authoring_parts("query_text", "status")
                        .expect("valid status field"),
                )
                .delivered_as("query_text.status"),
            ),
            read::QuerySchemaView::new(
                "worth-ui-qp02-async",
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
        .expect("Bridge-backed QP02 view must declare")
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
                    .expect("request-response descriptor retained")
                    .payload_contract_digest()
                    .clone(),
                payload_byte_len,
            ),
        )
        .expect("QP02 completion envelope must validate");
    bridge
        .admit_async_completion(request, &validated)
        .expect("QP02 completion must classify")
        .admitted_completion()
        .expect("QP02 completion must admit")
        .clone()
}

pub(crate) fn admitted_async_request(
    bridge: &RuntimeBridge,
    node: NodeId,
    truth_basis: BridgeAsyncRequestTruthViewBasis,
) -> AdmittedBridgeAsyncRequestIdentity {
    let draft = BridgeAsyncSourceDeclarationDraft::request_response(
        BridgeAsyncSourceDeclarationIdentity::from_stable_name("worth-ui:qp02-source"),
        BridgeAsyncSourceLegacyDeclarationIdentity::from_stable_name("worth-ui:legacy-qp02-source"),
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
                .expect("QP02 source declaration must validate"),
        )
        .expect("QP02 source declaration must lower");
    let binding = bridge.bind_async_request_basis(&lowered, truth_basis);
    bridge
        .admit_async_request_identity(
            BridgeAsyncRequestAdmissionRequest::request_response(&lowered, &binding)
                .expect("QP02 request must construct"),
        )
        .expect("QP02 request must admit")
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
