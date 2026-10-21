//! Immutable snapshots retain Subject facts separately from bookkeeping AuditContext.
use super::super::schema::{Row, Schema};
use worth_query_decl::facade::{
    application_schema::{
        ApplicationSchemaDeclarationBuilder, ApplicationSigningGraphSubjectProvenance,
        ApplicationSigningGraphSubjectProvenanceBinding, ApplicationSigningSubjectProvenance,
        ApplicationSigningSubjectProvenanceBinding, StringApplicationValueBinding,
        U64ApplicationValueBinding,
    },
    worth_query_aspect, worth_query_entity, worth_query_field, worth_query_relation,
};
worth_query_entity!(pub GraphEvidence for Schema);
worth_query_entity!(pub ScopeEvidence for Schema);
worth_query_aspect!(pub GraphFacts for Schema,GraphEvidence;identity=AspectIdentity(0x9175D020),revision=AspectContractRevision(1),);
worth_query_aspect!(pub ScopeFacts for Schema,ScopeEvidence;identity=AspectIdentity(0x9175D021),revision=AspectContractRevision(1),);
worth_query_field!(pub SignedValue for Schema,GraphEvidence,GraphFacts:u64=>U64ApplicationValueBinding,read_only,equality);
worth_query_field!(pub SignedRevision for Schema,GraphEvidence,GraphFacts:u64=>U64ApplicationValueBinding,read_only,equality);
worth_query_field!(pub SignedCompany for Schema,GraphEvidence,GraphFacts:optional String=>StringApplicationValueBinding,read_only,equality);
worth_query_field!(pub GraphActor for Schema,GraphEvidence,GraphFacts:u64=>U64ApplicationValueBinding,read_only,equality);
worth_query_field!(pub GraphProvenance for Schema,GraphEvidence,GraphFacts:ApplicationSigningGraphSubjectProvenance=>ApplicationSigningGraphSubjectProvenanceBinding,read_only,equality);
worth_query_field!(pub ScopeValue for Schema,ScopeEvidence,ScopeFacts:u64=>U64ApplicationValueBinding,read_only,equality);
worth_query_field!(pub ScopeActor for Schema,ScopeEvidence,ScopeFacts:u64=>U64ApplicationValueBinding,read_only,equality);
worth_query_field!(pub ScopeProvenance for Schema,ScopeEvidence,ScopeFacts:ApplicationSigningSubjectProvenance=>ApplicationSigningSubjectProvenanceBinding,read_only,equality);
worth_query_relation!(pub GraphHistory in Schema,Row=>GraphEvidence;integrity=same_context_unbounded_retain_dangling);
worth_query_relation!(pub ScopeHistory in Schema,Row=>ScopeEvidence;integrity=same_context_unbounded_retain_dangling);
pub fn declare(
    schema: ApplicationSchemaDeclarationBuilder<Schema>,
) -> ApplicationSchemaDeclarationBuilder<Schema> {
    schema
        .entity(GraphEvidence::reference())
        .aspect(GraphEvidence::reference(), GraphFacts::reference())
        .field(GraphEvidence::reference(), SignedValue::reference())
        .field(GraphEvidence::reference(), SignedRevision::reference())
        .field(GraphEvidence::reference(), SignedCompany::reference())
        .field(GraphEvidence::reference(), GraphActor::reference())
        .field(GraphEvidence::reference(), GraphProvenance::reference())
        .entity(ScopeEvidence::reference())
        .aspect(ScopeEvidence::reference(), ScopeFacts::reference())
        .field(ScopeEvidence::reference(), ScopeValue::reference())
        .field(ScopeEvidence::reference(), ScopeActor::reference())
        .field(ScopeEvidence::reference(), ScopeProvenance::reference())
        .relation(
            GraphHistory::reference(),
            Row::reference(),
            GraphEvidence::reference(),
        )
        .relation(
            ScopeHistory::reference(),
            Row::reference(),
            ScopeEvidence::reference(),
        )
}
