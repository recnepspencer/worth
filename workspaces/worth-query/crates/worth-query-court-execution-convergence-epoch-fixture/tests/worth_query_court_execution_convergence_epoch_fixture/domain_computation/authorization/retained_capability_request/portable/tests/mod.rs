use worth_query_declaration::facade::application_capability::ApplicationCapabilityRelationBinding;
use std::sync::Arc;
use worth_foundational::facade::AspectValue;
use std::collections::BTreeMap;
use worth_relational::facade::identity::EntityId;
use worth_foundational::facade::InternedString;
use worth_query_execution::court_doors_for_test::domain_computation::authorization::retained_capability_request::portable::MAXIMUM_PORTABLE_BYTES;
use worth_query_execution::court_doors_for_test::domain_computation::authorization::retained_capability_request::portable::WorkflowApprovalRequestEncodingDenial;
use worth_query_execution::court_doors_for_test::domain_computation::authorization::operation_progression::authority_validation::precondition_binding::transition::admitted_operation::capability_admission::preparation::observation::request_resolution::WorthQueryCapabilityContextKey;
use worth_query_declaration::facade::application_capability::WorthQueryPortableApplicationCapabilityRelationBindingParts;
use worth_query_execution::court_doors_for_test::domain_computation::authorization::retained_capability_request::WorthQueryRetainedCapabilityRequest;

#[test]
fn approval_request_round_trip_preserves_every_authorization_axis() {
    use std::num::NonZeroUsize;
    use worth_execution::CancellationToken;
    use worth_execution::ExecutionResourceLease;
    use worth_execution::LeaseRequest;
    use worth_foundational::DeterminismContract;
    use worth_foundational::ExecutionBudget;
    use worth_foundational::ExecutionPosture;
    use worth_foundational::ExecutionRequestPolicy;
    let lease = ExecutionResourceLease::serial(LeaseRequest {
        policy: ExecutionRequestPolicy::new(
            ExecutionPosture::Serial,
            DeterminismContract::CanonicalBitwise,
            // Reuse this actual SDK owner for passive capability-use retention as well.
            ExecutionBudget::new(NonZeroUsize::MIN, None, Some(100_000)),
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
        data: Arc::new(worth_query_execution::court_doors_for_test::domain_computation::authorization::retained_capability_request::RetainedCapabilityRequestData {
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
            context: worth_query_execution::court_doors_for_test::domain_computation::authorization::retained_capability_request::context::RetainedContext::Native(BTreeMap::new()),
        }),
    };
    let context_key = serde_json::from_str::<WorthQueryCapabilityContextKey>(
            r#"{"context":"context","context_type":"context-type","slot":"peer","slot_type":"entity","entity":"subject"}"#,
        )
        .unwrap();
    let worth_query_execution::court_doors_for_test::domain_computation::authorization::retained_capability_request::context::RetainedContext::Native(context) =
        &mut (*Arc::get_mut(&mut (*request.data_mut_for_test())).unwrap().context_mut_for_test())
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
        .encode_workflow_approval_request_for_test(&execution)
        .unwrap();
    let restored =
        WorthQueryRetainedCapabilityRequest::decode_workflow_approval_request_for_test(&execution, &encoded)
            .unwrap();
    assert!(request.matches_elevated_request_for_test(&restored, (*request.elevation_for_test()).unwrap()));
    assert!(
        request.matches_use_for_test(&restored),
        "Native and decoded context have the same use semantics"
    );
    let metadata: serde_json::Value = serde_json::from_str(&encoded).unwrap();
    let different_entity = serde_json::to_value(EntityId::new(
        worth_relational::facade::identity::PartitionId::main(),
        99,
        1,
    ))
    .unwrap();
    // These are alternate serialized request facts, not authorization verdicts.
    // Changing any named purpose/resource dimension must change correspondence.
    for (field, replacement) in [
        ("purpose", serde_json::json!({"UInt64": 6})),
        ("action", serde_json::json!({"UInt64": 6})),
        ("principal", different_entity.clone()),
        ("resource", different_entity.clone()),
        ("elevation", serde_json::Value::Null),
        ("related", different_entity),
        ("resource_entity", serde_json::json!("different-resource")),
        ("field", serde_json::json!({"UInt64": 9})),
        ("magnitude", serde_json::json!({"UInt64": 9})),
        ("cardinality", serde_json::json!(10)),
        ("context_name", serde_json::json!("different-context")),
        ("context_type", serde_json::json!("different-context-type")),
        (
            "capability_identity",
            serde_json::to_value([9u8; 32]).unwrap(),
        ),
    ] {
        let mut changed = metadata.clone();
        changed[field] = replacement;
        let changed = WorthQueryRetainedCapabilityRequest::decode_workflow_approval_request_for_test(
            &execution,
            &changed.to_string(),
        )
        .unwrap();
        assert!(
            !request.matches_use_for_test(&changed),
            "{field} belongs to exact capability-use correspondence"
        );
    }
    let mut changed_context = metadata.clone();
    changed_context["context"][0][1] = serde_json::to_value(EntityId::new(
        worth_relational::facade::identity::PartitionId::main(),
        100,
        2,
    ))
    .unwrap();
    let changed_context = WorthQueryRetainedCapabilityRequest::decode_workflow_approval_request_for_test(
        &execution,
        &changed_context.to_string(),
    )
    .unwrap();
    assert!(
        !request.matches_use_for_test(&changed_context),
        "context membership is part of the original use"
    );
    drop(changed_context);
    let mut changed_relation = metadata.clone();
    changed_relation["related_relation"]["relation"] = serde_json::json!("different-relation");
    let changed_relation = WorthQueryRetainedCapabilityRequest::decode_workflow_approval_request_for_test(
        &execution,
        &changed_relation.to_string(),
    )
    .unwrap();
    assert!(
        !request.matches_use_for_test(&changed_relation),
        "the actual relation binding is part of the original use"
    );
    drop(changed_relation);
    let elevation = EntityId::new(
        worth_relational::facade::identity::PartitionId::main(),
        101,
        2,
    );
    let mut changed_elevation = metadata.clone();
    changed_elevation["elevation"] = serde_json::to_value(elevation).unwrap();
    let changed_elevation = WorthQueryRetainedCapabilityRequest::decode_workflow_approval_request_for_test(
        &execution,
        &changed_elevation.to_string(),
    )
    .unwrap();
    assert!(!request.matches_use_for_test(&changed_elevation));
    assert!(
        request.matches_elevated_request_for_test(&changed_elevation, elevation),
        "factoring retains the separate original approval selector rule"
    );
    drop(changed_elevation);
    let before_use = lease.charged_memory_bytes();
    let allocation =
        worth_query_execution::court_doors_for_test::domain_computation::primary_graph::source_facts::allocation::SourceFactsAllocation::open(&execution).unwrap();
    let use_identity = worth_query_execution::court_doors_for_test::domain_computation::authorization::retained_capability_request::use_identity::WorthQueryCapabilityUseIdentity::retain_request_for_test(
        &restored,
        &execution,
        &allocation,
    )
    .unwrap();
    let retained_charge = lease.charged_memory_bytes();
    assert!(
        retained_charge > before_use,
        "the actual original request pays for extended immutable use custody"
    );
    drop(allocation);
    assert_eq!(lease.charged_memory_bytes(), retained_charge);
    drop(use_identity);
    assert_eq!(
        lease.charged_memory_bytes(),
        before_use,
        "the passive use releases its exact retained payer"
    );
    let retained = restored.clone();
    let charged = lease.charged_memory_bytes();
    assert!(charged > encoded.len() as u64);
    drop(restored);
    assert_eq!(lease.charged_memory_bytes(), charged);
    drop(retained);
    assert_eq!(lease.charged_memory_bytes(), encoded.len() as u64);
    let mut inconsistent =
        WorthQueryRetainedCapabilityRequest::decode_workflow_approval_request_for_test(&execution, &encoded)
            .unwrap();
    (*Arc::get_mut(&mut (*inconsistent.data_mut_for_test())).unwrap().related_mut_for_test()) = None;
    assert!(matches!(
        inconsistent.encode_workflow_approval_request_for_test(&execution),
        Err(WorkflowApprovalRequestEncodingDenial::InvalidRequest)
    ));
    let mut duplicated: serde_json::Value = serde_json::from_str(&encoded).unwrap();
    let context = duplicated["context"].as_array_mut().unwrap();
    let first = context[0].clone();
    context.push(first);
    assert!(
        WorthQueryRetainedCapabilityRequest::decode_workflow_approval_request_for_test(
            &execution,
            &duplicated.to_string()
        )
        .is_err()
    );
    let mut nonportable = request;
    (*Arc::get_mut(&mut (*nonportable.data_mut_for_test())).unwrap().action_mut_for_test()) = AspectValue::String(
        InternedString::Symbol(worth_foundational::facade::Symbol(1)),
    );
    assert!(nonportable
        .encode_workflow_approval_request_for_test(&execution)
        .is_err());
    let allocation =
        worth_query_execution::court_doors_for_test::domain_computation::primary_graph::source_facts::allocation::SourceFactsAllocation::open(&execution).unwrap();
    let use_identity = worth_query_execution::court_doors_for_test::domain_computation::authorization::retained_capability_request::use_identity::WorthQueryCapabilityUseIdentity::retain_request_for_test(
        &nonportable,
        &execution,
        &allocation,
    )
    .unwrap();
    drop(use_identity);
    drop(allocation);
}

#[test]
fn approval_request_rejects_unknown_or_oversized_forms() {
    let policy = worth_query_testkit_execution_primary_graph_application_attempt_2::test_request_policy::serial_request();
    let execution = worth_execution::ExecutionRequest::serial(&policy);
    assert!(
        WorthQueryRetainedCapabilityRequest::decode_workflow_approval_request_for_test(&execution, "{}")
            .is_err()
    );
    assert!(
        WorthQueryRetainedCapabilityRequest::decode_workflow_approval_request_for_test(
            &execution,
            &"x".repeat(MAXIMUM_PORTABLE_BYTES + 1)
        )
        .is_err()
    );
}

#[test]
fn canonical_envelope_native_backing_keeps_its_charge_until_last_clone() {
    use std::num::NonZeroUsize;
    use worth_execution::CancellationToken;
    use worth_execution::ExecutionResourceLease;
    use worth_execution::LeaseRequest;
    use worth_foundational::DeterminismContract;
    use worth_foundational::ExecutionBudget;
    use worth_foundational::ExecutionPosture;
    use worth_foundational::ExecutionRequestPolicy;
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
        let encoded = worth_query_execution::court_doors_for_test::domain_computation::primary_graph::workflow::actor::encoding::encode(
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
    use worth_execution::CancellationSource;
    use worth_execution::ExecutionResourceLease;
    use worth_execution::LeaseRequest;
    use worth_foundational::DeterminismContract;
    use worth_foundational::ExecutionBudget;
    use worth_foundational::ExecutionPosture;
    use worth_foundational::ExecutionRequestPolicy;
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
    let encode =
        worth_query_execution::court_doors_for_test::domain_computation::primary_graph::workflow::actor::encoding::encode;
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
        worth_query_execution::facade::domain_computation::WorthQueryApplicationAttemptDenialKind::Cancelled
    );
    assert_eq!(lease.charged_memory_bytes(), 0);
}
