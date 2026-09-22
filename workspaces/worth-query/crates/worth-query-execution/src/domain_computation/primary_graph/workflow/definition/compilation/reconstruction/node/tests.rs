use worth_query_declaration::facade::application_program::ApplicationWorkflowEvidenceJoinPolicy;

use super::{decode_kind, WorkflowNodeTag};

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
            None,
            None,
            Some("resource".to_owned()),
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
