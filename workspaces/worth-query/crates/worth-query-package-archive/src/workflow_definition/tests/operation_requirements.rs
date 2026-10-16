use super::*;
use worth_query_declaration::facade::application_operation::ApplicationWorkflowExecutionRequirements;

fn operation(version: u16, requirements: Option<&str>) -> Vec<u8> {
    let mut draft = Draft::new(version, 8);
    draft.output.u32(1);
    draft.output.raw_bytes(&[OPERATION]);
    for field in ["a", "operation", "input", "binding"] {
        draft.output.text(field);
    }
    draft.output.raw_bytes(&[1]);
    if let Some(requirements) = requirements {
        draft.output.text(requirements);
    }
    draft.output.u32(0);
    draft.bytes()
}

#[test]
fn current_operation_drafts_preserve_explicit_evidence_requirements() {
    // Structural decoding carries explicit applicability; installed authoring
    // separately rejects a guarded node whose declaration is NotApplicable.
    for expected in [
        ApplicationWorkflowExecutionRequirements::not_applicable(),
        ApplicationWorkflowExecutionRequirements::occurrence_only(),
        ApplicationWorkflowExecutionRequirements::Approval {
            capability: "review".to_owned(),
            identity: "review.v1".to_owned(),
        },
    ] {
        let encoded = expected.persistence_identity();
        let draft = decode_workflow_definition_draft(
            &operation(
                WORTH_QUERY_WORKFLOW_DEFINITION_DRAFT_PROTOCOL_VERSION,
                Some(&encoded),
            ),
            WorthQueryPackageArchiveLimits::DEFAULT,
        )
        .unwrap();
        match &draft.nodes[0].member {
            DraftMember::Operation {
                workflow_execution_requirements,
                ..
            } => {
                assert_eq!(workflow_execution_requirements, &expected);
            }
            other => panic!("operation decoded as {other:?}"),
        }
    }
}

#[test]
fn malformed_or_missing_current_operation_evidence_is_not_legacy_absence() {
    for encoded in [
        "",
        "1:none",
        "1:approval:6:9:reviewreview.v1",
        "2:approval:0:0:",
        "2:none",
    ] {
        assert_eq!(
            refused(&operation(
                WORTH_QUERY_WORKFLOW_DEFINITION_DRAFT_PROTOCOL_VERSION,
                Some(encoded)
            )),
            Kind::InvalidRecordShape
        );
    }
    // Current nodes always carry the field; omission cannot request a default.
    let mut truncated = operation(WORTH_QUERY_WORKFLOW_DEFINITION_DRAFT_PROTOCOL_VERSION, None);
    truncated.truncate(truncated.len() - 4);
    assert_eq!(refused(&truncated), Kind::Truncated);
}
