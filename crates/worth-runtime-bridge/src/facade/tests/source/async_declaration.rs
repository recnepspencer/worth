use super::*;
use crate::source::{
    BridgeAsyncSignalLoweringFamilyKind, BridgeAsyncSourceDeclarationDraft,
    BridgeAsyncSourceDeclarationIdentity, BridgeAsyncSourceLegacyDeclarationIdentity,
};
use worth_signal::facade::{
    AsyncNodeCapabilityDeclaration, AsyncNodePayloadContract, AsyncNodePayloadContractId, NodeId,
    ResourceNodeDeclaration, ResourceNodeId, ResourceObservationPolicyDeclaration,
    ResourcePayloadContract, ResourcePayloadContractId,
};

#[test]
fn equivalent_request_response_declarations_validate_and_lower_identically() {
    let runtime = runtime(BridgeRuntimePolicy::default());
    let first = runtime
        .validate_async_source_declaration(request_response_draft(
            "bridge-async:request-response",
            "source:legacy-request-response",
            ResourceObservationPolicyDeclaration::LifecycleOnly,
            NodeId::new(11, 0),
        ))
        .expect("request-response declaration should validate");
    let second = runtime
        .validate_async_source_declaration(request_response_draft(
            "bridge-async:request-response",
            "source:legacy-request-response",
            ResourceObservationPolicyDeclaration::LifecycleOnly,
            NodeId::new(88, 0),
        ))
        .expect("adapter-local request-response node variation should still validate");

    assert_eq!(first.family_kind(), second.family_kind());
    assert_eq!(first.canonical_basis(), second.canonical_basis());
    assert_eq!(first.digest(), second.digest());
    assert_eq!(first.counters().async_source_declaration_count(), 1);
    assert_eq!(first.counters().request_response_family_count(), 1);

    let lowered_first = runtime
        .lower_async_source_declaration(&first)
        .expect("validated request-response declaration should lower");
    let lowered_second = runtime
        .lower_async_source_declaration(&second)
        .expect("equivalent validated request-response declaration should lower");

    assert_eq!(
        lowered_first.lowering_family_kind(),
        BridgeAsyncSignalLoweringFamilyKind::ResourceDescriptor
    );
    assert_eq!(
        lowered_first.declaration_identity(),
        first.declaration_identity()
    );
    assert_eq!(
        lowered_first.canonical_basis(),
        lowered_second.canonical_basis()
    );
    assert_eq!(lowered_first.digest(), lowered_second.digest());
    assert_eq!(
        lowered_first.lowering_identity(),
        lowered_second.lowering_identity()
    );
    assert_eq!(
        lowered_first
            .counters()
            .signal_resource_descriptor_lowering_count(),
        1
    );
    assert_eq!(
        lowered_first
            .resource_descriptor()
            .map(|descriptor| descriptor.node()),
        lowered_second
            .resource_descriptor()
            .map(|descriptor| descriptor.node())
    );
    assert_eq!(
        lowered_first
            .resource_descriptor()
            .map(|descriptor| descriptor.node()),
        Some(ResourceNodeId::from_node(NodeId::new(0, 0)))
    );
}

#[test]
fn equivalent_subscription_backed_declarations_validate_and_lower_identically() {
    let runtime = runtime(BridgeRuntimePolicy::default());
    let first = runtime
        .validate_async_source_declaration(subscription_backed_draft(
            "bridge-async:subscription-backed",
            "source:legacy-subscription-backed",
            ResourceObservationPolicyDeclaration::LifecycleAndOutput,
            NodeId::new(19, 0),
        ))
        .expect("subscription-backed declaration should validate");
    let second = runtime
        .validate_async_source_declaration(subscription_backed_draft(
            "bridge-async:subscription-backed",
            "source:legacy-subscription-backed",
            ResourceObservationPolicyDeclaration::LifecycleAndOutput,
            NodeId::new(101, 0),
        ))
        .expect("adapter-local subscription-backed node variation should still validate");

    assert_eq!(first.family_kind(), second.family_kind());
    assert_eq!(first.canonical_basis(), second.canonical_basis());
    assert_eq!(first.digest(), second.digest());
    assert_eq!(first.counters().async_source_declaration_count(), 1);
    assert_eq!(first.counters().subscription_backed_family_count(), 1);

    let lowered_first = runtime
        .lower_async_source_declaration(&first)
        .expect("validated subscription-backed declaration should lower");
    let lowered_second = runtime
        .lower_async_source_declaration(&second)
        .expect("equivalent validated subscription-backed declaration should lower");

    assert_eq!(
        lowered_first.lowering_family_kind(),
        BridgeAsyncSignalLoweringFamilyKind::AsyncNodeCapability
    );
    assert_eq!(
        lowered_first.declaration_identity(),
        first.declaration_identity()
    );
    assert_eq!(
        lowered_first.canonical_basis(),
        lowered_second.canonical_basis()
    );
    assert_eq!(lowered_first.digest(), lowered_second.digest());
    assert_eq!(
        lowered_first.lowering_identity(),
        lowered_second.lowering_identity()
    );
    assert_eq!(
        lowered_first
            .counters()
            .signal_async_node_capability_lowering_count(),
        1
    );
    assert_eq!(
        lowered_first
            .async_node_capability_bundle()
            .map(|bundle| bundle.node()),
        lowered_second
            .async_node_capability_bundle()
            .map(|bundle| bundle.node())
    );
    assert_eq!(
        lowered_first
            .async_node_capability_bundle()
            .map(|bundle| bundle.node()),
        Some(NodeId::new(0, 0))
    );
}

#[test]
fn source_family_and_observation_policy_are_independent() {
    let runtime = runtime(BridgeRuntimePolicy::default());
    let visible = ResourceObservationPolicyDeclaration::LifecycleAndOutput;
    let lifecycle = ResourceObservationPolicyDeclaration::LifecycleOnly;
    let request = runtime
        .validate_async_source_declaration(request_response_draft(
            "bridge-async:visible-response",
            "source:visible-response",
            visible.clone(),
            NodeId::new(13, 0),
        ))
        .unwrap();
    let subscription = runtime
        .validate_async_source_declaration(subscription_backed_draft(
            "bridge-async:lifecycle-subscription",
            "source:lifecycle-subscription",
            lifecycle.clone(),
            NodeId::new(23, 0),
        ))
        .unwrap();

    let response = runtime.lower_async_source_declaration(&request).unwrap();
    let subscription = runtime
        .lower_async_source_declaration(&subscription)
        .unwrap();
    assert_eq!(
        response
            .request_response_declaration()
            .unwrap()
            .observation_policy(),
        &visible
    );
    assert_eq!(
        subscription
            .subscription_backed_declaration()
            .unwrap()
            .observation_policy(),
        &lifecycle
    );
}

#[test]
fn async_source_families_produce_distinct_validated_and_lowered_digests() {
    let runtime = runtime(BridgeRuntimePolicy::default());
    let request_response = runtime
        .validate_async_source_declaration(request_response_draft(
            "bridge-async:request-response-distinct",
            "source:legacy-request-response-distinct",
            ResourceObservationPolicyDeclaration::LifecycleOnly,
            NodeId::new(31, 0),
        ))
        .expect("request-response declaration should validate");
    let subscription_backed = runtime
        .validate_async_source_declaration(subscription_backed_draft(
            "bridge-async:subscription-backed-distinct",
            "source:legacy-subscription-backed-distinct",
            ResourceObservationPolicyDeclaration::LifecycleAndOutput,
            NodeId::new(41, 0),
        ))
        .expect("subscription-backed declaration should validate");

    assert_ne!(request_response.digest(), subscription_backed.digest());

    let lowered_request_response = runtime
        .lower_async_source_declaration(&request_response)
        .expect("request-response declaration should lower");
    let lowered_subscription_backed = runtime
        .lower_async_source_declaration(&subscription_backed)
        .expect("subscription-backed declaration should lower");

    assert_ne!(
        lowered_request_response.lowering_family_kind(),
        lowered_subscription_backed.lowering_family_kind()
    );
    assert_ne!(
        lowered_request_response.digest(),
        lowered_subscription_backed.digest()
    );
}

fn request_response_draft(
    declaration_identity: &str,
    legacy_declaration_identity: &str,
    observation_policy: ResourceObservationPolicyDeclaration,
    node: NodeId,
) -> BridgeAsyncSourceDeclarationDraft {
    let declaration = ResourceNodeDeclaration::new(
        ResourceNodeId::from_node(node),
        ResourcePayloadContract::new(ResourcePayloadContractId::new(41))
            .with_max_payload_bytes(512),
    )
    .with_observation_policy(observation_policy)
    .with_retry_max_attempts(3);

    BridgeAsyncSourceDeclarationDraft::request_response(
        BridgeAsyncSourceDeclarationIdentity::admit_bridge_owned(declaration_identity),
        BridgeAsyncSourceLegacyDeclarationIdentity::admit_bridge_owned(legacy_declaration_identity),
        declaration,
    )
}

fn subscription_backed_draft(
    declaration_identity: &str,
    legacy_declaration_identity: &str,
    observation_policy: ResourceObservationPolicyDeclaration,
    node: NodeId,
) -> BridgeAsyncSourceDeclarationDraft {
    let declaration = AsyncNodeCapabilityDeclaration::new(
        node,
        AsyncNodePayloadContract::new(AsyncNodePayloadContractId::new(73))
            .with_max_payload_bytes(256),
    )
    .with_observation_policy(observation_policy)
    .with_retry_max_attempts(2);

    BridgeAsyncSourceDeclarationDraft::subscription_backed(
        BridgeAsyncSourceDeclarationIdentity::admit_bridge_owned(declaration_identity),
        BridgeAsyncSourceLegacyDeclarationIdentity::admit_bridge_owned(legacy_declaration_identity),
        declaration,
    )
}
