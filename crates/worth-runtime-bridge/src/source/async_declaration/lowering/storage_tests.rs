//! Real original-memory admission and immutable-template correspondence.
use super::*;
use crate::source::{
    BridgeAsyncSourceDeclarationDraft, BridgeAsyncSourceLegacyDeclarationIdentity,
};
use worth_execution::{CancellationSource, ExecutionRequest, SerialMemoryBudget, SerialRequest};
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
fn original_lowering_charge_lives_until_the_last_shared_template_drops() {
    let budget = SerialMemoryBudget::new(None);
    let cancellation = CancellationSource::new();
    let serial = SerialRequest::from_memory(budget.clone(), cancellation.token(), None);
    let storage = OriginalLoweringStorage::reserve(ExecutionRequest::serial(&serial)).unwrap();
    let bytes = OriginalLoweringStorage::bytes().unwrap();
    assert_eq!(budget.charged_memory_bytes(), bytes);
    let template =
        LoweredBridgeAsyncSourceDeclaration::lower_original(&declaration(true), storage).unwrap();
    let first = template.instantiate_request_response("first").unwrap();
    let second = first.instantiate_request_response("second").unwrap();
    assert!(Arc::ptr_eq(&template.body, &first.body));
    assert!(Arc::ptr_eq(&first.body, &second.body));
    cancellation.cancel();
    drop(serial);
    drop(template);
    drop(first);
    assert_eq!(budget.charged_memory_bytes(), bytes);
    drop(second);
    assert_eq!(budget.charged_memory_bytes(), 0);
}

#[test]
fn original_budget_refuses_lowering_backing_without_retaining_a_charge() {
    let budget = SerialMemoryBudget::new(Some(OriginalLoweringStorage::bytes().unwrap() - 1));
    let serial =
        SerialRequest::from_memory(budget.clone(), CancellationSource::new().token(), None);
    assert!(matches!(
        OriginalLoweringStorage::reserve(ExecutionRequest::serial(&serial)),
        Err(worth_execution::WorkCeilingDenial::Admission(
            worth_execution::LeaseDenial::MemoryExhausted(_)
        ))
    ));
    assert_eq!(budget.charged_memory_bytes(), 0);
}

#[test]
fn cancelled_original_request_refuses_lowering_backing() {
    let budget = SerialMemoryBudget::new(None);
    let cancellation = CancellationSource::new();
    let serial = SerialRequest::from_memory(budget.clone(), cancellation.token(), None);
    cancellation.cancel();
    assert!(matches!(
        OriginalLoweringStorage::reserve(ExecutionRequest::serial(&serial)),
        Err(worth_execution::WorkCeilingDenial::Stopped(
            worth_execution::MapKernelStop::Cancelled
        ))
    ));
    assert_eq!(budget.charged_memory_bytes(), 0);
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
