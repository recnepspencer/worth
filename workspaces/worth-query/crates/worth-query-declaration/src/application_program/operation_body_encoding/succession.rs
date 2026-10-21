//! Existing-subject succession is distinct canonical meaning from creation enrollment.
use super::{framed_record, ApplicationOperationBodyProgram};
pub(in crate::application_program) fn operation_body_record(
    body: &ApplicationOperationBodyProgram,
) -> String {
    let record = super::related_source::operation_body_record(body);
    match body.succession() {
        Some(succession) => framed_record(
            "operation-body-v16",
            [
                ("body", record),
                ("succession", succession.canonical_record()),
            ],
        ),
        None => record,
    }
}
