//! A current snapshot is row-owned, not a retained history record.
pub use super::schema::{Row, Schema};
pub use worth_query_decl::facade::application_schema::*;

pub fn declare(
    schema: ApplicationSchemaDeclarationBuilder<Schema>,
) -> ApplicationSchemaDeclarationBuilder<Schema> {
    schema

    // A provider source row is not a global snapshot allocation identity.
}

pub use crate::facade::certification::native_graph_input_guards::schema::storage_model::{
    CurrentSnapshot, Snapshot, SnapshotFacts, SnapshotIdentity, SnapshotKind, SnapshotLabel,
    SnapshotNote, SnapshotValue,
};
