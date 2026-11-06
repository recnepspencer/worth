//! Creation meaning belongs to the completed operation, not a creation handler.
pub use super::super::schema::*;
pub use worth_query_decl::facade::worth_query_operation_creates;
pub use worth_query_decl::facade::worth_query_operation_links;
pub use worth_query_host::facade::{
    declaration::application_schema::*, worth_query_operation, worth_query_operation_reads,
    worth_query_operation_writes,
};

worth_query_operation!(pub CreateDocument for Schema: BodySchemaBinding, input super::binding::CreateInputBinding);
worth_query_operation_reads!(CreateDocument => [Tenant]);
worth_query_operation_creates!(CreateDocument => [Document]);

worth_query_operation_links!(CreateDocument => [TenantDocuments, DocumentTenant]);

pub fn declare<const MODE: u64>(
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
        .initialize_optional_input(DocumentDraftNoteField::at("DocumentDraftNoteField"), "note")
        .unwrap()
        .initialize_input(
            DocumentIdentityField::at("DocumentIdentityField"),
            "identity",
        )
        .unwrap()
        .initialize_input(
            DocumentRetentionField::at("DocumentRetentionField"),
            "retention_days",
        )
        .unwrap()
        .initialize(DocumentAuditSourceField::at("DocumentAuditSourceField"), 7)
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
        .operation_read_field(
            CreateDocument::reference(),
            TenantIdentity::at("TenantIdentity"),
        )
        .application_mutation_binding::<super::binding::CreateBinding<MODE>>();
    super::super::creation_link::query::declare(schema)
}

pub use crate::facade::certification::application_graph::operation_body::schema::storage_model::{
    DocumentTenant, Tenant, TenantDocuments, TenantFacts, TenantIdentity,
};
