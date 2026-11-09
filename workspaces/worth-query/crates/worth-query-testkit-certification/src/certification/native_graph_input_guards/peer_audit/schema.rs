pub use crate::facade::certification::native_graph_input_guards::child_actions::Detail;
pub use crate::facade::certification::native_graph_input_guards::schema::{Parent, Schema};
pub use worth_query_decl::facade::application_schema::*;

pub fn declare(
    builder: ApplicationSchemaDeclarationBuilder<Schema>,
) -> ApplicationSchemaDeclarationBuilder<Schema> {
    builder
}

pub use crate::facade::certification::native_graph_input_guards::schema::storage_model::{
    Journal, JournalActor, JournalDetails, JournalFacts, JournalName, JournalRevision,
    ParentJournal,
};
