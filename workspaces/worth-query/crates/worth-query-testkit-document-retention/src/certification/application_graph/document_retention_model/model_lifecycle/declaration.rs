use super::super::{schema::*, workflow::WorkflowDocumentOwner};
use worth_query_decl::facade::{
    application_operation::*, application_schema::*, worth_query_mutation_binding,
    worth_query_operation, worth_query_operation_creates, worth_query_operation_deletes,
    worth_query_operation_links, worth_query_operation_reads, worth_query_operation_unlinks,
    worth_query_structured_value_binding,
};

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
pub struct ModelLifecycleInput {
    pub context: String,
    pub identity: String,
    pub restore: bool,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ModelLifecycleChanged;
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ModelLifecycleDenial {
    WrongState,
}
worth_query_structured_value_binding!(pub ModelLifecycleInputBinding for ModelLifecycleInput {
    identity: "worth.query.certification.model-lifecycle-input.v1"
});
worth_query_structured_value_binding!(pub ModelLifecycleChangedBinding for ModelLifecycleChanged {
    identity: "worth.query.certification.model-lifecycle-changed.v1"
});
worth_query_structured_value_binding!(pub ModelLifecycleDenialBinding for ModelLifecycleDenial {
    identity: "worth.query.certification.model-lifecycle-denial.v1"
});
worth_query_operation!(pub ModelLifecycle for DocumentRetentionSchema, input ModelLifecycleInputBinding);
worth_query_operation_reads!(ModelLifecycle => [Document, Principal, WorkflowDocumentOwner]);

worth_query_operation_creates!(ModelLifecycle => [Document]);
worth_query_operation_deletes!(ModelLifecycle => [Document]);
worth_query_operation_links!(ModelLifecycle => [WorkflowDocumentOwner]);
worth_query_operation_unlinks!(ModelLifecycle => [WorkflowDocumentOwner]);

pub fn declare(
    schema: ApplicationSchemaDeclarationBuilder<DocumentRetentionSchema>,
) -> ApplicationSchemaDeclarationBuilder<DocumentRetentionSchema> {
    let operation = ModelLifecycle::reference();
    schema
        .operation(
            operation
                .definition()
                .no_external_effect()
                .no_aftermath()
                .finish(),
        )
        .operation_projection_work_budget(operation, 64)
        .operation_read_entity(operation, Document::reference())
        .operation_read_entity(operation, Principal::reference())
        .operation_read_field(
            operation,
            DocumentIdentityField::at("DocumentIdentityField"),
        )
        .operation_read_field(
            operation,
            DocumentRetentionField::at("DocumentRetentionField"),
        )
        .operation_read_field(
            operation,
            PrincipalIdentityField::at("PrincipalIdentityField"),
        )
        .operation_read_relation(operation, WorkflowDocumentOwner::reference())
        .operation_write(
            operation,
            DocumentIdentityField::at("DocumentIdentityField"),
        )
        .operation_write(
            operation,
            DocumentRetentionField::at("DocumentRetentionField"),
        )
        .operation_create(operation, Document::reference())
        .operation_delete(operation, Document::reference())
        .operation_link(operation, WorkflowDocumentOwner::reference())
        .operation_unlink(operation, WorkflowDocumentOwner::reference())
        .application_mutation_binding::<ModelLifecycleBinding>()
}
fn context(input: &ModelLifecycleInput) -> String {
    input.context.clone()
}
worth_query_mutation_binding!(
    pub ModelLifecycleBinding for ModelLifecycleInput, schema DocumentRetentionSchema,
    identity "worth.query.certification.model-lifecycle-binding.v1", input ModelLifecycleInputBinding,
    operation ModelLifecycle, result ModelLifecycleChangedBinding,
    idempotency u64, identity "worth.query.certification.model-lifecycle-command.v1",
    decision super::handler::Targets, denial ModelLifecycleDenialBinding,
    handler identity "worth.query.certification.model-lifecycle-handler.v1",
    program required,
    workflow_requirements ApplicationWorkflowExecutionRequirements::not_applicable(),
    outputs NoApplicationMutationOutputs,
    principal DocumentPrincipalBinding, mapping ExternalMapping, principal_entity Principal,
        principal_identity u64, identity_binding U64ApplicationValueBinding,
    scope DocumentIdentityField,
    field DocumentIdentityField::at("DocumentIdentityField"), value context,
);
#[derive(Clone)]
pub struct ModelLifecycleIntent {
    pub input: ModelLifecycleInput,
}
impl ApplicationMutationIntent<DocumentRetentionSchema> for ModelLifecycleIntent {
    type Binding = ModelLifecycleBinding;
    fn input(&self) -> &ModelLifecycleInput {
        &self.input
    }
    fn scope_binding(
        &self,
    ) -> <Self::Binding as ApplicationMutationBinding<DocumentRetentionSchema>>::ScopeBinding {
        ApplicationMutationFieldScope::new(
            DocumentIdentityField::at("DocumentIdentityField"),
            self.input.context.clone(),
        )
    }
}
