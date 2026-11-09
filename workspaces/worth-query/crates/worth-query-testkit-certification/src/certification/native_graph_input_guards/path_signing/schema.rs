//! Immutable snapshots retain Subject facts separately from bookkeeping AuditContext.
pub use super::super::schema::{Row, Schema};
pub use worth_query_decl::facade::application_schema::{
    ApplicationSchemaDeclarationBuilder, ApplicationSigningGraphSubjectProvenance,
    ApplicationSigningGraphSubjectProvenanceBinding, ApplicationSigningSubjectProvenance,
    ApplicationSigningSubjectProvenanceBinding, StringApplicationValueBinding,
    U64ApplicationValueBinding,
};

pub fn declare(
    schema: ApplicationSchemaDeclarationBuilder<Schema>,
) -> ApplicationSchemaDeclarationBuilder<Schema> {
    schema
}

pub use crate::facade::certification::native_graph_input_guards::schema::storage_model::{
    GraphActor, GraphEvidence, GraphFacts, GraphHistory, GraphProvenance, ScopeActor,
    ScopeEvidence, ScopeFacts, ScopeHistory, ScopeProvenance, ScopeValue, SignedCompany,
    SignedRevision, SignedValue,
};
