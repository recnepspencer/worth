//! Same-name scope replacement uses ordinary declared entity lifecycle effects.
use super::super::schema::*;
use worth_query_decl::facade::{
    application_operation::*, application_schema::*, worth_query_mutation_binding,
    worth_query_operation, worth_query_operation_creates, worth_query_operation_deletes,
    worth_query_operation_reads, worth_query_operation_writes,
    worth_query_structured_value_binding,
};
use worth_query_host::facade::primary_graph::*;

#[derive(Clone, Debug, serde::Serialize)]
pub struct Change {
    pub restore: bool,
}
#[derive(Debug)]
pub struct Changed;
#[derive(Debug)]
pub enum Denial {
    WrongState,
}
worth_query_structured_value_binding!(pub ChangeBinding for Change { identity: "resource-scope.lifecycle-input.v1" });
worth_query_structured_value_binding!(pub ChangedBinding for Changed { identity: "resource-scope.lifecycle-result.v1" });
worth_query_structured_value_binding!(pub DenialBinding for Denial { identity: "resource-scope.lifecycle-denial.v1" });
worth_query_operation!(pub Lifecycle for CapabilitySchema, input ChangeBinding);
worth_query_operation_reads!(Lifecycle => [Principal, PrincipalIdentityField, Document, DocumentIdentityField, DocumentRetentionField]);
worth_query_operation_creates!(Lifecycle => [Document]);
worth_query_operation_deletes!(Lifecycle => [Document]);
worth_query_operation_writes!(Lifecycle => [DocumentIdentityField, DocumentRetentionField]);
pub struct Targets {
    principal: WorthQueryInvariantMutationTarget<CapabilitySchema, Principal>,
    document: Option<WorthQueryInvariantMutationTarget<CapabilitySchema, Document>>,
}
worth_query_mutation_binding!(
    pub Binding for Change, schema CapabilitySchema, identity "resource-scope.lifecycle-binding.v1",
    input ChangeBinding, operation Lifecycle, result ChangedBinding,
    idempotency u64, identity "resource-scope.lifecycle-key.v1", decision Targets,
    denial DenialBinding, handler identity "resource-scope.lifecycle-handler.v1", program required,
    workflow_requirements ApplicationWorkflowExecutionRequirements::not_applicable(),
    outputs NoApplicationMutationOutputs,
    principal DocumentPrincipalBinding, mapping ExternalMapping, principal_entity Principal,
        principal_identity u64, identity_binding U64ApplicationValueBinding,
    scope Principal, PrincipalFacts, PrincipalIdentityField, u64, ReadOnly, NoApplicationUnit,
    principal_field PrincipalIdentityField::reference(),
);
pub struct Handler;
impl OperationHandler<CapabilitySchema, Binding> for Handler {
    fn decide(
        &self,
        input: &Change,
        reader: &mut DecisionReader<'_, '_, '_, CapabilitySchema, Binding>,
    ) -> HandlerResult<Targets, Denial> {
        let mut observe = || -> Result<Option<Targets>, HandlerExecutionDenial> {
            let principal = reader.resolve_entity(PrincipalIdentityField::reference(), 1)?;
            let document = reader.resolve_optional_entity(
                DocumentIdentityField::reference(),
                DOCUMENT_IDENTITY.to_owned(),
            )?;
            if input.restore == document.is_some() {
                return Ok(None);
            }
            if let Some(document) = document.as_ref() {
                reader
                    .reader()
                    .require_decision_entity(document, Document::reference())
                    .map_err(HandlerExecutionDenial::new)?;
                reader.field(document, DocumentRetentionField::reference())?;
            }
            Ok(Some(Targets {
                principal: reader.mutation_target(&principal)?,
                document: document
                    .map(|value| reader.mutation_target(&value))
                    .transpose()?,
            }))
        };
        match observe() {
            Ok(Some(targets)) => HandlerResult::Completed(targets),
            Ok(None) => HandlerResult::DomainDenied(Denial::WrongState),
            Err(cause) => HandlerResult::ExecutionDenied(cause),
        }
    }
    fn build_candidate(
        &self,
        _: &Change,
        targets: Targets,
        writer: &mut CandidateWriter<'_, CapabilitySchema, Binding>,
    ) -> HandlerResult<Changed, Denial> {
        let apply = || -> Result<(), HandlerExecutionDenial> {
            if let Some(document) = targets.document {
                let document = writer
                    .projected_entity(&document)
                    .map_err(HandlerExecutionDenial::new)?;
                writer
                    .delete_entity(Document::reference(), &document)
                    .map_err(HandlerExecutionDenial::new)?;
            } else {
                let principal = writer
                    .projected_entity(&targets.principal)
                    .map_err(HandlerExecutionDenial::new)?;
                let document = writer
                    .create_entity_in_context(
                        &principal,
                        Document::reference(),
                        WorthQueryApplicationEntityKey::new("resource-scope-successor")
                            .map_err(HandlerExecutionDenial::new)?,
                    )
                    .map_err(HandlerExecutionDenial::new)?;
                writer
                    .initialize_field(
                        &document,
                        DocumentIdentityField::reference(),
                        DOCUMENT_IDENTITY.to_owned(),
                    )
                    .map_err(HandlerExecutionDenial::new)?;
                writer
                    .initialize_field(&document, DocumentRetentionField::reference(), 8)
                    .map_err(HandlerExecutionDenial::new)?;
            }
            Ok(())
        };
        match apply() {
            Ok(()) => HandlerResult::Completed(Changed),
            Err(cause) => HandlerResult::ExecutionDenied(cause),
        }
    }
}
pub fn declare(
    schema: ApplicationSchemaDeclarationBuilder<CapabilitySchema>,
) -> ApplicationSchemaDeclarationBuilder<CapabilitySchema> {
    let op = Lifecycle::reference();
    schema
        .operation(op.definition().no_external_effect().no_aftermath().finish())
        .operation_projection_work_budget(op, 64)
        .operation_read_entity(op, Principal::reference())
        .operation_read_entity(op, Document::reference())
        .operation_read_field(op, PrincipalIdentityField::reference())
        .operation_read_field(op, DocumentIdentityField::reference())
        .operation_read_field(op, DocumentRetentionField::reference())
        .operation_create(op, Document::reference())
        .operation_delete(op, Document::reference())
        .operation_write(op, DocumentIdentityField::reference())
        .operation_write(op, DocumentRetentionField::reference())
        .application_mutation_binding::<Binding>()
}
