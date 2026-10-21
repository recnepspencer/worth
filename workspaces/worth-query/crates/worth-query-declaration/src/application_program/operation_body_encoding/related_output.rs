//! Related output selection is distinct meaning; older output targets keep their records.
use super::{framed_record, ApplicationOperationBodyProgram};
pub(in crate::application_program) fn operation_body_record(
    body: &ApplicationOperationBodyProgram,
) -> String {
    let record = super::succession::operation_body_record(body);
    if body
        .succession()
        .is_some_and(|s| s.output_match().is_some())
    {
        return framed_record("operation-body-v18", [("body", record)]);
    }
    if body.outputs().iter().any(|output| {
        matches!(
            output.target,
            crate::application_schema::ApplicationOperationBodyOutputTarget::Related { .. }
        )
    }) {
        framed_record("operation-body-v17", [("body", record)])
    } else {
        record
    }
}
