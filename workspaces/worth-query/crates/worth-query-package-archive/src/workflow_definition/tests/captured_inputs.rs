//! Independent wire tags prove profile gating, not executable workflow authority.
use super::*;
use worth_query_declaration::facade::application_program::ApplicationWorkflowDataFlow as Flow;
fn bytes(version: u16, flow: u8) -> Vec<u8> {
    let mut draft = Draft::new(version, 8).terminals(&["a", "b"]);
    draft.output.u32(1);
    draft.output.raw_bytes(&[2]);
    draft.output.text("a");
    draft.output.text("b");
    draft.output.raw_bytes(&[flow]);
    draft.bytes()
}
#[test]
fn captured_tag_requires_its_explicit_profile_and_old_input_tag_keeps_its_meaning() {
    assert_eq!(refused(&bytes(7, 7)), Kind::UnsupportedRecordVariant);
    let captured =
        decode_workflow_definition_draft(&bytes(8, 7), WorthQueryPackageArchiveLimits::DEFAULT)
            .unwrap();
    assert_eq!(
        captured.connections[0].kind,
        DraftConnectionKind::Data(Flow::CapturedOperationInput)
    );
    for version in [7, 8] {
        let original = decode_workflow_definition_draft(
            &bytes(version, 5),
            WorthQueryPackageArchiveLimits::DEFAULT,
        )
        .unwrap();
        assert_eq!(
            original.connections[0].kind,
            DraftConnectionKind::Data(Flow::OperationInput)
        );
        assert_eq!(refused(&bytes(version, 8)), Kind::UnsupportedRecordVariant);
    }
}
