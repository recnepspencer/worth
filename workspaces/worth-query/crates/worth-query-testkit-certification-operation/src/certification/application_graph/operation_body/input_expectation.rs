//! Actual native guards consume retained input and carry graph facts through publication.
pub use super::{
    authenticate_operator, binding, input::SetRetentionInput, install, invoke, invoke_input,
    program, read_retention, request_scope, Outcome, DOCUMENT_IDENTITY,
};
pub use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc,
};
pub use worth_query_host::facade::{
    application_entry::{
        WorthQueryApplicationProgramMutationPreparation as Preparation,
        WorthQueryApplicationRequestExt,
    },
    declaration::application_schema::{
        validate_portable_application_schema_freshly, WorthQueryPortableApplicationSchemaRecord,
    },
    primary_graph::{WorthQueryApplicationUncommitted, WorthQueryOperationBodyDenial},
    runtime::ExecutionAllocationPolicy,
};

pub fn input(value: u64) -> SetRetentionInput {
    SetRetentionInput {
        identity: DOCUMENT_IDENTITY.to_owned(),
        retention_days: value,
        ..Default::default()
    }
}
pub fn body<const DESTINATION: u64>(
) -> worth_query_host::facade::declaration::application_schema::ApplicationOperationBody<
    super::DocumentRetentionSchema<DESTINATION>,
    super::SetRetention,
    SetRetentionInput,
> {
    use super::{
        DocumentAuditSourceField, DocumentDraftNoteField, DocumentRetentionField,
        DocumentRetentionSchema, SetRetention,
    };
    if DESTINATION == 116 {
        return SetRetention::reference::<DocumentRetentionSchema<DESTINATION>>()
            .body()
            .expect_input(DocumentDraftNoteField::reference(), "identity")
            .unwrap()
            .assign_input(DocumentRetentionField::reference(), "retention_days")
            .unwrap()
            .finish()
            .unwrap();
    }
    if matches!(DESTINATION, 110..=115) {
        let operation = SetRetention::reference::<DocumentRetentionSchema<DESTINATION>>();
        let body = operation.body();
        let body = if matches!(DESTINATION, 113 | 114) {
            body.expect_input(DocumentDraftNoteField::reference(), "identity")
        } else if DESTINATION == 115 {
            body.expect_input(DocumentAuditSourceField::reference(), "retention_days")
        } else {
            body.expect_input(
                DocumentRetentionField::reference(),
                match DESTINATION {
                    111 => "identity",
                    112 => "missing",
                    _ => "retention_days",
                },
            )
        }
        .unwrap();
        return body
            .assign(DocumentRetentionField::reference(), 99)
            .unwrap()
            .finish()
            .unwrap();
    }
    unreachable!("only input-guard fixture modes select this authoring recipe")
}
