//! Creation meaning belongs to the completed operation, not a creation handler.
use super::super::schema::*;
use worth_query_decl::facade::worth_query_operation_creates;
use worth_query_decl::facade::worth_query_operation_links;
use worth_query_host::facade::{
    declaration::application_schema::*, worth_query_aspect, worth_query_entity, worth_query_field,
    worth_query_operation, worth_query_operation_reads, worth_query_operation_writes,
};

worth_query_entity!(pub Tenant for Schema: BodySchemaBinding);
worth_query_aspect!(pub TenantFacts for Schema: BodySchemaBinding, Tenant;
    identity = AspectIdentity(0x9175B006), revision = AspectContractRevision(1),);
worth_query_field!(pub TenantIdentity for Schema: BodySchemaBinding, Tenant, TenantFacts:
    u64 => U64ApplicationValueBinding, read_only, equality);
worth_query_operation!(pub CreateDocument for Schema: BodySchemaBinding, input super::binding::CreateInputBinding);
worth_query_operation_reads!(CreateDocument => [Tenant, TenantIdentity]);
worth_query_operation_creates!(CreateDocument => [Document]);
pub struct TenantDocuments;
impl TenantDocuments {
    pub fn reference<Schema: BodySchemaBinding>(
    ) -> ApplicationRelationRef<Schema, Self, Tenant, Document> {
        ApplicationRelationRef::from_schema_identifiers(
            "TenantDocuments",
            "Tenant",
            "Document",
            ApplicationRelationIntegrity::same_context_unbounded_retain_dangling(),
        )
    }
}
pub struct DocumentTenant;
impl DocumentTenant {
    pub fn reference<Schema: BodySchemaBinding>(
    ) -> ApplicationRelationRef<Schema, Self, Document, Tenant> {
        ApplicationRelationRef::from_schema_identifiers(
            "DocumentTenant",
            "Document",
            "Tenant",
            ApplicationRelationIntegrity::same_context_unbounded_retain_dangling(),
        )
    }
}
worth_query_operation_links!(CreateDocument => [TenantDocuments, DocumentTenant]);
worth_query_operation_writes!(CreateDocument => [DocumentIdentityField, DocumentRetentionField, DocumentAuditSourceField, DocumentDraftNoteField]);

pub(in super::super) fn declare<const MODE: u64>(
    schema: ApplicationSchemaDeclarationBuilder<DocumentRetentionSchema<MODE>>,
) -> ApplicationSchemaDeclarationBuilder<DocumentRetentionSchema<MODE>> {
    if !matches!(MODE, 30 | 31) {
        return schema;
    }
    let body = CreateDocument::reference()
        .body()
        .in_scope(Tenant::reference())
        .unwrap()
        .create_in_scope(Document::reference(), "document")
        .unwrap()
        .initialize_optional_input(DocumentDraftNoteField::reference(), "note")
        .unwrap()
        .initialize_input(DocumentIdentityField::reference(), "identity")
        .unwrap()
        .initialize_input(DocumentRetentionField::reference(), "retention_days")
        .unwrap()
        .initialize(DocumentAuditSourceField::reference(), 7)
        .unwrap()
        .link_from_scope(TenantDocuments::reference(), "tenant-document")
        .unwrap()
        .link_to_scope(DocumentTenant::reference(), "document-tenant")
        .unwrap();
    let body = if MODE == 31 {
        body.create_output::<super::binding::CreateBinding<MODE>, super::super::output::CreatedOutput<MODE>>().unwrap()
    } else {
        body
    };
    let body = body.finish().unwrap();
    let schema = schema
        .entity(Tenant::reference())
        .aspect(Tenant::reference(), TenantFacts::reference())
        .field(Tenant::reference(), TenantIdentity::reference())
        .relation(
            TenantDocuments::reference(),
            Tenant::reference(),
            Document::reference(),
        )
        .relation(
            DocumentTenant::reference(),
            Document::reference(),
            Tenant::reference(),
        )
        .operation(
            CreateDocument::reference()
                .definition()
                .no_external_effect()
                .no_aftermath()
                .finish()
                .with_body(body)
                .unwrap(),
        )
        .operation_projection_work_budget(CreateDocument::reference(), 16)
        .operation_read_field(CreateDocument::reference(), TenantIdentity::reference())
        .application_mutation_binding::<super::binding::CreateBinding<MODE>>();
    super::super::creation_link::query::declare(schema)
}
