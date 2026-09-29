use worth_query_declaration::facade::application_program::ApplicationWorkflowEvidenceJoinPolicy;

use super::{decode::InboundFields, decode_kind, WorkflowNodeTag};

#[test]
fn assessment_subject_is_rejected_for_every_non_assessment_shape_that_uses_empty_fields() {
    let shapes = [
        (
            WorkflowNodeTag::Operation,
            "operation",
            Some("operation-input"),
        ),
        (
            WorkflowNodeTag::EvidenceJoin,
            ApplicationWorkflowEvidenceJoinPolicy::AllRequiredPassing.identity(),
            None,
        ),
        (WorkflowNodeTag::Terminal, "", None),
    ];

    for (tag, member, input_type) in shapes {
        let result = decode_kind(
            tag.persisted(),
            member.to_owned(),
            input_type.map(str::to_owned),
            None,
            InboundFields::default(),
            None,
            None,
            Some("resource".to_owned()),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            false,
        );

        assert!(
            result.is_err(),
            "{} accepted assessment subject",
            tag.identity()
        );
    }
}

#[test]
fn inbound_node_requires_its_own_fields_and_rejects_mixed_operation_shape() {
    let contract = serde_json::json!([
        1,
        "worth.query.workflow.remote",
        1,
        "rail",
        1024,
        256,
        8,
        8,
        2048,
        2,
        16,
        1000,
        16,
    ])
    .to_string();
    let inbound = || InboundFields {
        origin: Some("operation".to_owned()),
        contract: Some(contract.clone()),
        wait: Some(0),
    };
    let decode = |input_type, fields| {
        decode_kind(
            WorkflowNodeTag::AwaitInbound.persisted(),
            "remote-effect".to_owned(),
            input_type,
            None,
            fields,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            false,
        )
    };
    assert!(decode(None, inbound()).is_ok());
    assert!(decode(Some("operation-input".to_owned()), inbound()).is_err());
    assert!(decode(
        None,
        InboundFields {
            origin: None,
            ..inbound()
        }
    )
    .is_err());
}
