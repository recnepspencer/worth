//! Enrollment adds a disjoint record only when authored.
use super::{framed_record, operation_body_lifecycle_record, ApplicationOperationBodyProgram};
pub(in crate::application_program) fn operation_body_record(
    body: &ApplicationOperationBodyProgram,
) -> String {
    let record = operation_body_lifecycle_record(body);
    match body.creation().and_then(|creation| creation.enrollment()) {
        Some(enrollment) => framed_record(
            "operation-body-v13",
            [
                ("body", record),
                ("enrollment", enrollment.canonical_record()),
            ],
        ),
        None => record,
    }
}
