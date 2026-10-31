//! The existing ordinary document read and edit form one declared model history.
use super::{
    retention_entry::{DocumentRetentionRead, SetRetentionBinding},
    schema::DocumentRetentionSchema,
};
use worth_query_decl::facade::{
    application_history::{ApplicationHistoryBinding, ApplicationHistoryDefinition},
    application_schema::ApplicationSchemaDeclarationBuilder,
    worth_query_portable_type,
};

pub struct RetentionHistory;
worth_query_portable_type!(RetentionHistory => "worth.query.document-retention.history.v1");

impl ApplicationHistoryBinding<DocumentRetentionSchema> for RetentionHistory {
    type Access = DocumentRetentionRead;
    const IDENTITY: &'static str = "DocumentRetentionHistory";
}

/// A real authored label payload exercises record allocation without replacing
/// READ, action authorization, the business operation or native funding.
pub struct DetailedRetentionHistory;
worth_query_portable_type!(DetailedRetentionHistory => "worth.query.document-retention.detailed-history.v1");
impl ApplicationHistoryBinding<DocumentRetentionSchema> for DetailedRetentionHistory {
    type Access = DocumentRetentionRead;
    const IDENTITY: &'static str = "DetailedDocumentRetentionHistory";
}

pub fn detailed_edit_label() -> String {
    "Set retention ".repeat(4096)
}

pub fn declare(
    schema: ApplicationSchemaDeclarationBuilder<DocumentRetentionSchema>,
) -> ApplicationSchemaDeclarationBuilder<DocumentRetentionSchema> {
    schema.application_history(
        ApplicationHistoryDefinition::<DocumentRetentionSchema, RetentionHistory>::for_binding()
            .operation::<SetRetentionBinding>("Set retention"),
    ).application_history(
        ApplicationHistoryDefinition::<DocumentRetentionSchema, DetailedRetentionHistory>::for_binding()
            .operation::<SetRetentionBinding>(detailed_edit_label()),
    )
}
