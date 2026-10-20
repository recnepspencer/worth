use super::*;

#[test]
fn approval_request_round_trip_preserves_every_authorization_axis() {
    use std::num::NonZeroUsize;
    use worth_execution::{CancellationToken, ExecutionResourceLease, LeaseRequest};
    use worth_foundational::{
        DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
    };
    let lease = ExecutionResourceLease::serial(LeaseRequest {
        policy: ExecutionRequestPolicy::new(
            ExecutionPosture::Serial,
            DeterminismContract::CanonicalBitwise,
            ExecutionBudget::new(NonZeroUsize::MIN, None, Some(1)),
        ),
        deadline: None,
        cancellation: CancellationToken::new(),
    })
    .unwrap();
    let execution = worth_execution::ExecutionRequest::leased(&lease);
    let mut request = WorthQueryRetainedCapabilityRequest {
        recorded_actor: None,
        capability_identity: [7; 32],
        principal: EntityId::new(
            worth_relational::facade::identity::PartitionId::main(),
            1,
            0,
        ),
        resource: EntityId::new(
            worth_relational::facade::identity::PartitionId::main(),
            2,
            0,
        ),
        data: Arc::new(super::super::RetainedCapabilityRequestData {
            _codec_funding: None,
            resource_entity: Arc::from("resource"),
            elevation: Some(EntityId::new(
                worth_relational::facade::identity::PartitionId::main(),
                3,
                0,
            )),
            action: AspectValue::UInt64(4),
            purpose: AspectValue::UInt64(5),
            related_relation: Some(ApplicationCapabilityRelationBinding::from_untrusted_parts(
                WorthQueryPortableApplicationCapabilityRelationBindingParts {
                    relation: "related".into(),
                    from: "resource".into(),
                    to: "subject".into(),
                },
            )),
            related: Some(EntityId::new(
                worth_relational::facade::identity::PartitionId::main(),
                6,
                0,
            )),
            field: Some(AspectValue::UInt64(7)),
            magnitude: Some(AspectValue::UInt64(8)),
            cardinality: 9,
            context_name: Arc::from("context"),
            context_type: Arc::from("context-type"),
            context: super::super::context::RetainedContext::Native(BTreeMap::new()),
        }),
    };
    let context_key = serde_json::from_str::<WorthQueryCapabilityContextKey>(
            r#"{"context":"context","context_type":"context-type","slot":"peer","slot_type":"entity","entity":"subject"}"#,
        )
        .unwrap();
    let super::super::context::RetainedContext::Native(context) =
        &mut Arc::get_mut(&mut request.data).unwrap().context
    else {
        panic!("fixture has native context")
    };
    context.insert(
        context_key,
        EntityId::new(
            worth_relational::facade::identity::PartitionId::main(),
            10,
            0,
        ),
    );
    let encoded = request
        .encode_workflow_approval_request(&execution)
        .unwrap();
    let restored =
        WorthQueryRetainedCapabilityRequest::decode_workflow_approval_request(&execution, &encoded)
            .unwrap();
    assert!(request.matches_elevated_request(&restored, request.elevation.unwrap()));
    let retained = restored.clone();
    let charged = lease.charged_memory_bytes();
    assert!(charged > encoded.len() as u64);
    drop(restored);
    assert_eq!(lease.charged_memory_bytes(), charged);
    drop(retained);
    assert_eq!(lease.charged_memory_bytes(), encoded.len() as u64);
    let mut inconsistent =
        WorthQueryRetainedCapabilityRequest::decode_workflow_approval_request(&execution, &encoded)
            .unwrap();
    Arc::get_mut(&mut inconsistent.data).unwrap().related = None;
    assert!(matches!(
        inconsistent.encode_workflow_approval_request(&execution),
        Err(WorkflowApprovalRequestEncodingDenial::InvalidRequest)
    ));
    let mut duplicated: serde_json::Value = serde_json::from_str(&encoded).unwrap();
    let context = duplicated["context"].as_array_mut().unwrap();
    let first = context[0].clone();
    context.push(first);
    assert!(
        WorthQueryRetainedCapabilityRequest::decode_workflow_approval_request(
            &execution,
            &duplicated.to_string()
        )
        .is_err()
    );
    let mut nonportable = request;
    Arc::get_mut(&mut nonportable.data).unwrap().action = AspectValue::String(
        InternedString::Symbol(worth_foundational::facade::Symbol(1)),
    );
    assert!(nonportable
        .encode_workflow_approval_request(&execution)
        .is_err());
}

#[test]
fn approval_request_rejects_unknown_or_oversized_forms() {
    let policy = crate::test_request_policy::serial_request();
    let execution = worth_execution::ExecutionRequest::serial(&policy);
    assert!(
        WorthQueryRetainedCapabilityRequest::decode_workflow_approval_request(&execution, "{}")
            .is_err()
    );
    assert!(
        WorthQueryRetainedCapabilityRequest::decode_workflow_approval_request(
            &execution,
            &"x".repeat(MAXIMUM_PORTABLE_BYTES + 1)
        )
        .is_err()
    );
}

#[test]
fn canonical_envelope_native_backing_keeps_its_charge_until_last_clone() {
    use std::num::NonZeroUsize;
    use worth_execution::{CancellationToken, ExecutionResourceLease, LeaseRequest};
    use worth_foundational::{
        DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
    };
    let lease = ExecutionResourceLease::serial(LeaseRequest {
        policy: ExecutionRequestPolicy::new(
            ExecutionPosture::Serial,
            DeterminismContract::CanonicalBitwise,
            ExecutionBudget::new(NonZeroUsize::MIN, Some(64), Some(1)),
        ),
        deadline: None,
        cancellation: CancellationToken::new(),
    })
    .unwrap();
    let retained = {
        let execution = worth_execution::ExecutionRequest::leased(&lease);
        let encoded = crate::domain_computation::primary_graph::workflow::actor::encoding::encode(
            &execution,
            &("actor\n", [1u8, 2]),
            MAXIMUM_PORTABLE_BYTES,
        )
        .unwrap();
        assert_eq!(encoded.as_str(), r#"["actor\n",[1,2]]"#);
        assert_eq!(lease.charged_memory_bytes(), encoded.len() as u64);
        let retained = encoded.clone();
        drop(encoded);
        assert_eq!(lease.charged_memory_bytes(), retained.len() as u64);
        retained
    };
    assert_eq!(lease.charged_memory_bytes(), retained.len() as u64);
    drop(retained);
    assert_eq!(lease.charged_memory_bytes(), 0);
}

#[test]
fn envelope_count_bound_precedes_exact_native_payload_refusal() {
    use std::num::NonZeroUsize;
    use worth_execution::{CancellationSource, ExecutionResourceLease, LeaseRequest};
    use worth_foundational::{
        DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
    };
    let cancellation = CancellationSource::new();
    let lease = ExecutionResourceLease::serial(LeaseRequest {
        policy: ExecutionRequestPolicy::new(
            ExecutionPosture::Serial,
            DeterminismContract::CanonicalBitwise,
            ExecutionBudget::new(NonZeroUsize::MIN, Some(0), Some(1)),
        ),
        deadline: None,
        cancellation: cancellation.token(),
    })
    .unwrap();
    let execution = worth_execution::ExecutionRequest::leased(&lease);
    let original = "original";
    let encode = crate::domain_computation::primary_graph::workflow::actor::encoding::encode;
    assert!(matches!(
        encode(&execution, &original, 1),
        Err(WorkflowApprovalRequestEncodingDenial::TooLarge)
    ));
    let denied = encode(&execution, &original, MAXIMUM_PORTABLE_BYTES).unwrap_err();
    let WorkflowApprovalRequestEncodingDenial::Native(cause) = denied else {
        panic!("native allocation refusal required")
    };
    assert_eq!(
        cause.allocation_denial().unwrap().requested_payload_bytes(),
        Some(10)
    );
    assert_eq!(lease.charged_memory_bytes(), 0);
    cancellation.cancel();
    let denied = encode(&execution, &original, MAXIMUM_PORTABLE_BYTES).unwrap_err();
    let WorkflowApprovalRequestEncodingDenial::Native(cause) = denied else {
        panic!("original native cancellation required")
    };
    assert_eq!(
        cause.kind(),
        crate::domain_computation::primary_graph::WorthQueryApplicationAttemptDenialKind::Cancelled
    );
    assert_eq!(lease.charged_memory_bytes(), 0);
}
