//! Authored retention meaning for this court; canonical P0 is untouched.
pub use crate::document_retention_model::retention_entry::{
    DocumentRetentionWritten, DocumentRetentionWrittenBinding, SetRetentionDenial,
    SetRetentionDenialBinding, DOCUMENT_IDENTITY,
};
pub use crate::document_retention_model::schema::{SetRetentionInput, SetRetentionInputBinding};
use declaration::application_schema::*;
use worth_query_decl::facade::worth_query_mutation_binding;
use worth_query_host::facade::{declaration, *};

worth_query_application! {
    pub SourceOnlyRetentionSchema {
        owner: "source_only_retention_courtroom", version: (1, 0),
        contributions: [SourceOnlyRetentionContribution],
    }
}
worth_query_application_contribution! {
    pub contribution SourceOnlyRetentionContribution in SourceOnlyRetentionSchema {
        identity: "source-only-retention.application.v1",
        members: |schema| {
            schema.entity(ExternalMapping::reference()).entity(Principal::reference()).entity(Document::reference())
                .aspect(ExternalMapping::reference(), ExternalIdentity::reference())
                .aspect(Principal::reference(), PrincipalFacts::reference()).aspect(Document::reference(), DocumentFacts::reference())
                .field(ExternalMapping::reference(), ExternalIdentityField::reference()).field(ExternalMapping::reference(), MappingStatusField::reference())
                .field(Principal::reference(), PrincipalIdentityField::reference())
                .field(Document::reference(), DocumentIdentityField::reference()).field(Document::reference(), DocumentRetentionField::reference())
                .relation(MappingTarget::reference(), ExternalMapping::reference(), Principal::reference())
                .principal_binding(DocumentPrincipalBinding::reference())
                .operation(SetRetention::reference().definition().no_external_effect().no_aftermath().finish())
                .operation_projection_work_budget(SetRetention::reference(), 8)
                .operation_read_field(SetRetention::reference(), DocumentIdentityField::reference())
                .operation_read_field(SetRetention::reference(), DocumentRetentionField::reference())
                .operation_write(SetRetention::reference(), DocumentRetentionField::reference())
                .operation_expected_fact(SetRetention::reference(), DocumentIdentityField::reference())
                .operation_expected_fact(SetRetention::reference(), DocumentRetentionField::reference())
                .application_mutation_binding::<SetRetentionBinding>().application_query_binding::<super::readback::ReadBinding>()
                .application_query(super::readback::definition())
        }
    }
}
worth_query_entity!(pub ExternalMapping for SourceOnlyRetentionSchema);
worth_query_entity!(pub Principal for SourceOnlyRetentionSchema);
worth_query_entity!(pub Document for SourceOnlyRetentionSchema);
worth_query_aspect!(pub ExternalIdentity for SourceOnlyRetentionSchema, ExternalMapping; identity = AspectIdentity(0x91764001), revision = AspectContractRevision(1));
worth_query_aspect!(pub PrincipalFacts for SourceOnlyRetentionSchema, Principal; identity = AspectIdentity(0x91764002), revision = AspectContractRevision(1));
worth_query_aspect!(pub DocumentFacts for SourceOnlyRetentionSchema, Document; identity = AspectIdentity(0x91764003), revision = AspectContractRevision(1));
worth_query_field!(pub ExternalIdentityField for SourceOnlyRetentionSchema, ExternalMapping, ExternalIdentity: declaration::authentication::WorthQueryExternalPrincipalIdentity => declaration::authentication::WorthQueryExternalPrincipalIdentityBinding, read_only, equality);
worth_query_field!(pub MappingStatusField for SourceOnlyRetentionSchema, ExternalMapping, ExternalIdentity: declaration::authentication::WorthQueryPrincipalMappingStatus => declaration::authentication::WorthQueryPrincipalMappingStatusBinding, read_write, equality);
worth_query_field!(pub PrincipalIdentityField for SourceOnlyRetentionSchema, Principal, PrincipalFacts: u64 => declaration::application_schema::U64ApplicationValueBinding, read_only, equality);
worth_query_field!(pub DocumentIdentityField for SourceOnlyRetentionSchema, Document, DocumentFacts: String => declaration::application_schema::StringApplicationValueBinding, read_only, equality);
worth_query_field!(pub DocumentRetentionField for SourceOnlyRetentionSchema, Document, DocumentFacts: u64 => declaration::application_schema::U64ApplicationValueBinding, read_write, equality);
worth_query_relation!(pub MappingTarget in SourceOnlyRetentionSchema, ExternalMapping => Principal; integrity = same_context_unbounded_retain_dangling);
worth_query_principal_binding!(pub DocumentPrincipalBinding in SourceOnlyRetentionSchema, mapping ExternalMapping { identity: ExternalIdentityField, status: MappingStatusField, target: MappingTarget => Principal, principal_identity: PrincipalIdentityField });
worth_query_operation!(pub SetRetention for SourceOnlyRetentionSchema, input SetRetentionInputBinding);
worth_query_operation_reads!(SetRetention => [DocumentIdentityField, DocumentRetentionField]);
worth_query_operation_writes!(SetRetention => [DocumentRetentionField]);
impl OperationExpectsFact<SetRetention> for DocumentIdentityField {}
impl OperationExpectsFact<SetRetention> for DocumentRetentionField {}

fn document_identity(input: &SetRetentionInput) -> String {
    input.identity.clone()
}
worth_query_mutation_binding!(
    pub SetRetentionBinding for SetRetentionInput, schema SourceOnlyRetentionSchema,
    identity "source-only-retention.set.v1", input SetRetentionInputBinding,
    operation SetRetention, result DocumentRetentionWrittenBinding,
    idempotency u64, identity "source-only-retention.key.v1",
    decision primary_graph::WorthQueryInvariantMutationTarget<SourceOnlyRetentionSchema, Document>, denial SetRetentionDenialBinding,
    handler identity "source-only-retention.handler.v1",
    outputs declaration::application_operation::NoApplicationMutationOutputs,
    principal DocumentPrincipalBinding, mapping ExternalMapping, principal_entity Principal,
        principal_identity u64, identity_binding U64ApplicationValueBinding,
    scope Document, DocumentFacts, DocumentIdentityField, String, ReadOnly, NoApplicationUnit,
    field DocumentIdentityField::reference(), value document_identity,
);
