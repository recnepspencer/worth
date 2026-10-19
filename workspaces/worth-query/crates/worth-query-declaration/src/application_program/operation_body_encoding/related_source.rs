//! Related preimages extend executable meaning without changing existing body profiles.
use super::{framed_record, ApplicationOperationBodyProgram};
pub(in crate::application_program) fn operation_body_record(
    body: &ApplicationOperationBodyProgram,
) -> String {
    let record = super::enrollment::operation_body_record(body);
    if body.creation().is_some_and(|creation| {
        creation
            .fields()
            .iter()
            .any(|step| step.related_source().is_some())
    }) {
        framed_record("operation-body-v15", [("body", record)])
    } else {
        record
    }
}
