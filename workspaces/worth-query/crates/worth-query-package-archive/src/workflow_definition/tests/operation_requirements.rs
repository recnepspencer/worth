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
fn current_operation_drafts_distinguish_absent_and_empty_evidence() {
    for (version, encoded, expected) in [
        (4, Some(""), None),
        (
            4,
            Some("1:none"),
            Some(ApplicationWorkflowExecutionRequirements::occurrence_only()),
        ),
    ] {
        let draft = decode_workflow_definition_draft(
            &operation(version, encoded),
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
fn malformed_or_missing_new_operation_evidence_is_not_legacy_absence() {
    for encoded in ["2:none", "1:approval:0:0:", "1:nonex"] {
        assert_eq!(
            refused(&operation(4, Some(encoded))),
            Kind::InvalidRecordShape
        );
    }
    // Version 4 always carries the field; deleting it cannot request legacy semantics.
    let mut truncated = operation(4, None);
    truncated.truncate(truncated.len() - 4);
    assert_eq!(refused(&truncated), Kind::Truncated);
}
