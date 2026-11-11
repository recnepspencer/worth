//! Real original-memory admission and immutable-template correspondence.
use super::*;
use crate::source::{
    BridgeAsyncSourceDeclarationDraft, BridgeAsyncSourceLegacyDeclarationIdentity,
};
use worth_signal::facade::{
    NodeId, ResourceObservationPolicyDeclaration, ResourcePayloadContract,
    ResourcePayloadContractId, ResourceTimeoutPolicyDeclaration, TemporalDuration,
};

fn declaration(deadline: bool) -> ValidatedBridgeAsyncSourceDeclaration {
    let node = ResourceNodeDeclaration::new(
        ResourceNodeId::from_node(NodeId::new(0, 0)),
        ResourcePayloadContract::new(ResourcePayloadContractId::new(915_601))
            .with_max_payload_bytes(0),
    )
    .with_observation_policy(ResourceObservationPolicyDeclaration::LifecycleOnly);
    let node = if deadline {
        node.with_timeout_policy(
            ResourceTimeoutPolicyDeclaration::TotalRequestLifetimeTimeout {
                timeout: TemporalDuration::temporal_duration(17).unwrap(),
            },
        )
    } else {
        node
    };
    ValidatedBridgeAsyncSourceDeclaration::validate(
        BridgeAsyncSourceDeclarationDraft::request_response(
            BridgeAsyncSourceDeclarationIdentity::from_stable_name("immutable-template"),
            BridgeAsyncSourceLegacyDeclarationIdentity::admit_bridge_owned(
                "immutable-template-legacy",
            ),
            node,
        ),
    )
    .unwrap()
}

#[test]
fn repeated_instances_preserve_fresh_template_canonical_meaning() {
    for deadline in [false, true] {
        let validated = declaration(deadline);
        let template = LoweredBridgeAsyncSourceDeclaration::lower(&validated).unwrap();
        let first = template.instantiate_request_response("first").unwrap();
        let second = first.instantiate_request_response("second").unwrap();
        let third = second.instantiate_request_response("third").unwrap();
        for (identity, instance) in [("first", first), ("second", second), ("third", third)] {
            let fresh = LoweredBridgeAsyncSourceDeclaration::lower(&validated)
                .unwrap()
                .instantiate_request_response(identity)
                .unwrap();
            assert_eq!(instance, fresh);
            assert_eq!(instance.canonical_basis(), fresh.canonical_basis());
            assert_eq!(instance.resource_descriptor(), fresh.resource_descriptor());
            assert!(Arc::ptr_eq(&template.body, &instance.body));
        }
    }
}
