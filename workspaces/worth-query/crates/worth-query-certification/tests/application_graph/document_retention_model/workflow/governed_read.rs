//! Independent disclosure capabilities in the isolated Document Retention contribution.
use super::super::schema::{
    Document, DocumentIdentityField, DocumentPrincipalBinding, DocumentRetentionSchema,
    ExternalMapping, Principal,
};
use super::declaration::WorkflowAuthoringGrant;
use worth_query_decl::facade::{
    application_capability::*, application_program::*, application_schema::*,
    worth_query_capability, worth_query_capability_context, worth_query_capability_provenance,
    worth_query_operation, worth_query_operation_reads, worth_query_portable_type,
    worth_query_structured_value_binding,
};
#[path = "governed_read/contract.rs"]
mod contract;
#[path = "governed_read/runtime.rs"]
mod runtime;
pub use runtime::{install, GovernedRuntime};
#[cfg(test)]
#[path = "governed_read/court.rs"]
mod court;
#[path = "governed_read/grants.rs"]
mod grants;
pub(in super::super) use grants::seed;

worth_query_capability_context!(pub WorkflowReadContext in DocumentRetentionSchema);
worth_query_capability_provenance!(pub WorkflowReadProvenance in DocumentRetentionSchema);
worth_query_capability!(pub SubjectReadCapability in DocumentRetentionSchema);
worth_query_capability!(pub DefinitionReadCapability in DocumentRetentionSchema);
worth_query_capability!(pub MetadataReadCapability in DocumentRetentionSchema);
worth_query_capability!(pub AuditReadCapability in DocumentRetentionSchema);
#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
pub struct WorkflowReadInput {
    pub document_identity: String,
}
worth_query_portable_type!(WorkflowReadInput => "worth.query.certification.workflow-read-input.v1");
worth_query_structured_value_binding!(pub WorkflowReadInputBinding for WorkflowReadInput {identity:"worth.query.certification.workflow-read-input.v1"});
worth_query_operation!(pub SubjectReadOperation for DocumentRetentionSchema,input WorkflowReadInputBinding);
worth_query_operation!(pub DefinitionReadOperation for DocumentRetentionSchema,input WorkflowReadInputBinding);
worth_query_operation!(pub MetadataReadOperation for DocumentRetentionSchema,input WorkflowReadInputBinding);
worth_query_operation!(pub AuditReadOperation for DocumentRetentionSchema,input WorkflowReadInputBinding);
worth_query_operation_reads!(SubjectReadOperation => [DocumentIdentityField]);
worth_query_operation_reads!(DefinitionReadOperation => [DocumentIdentityField]);
worth_query_operation_reads!(MetadataReadOperation => [DocumentIdentityField]);
worth_query_operation_reads!(AuditReadOperation => [DocumentIdentityField]);
trait ReadCapability: ApplicationCapabilityMarkerIdentity<Schema = DocumentRetentionSchema> {
    const ACTION: &'static str;
}
impl ReadCapability for SubjectReadCapability {
    const ACTION: &'static str = "read-workflow-subject";
}
impl ReadCapability for DefinitionReadCapability {
    const ACTION: &'static str = "enumerate-workflow-definition";
}
impl ReadCapability for MetadataReadCapability {
    const ACTION: &'static str = "read-workflow-definition-metadata";
}
impl ReadCapability for AuditReadCapability {
    const ACTION: &'static str = "audit-workflow-subject";
}
impl<C: ReadCapability> ApplicationCapabilityRequest<DocumentRetentionSchema, C>
    for WorkflowReadInput
{
    type Scope = Document;
    type Context = WorkflowReadContext;
    fn capability_request(
        &self,
    ) -> Result<
        ApplicationCapabilityRequestProjection<
            DocumentRetentionSchema,
            Document,
            WorkflowReadContext,
        >,
        ApplicationCapabilityRequestProjectionDenial,
    > {
        Ok(ApplicationCapabilityRequestProjection::new(
            ApplicationCapabilityEntitySelector::new(
                DocumentIdentityField::reference(),
                encoded(self.document_identity.clone()),
            ),
            encoded(C::ACTION.to_owned()),
            encoded("reviewed-document".to_owned()),
            ApplicationCapabilityRequestContext::new(WorkflowReadContext::reference()),
        ))
    }
}
pub struct SubjectRead;
pub struct DefinitionRead;
pub struct AuditRead;
pub struct MetadataRead;
impl ApplicationWorkflowReadBinding<DocumentRetentionSchema> for SubjectRead {
    const IDENTITY: &'static str = "worth.query.certification.workflow-subject-read.v1";
    type Capability = SubjectReadCapability;
    type Input = WorkflowReadInput;
    type InputBinding = WorkflowReadInputBinding;
    type Operation = SubjectReadOperation;
    type PrincipalBinding = DocumentPrincipalBinding;
    type Mapping = ExternalMapping;
    type Principal = Principal;
    type PrincipalIdentity = u64;
    type PrincipalIdentityBinding = U64ApplicationValueBinding;
    fn principal_binding() -> ApplicationPrincipalBindingRef<
        DocumentRetentionSchema,
        DocumentPrincipalBinding,
        ExternalMapping,
        Principal,
        u64,
        U64ApplicationValueBinding,
    > {
        DocumentPrincipalBinding::reference()
    }
}
impl ApplicationWorkflowReadBinding<DocumentRetentionSchema> for DefinitionRead {
    const IDENTITY: &'static str = "worth.query.certification.workflow-definition-read.v1";
    type Capability = DefinitionReadCapability;
    type Input = WorkflowReadInput;
    type InputBinding = WorkflowReadInputBinding;
    type Operation = DefinitionReadOperation;
    type PrincipalBinding = DocumentPrincipalBinding;
    type Mapping = ExternalMapping;
    type Principal = Principal;
    type PrincipalIdentity = u64;
    type PrincipalIdentityBinding = U64ApplicationValueBinding;
    fn principal_binding() -> ApplicationPrincipalBindingRef<
        DocumentRetentionSchema,
        DocumentPrincipalBinding,
        ExternalMapping,
        Principal,
        u64,
        U64ApplicationValueBinding,
    > {
        DocumentPrincipalBinding::reference()
    }
}
impl ApplicationWorkflowReadBinding<DocumentRetentionSchema> for MetadataRead {
    const IDENTITY: &'static str = "worth.query.certification.workflow-metadata-read.v1";
    type Capability = MetadataReadCapability;
    type Input = WorkflowReadInput;
    type InputBinding = WorkflowReadInputBinding;
    type Operation = MetadataReadOperation;
    type PrincipalBinding = DocumentPrincipalBinding;
    type Mapping = ExternalMapping;
    type Principal = Principal;
    type PrincipalIdentity = u64;
    type PrincipalIdentityBinding = U64ApplicationValueBinding;
    fn principal_binding() -> ApplicationPrincipalBindingRef<
        DocumentRetentionSchema,
        DocumentPrincipalBinding,
        ExternalMapping,
        Principal,
        u64,
        U64ApplicationValueBinding,
    > {
        DocumentPrincipalBinding::reference()
    }
}
impl ApplicationWorkflowReadBinding<DocumentRetentionSchema> for AuditRead {
    const IDENTITY: &'static str = "worth.query.certification.workflow-audit-read.v1";
    type Capability = AuditReadCapability;
    type Input = WorkflowReadInput;
    type InputBinding = WorkflowReadInputBinding;
    type Operation = AuditReadOperation;
    type PrincipalBinding = DocumentPrincipalBinding;
    type Mapping = ExternalMapping;
    type Principal = Principal;
    type PrincipalIdentity = u64;
    type PrincipalIdentityBinding = U64ApplicationValueBinding;
    fn principal_binding() -> ApplicationPrincipalBindingRef<
        DocumentRetentionSchema,
        DocumentPrincipalBinding,
        ExternalMapping,
        Principal,
        u64,
        U64ApplicationValueBinding,
    > {
        DocumentPrincipalBinding::reference()
    }
}

pub(super) fn declare(
    schema: ApplicationSchemaDeclarationBuilder<DocumentRetentionSchema>,
) -> ApplicationSchemaDeclarationBuilder<DocumentRetentionSchema> {
    let schema = schema
        .capability_context(WorkflowReadContext::reference())
        .capability_provenance(WorkflowReadProvenance::reference());
    let op = SubjectReadOperation::reference();
    let schema = schema
        .operation(op.definition().no_external_effect().no_aftermath().finish())
        .operation_projection_work_budget(op, 512)
        .operation_read_field(op, DocumentIdentityField::reference())
        .capability(
            contract::read::<SubjectReadCapability, SubjectReadOperation>(
                SubjectReadCapability::reference(),
                op,
            ),
        );
    let op = DefinitionReadOperation::reference();
    let schema = schema
        .operation(op.definition().no_external_effect().no_aftermath().finish())
        .operation_projection_work_budget(op, 512)
        .operation_read_field(op, DocumentIdentityField::reference())
        .capability(contract::read::<
            DefinitionReadCapability,
            DefinitionReadOperation,
        >(DefinitionReadCapability::reference(), op));
    let op = MetadataReadOperation::reference();
    let schema = schema
        .operation(op.definition().no_external_effect().no_aftermath().finish())
        .operation_projection_work_budget(op, 512)
        .operation_read_field(op, DocumentIdentityField::reference())
        .capability(contract::read::<
            MetadataReadCapability,
            MetadataReadOperation,
        >(MetadataReadCapability::reference(), op));
    let op = AuditReadOperation::reference();
    let schema = schema
        .operation(op.definition().no_external_effect().no_aftermath().finish())
        .operation_projection_work_budget(op, 512)
        .operation_read_field(op, DocumentIdentityField::reference())
        .capability(contract::read::<AuditReadCapability, AuditReadOperation>(
            AuditReadCapability::reference(),
            op,
        ));
    schema
}
fn encoded(value: String) -> ApplicationEncodedScalarValue<StringApplicationValueBinding> {
    ApplicationEncodedScalarValue::try_new(value).expect("finite fixture read text")
}

/// This separate spec alone opens governed disclosure; the original fixture stays disabled.
pub struct GovernedDocumentWorkflow;
impl ApplicationWorkflowSpec for GovernedDocumentWorkflow {
    type Schema = DocumentRetentionSchema;
    const IDENTITY: ApplicationWorkflowSpecIdentity = ApplicationWorkflowSpecIdentity::new(
        "worth.query.certification.governed-document-workflow.v1",
    );
    fn observation_policy() -> ApplicationWorkflowObservationPolicy {
        ApplicationWorkflowObservationPolicy::governed(
            ApplicationWorkflowReadRule::subject::<Self, SubjectRead>(),
            ApplicationWorkflowReadRule::definition::<Self, DefinitionRead>(),
            Some(ApplicationWorkflowReadRule::definition_metadata::<
                Self,
                MetadataRead,
            >()),
            ApplicationWorkflowAuditPolicy::Governed(ApplicationWorkflowReadRule::subject::<
                Self,
                AuditRead,
            >()),
        )
    }
    fn actor_policy() -> ApplicationWorkflowActorPolicy {
        ApplicationWorkflowActorPolicy::for_approval::<Self, super::WorkflowApprovalBinding>(
            ApplicationWorkflowRecordedActorSource::BoundProposal,
        )
    }
}
