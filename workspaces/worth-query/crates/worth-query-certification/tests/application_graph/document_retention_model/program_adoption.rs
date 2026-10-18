//! Real capability authoring and grant custody for program adoption.
use super::retention_entry::{DOCUMENT_IDENTITY, RELATED_DOCUMENT_IDENTITY};
use super::schema::*;
use super::workflow::declaration::*;
use worth_query_decl::facade::{worth_query_capability, worth_query_operation_reads};
use worth_query_host::facade::declaration::{
    application_capability::*, application_program::*, application_schema::*,
};
use worth_query_host::facade::{
    worth_query_operation, worth_query_portable_type, worth_query_structured_value_binding,
};
#[path = "program_adoption/contract.rs"]
mod contract;
#[path = "program_adoption/grant.rs"]
mod grant;
pub use grant::seed;

worth_query_capability!(pub ProgramAdoptionCapability in DocumentRetentionSchema);
#[derive(Clone, serde::Serialize)]
pub struct ProgramAdoptionInput {
    pub source: String,
    pub target: String,
}
worth_query_portable_type!(ProgramAdoptionInput => "worth.query.certification.program-adoption-input.v1");
worth_query_structured_value_binding!(pub ProgramAdoptionInputBinding for ProgramAdoptionInput { identity: "worth.query.certification.program-adoption-input.v1" });
worth_query_operation!(pub ProgramAdoptionOperation for DocumentRetentionSchema, input ProgramAdoptionInputBinding);
worth_query_operation_reads!(ProgramAdoptionOperation => [DocumentIdentityField]);

pub struct ProgramAdoptionBinding;
impl ApplicationProgramAdoptionBinding<DocumentRetentionSchema> for ProgramAdoptionBinding {
    const IDENTITY: &'static str = "worth.query.certification.program-adoption-binding.v1";
    type Capability = ProgramAdoptionCapability;
    type Input = ProgramAdoptionInput;
    type InputBinding = ProgramAdoptionInputBinding;
    type Operation = ProgramAdoptionOperation;
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
    fn input(context: ApplicationProgramAdoptionContext<'_>) -> ProgramAdoptionInput {
        ProgramAdoptionInput {
            source: context.source.to_string(),
            target: context.target.to_string(),
        }
    }
}
impl ApplicationCapabilityRequest<DocumentRetentionSchema, ProgramAdoptionCapability>
    for ProgramAdoptionInput
{
    type Scope = Document;
    type Context = WorkflowDefinitionAuthoringContext;
    fn capability_request(
        &self,
    ) -> Result<
        ApplicationCapabilityRequestProjection<DocumentRetentionSchema, Document, Self::Context>,
        ApplicationCapabilityRequestProjectionDenial,
    > {
        Ok(ApplicationCapabilityRequestProjection::new(
            ApplicationCapabilityEntitySelector::new(
                DocumentIdentityField::reference(),
                encoded(DOCUMENT_IDENTITY),
            ),
            encoded("adopt-program"),
            encoded("reviewed-document"),
            ApplicationCapabilityRequestContext::new(
                WorkflowDefinitionAuthoringContext::reference(),
            ),
        )
        .related_entity(ApplicationCapabilityRelatedEntitySelector::new(
            WorkflowGrantRelated::reference(),
            ApplicationCapabilityEntitySelector::new(
                DocumentIdentityField::reference(),
                encoded(RELATED_DOCUMENT_IDENTITY),
            ),
        )))
    }
}
fn encoded(value: &str) -> ApplicationEncodedScalarValue<StringApplicationValueBinding> {
    ApplicationEncodedScalarValue::try_new(value.to_owned()).unwrap()
}
pub fn declare(
    schema: ApplicationSchemaDeclarationBuilder<DocumentRetentionSchema>,
) -> ApplicationSchemaDeclarationBuilder<DocumentRetentionSchema> {
    schema
        .operation(
            ProgramAdoptionOperation::reference()
                .definition()
                .no_external_effect()
                .no_aftermath()
                .finish(),
        )
        .operation_projection_work_budget(ProgramAdoptionOperation::reference(), 512)
        .operation_read_field(
            ProgramAdoptionOperation::reference(),
            DocumentIdentityField::reference(),
        )
        .capability(contract::contract())
}
