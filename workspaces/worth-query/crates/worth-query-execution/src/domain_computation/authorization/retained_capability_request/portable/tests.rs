use super::*;

#[test]
fn approval_request_round_trip_preserves_every_authorization_axis() {
    let policy = crate::test_request_policy::serial_request();
    let execution = worth_execution::ExecutionRequest::serial(&policy);
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
    let encoded = request.encode_workflow_approval_request().unwrap();
    let restored =
        WorthQueryRetainedCapabilityRequest::decode_workflow_approval_request(&execution, &encoded)
            .unwrap();
    assert!(request.matches_elevated_request(&restored, request.elevation.unwrap()));
    let mut inconsistent =
        WorthQueryRetainedCapabilityRequest::decode_workflow_approval_request(&execution, &encoded)
            .unwrap();
    Arc::get_mut(&mut inconsistent.data).unwrap().related = None;
    assert_eq!(
        inconsistent.encode_workflow_approval_request(),
        Err(WorkflowApprovalRequestEncodingDenial::InvalidRequest)
    );
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
    assert!(nonportable.encode_workflow_approval_request().is_err());
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
