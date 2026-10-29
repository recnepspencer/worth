//! Exact target-binding ownership hostility for migration preparation.

pub use worth_query_host::facade::application_entry::{
    WorthQueryApplicationProgramMigrationPreparationDenial, WorthQueryApplicationRequestExt,
};
pub use worth_query_host::facade::application_installation::WorthQueryProgramOwner;

pub use crate::facade::certification::document_retention_model::host::publish_on_first_program;
pub use crate::facade::certification::document_retention_model::operator_identity::{
    authenticate_operator, request_scope,
};
pub use crate::facade::certification::document_retention_model::programs::RetentionProgramP1;
pub use crate::facade::certification::document_retention_model::retention_entry::{
    DocumentRetentionWritten, DocumentRetentionWrittenBinding, SetRetentionDenial,
    SetRetentionDenialBinding, DOCUMENT_IDENTITY,
};
pub use crate::facade::certification::document_retention_model::schema::SetRetentionInput;

pub type UnownedMigrationScope =
    worth_query_host::facade::declaration::application_operation::ApplicationMutationFieldScope<
        crate::facade::certification::document_retention_model::schema::DocumentRetentionSchema,
        crate::facade::certification::document_retention_model::schema::Document,
        crate::facade::certification::document_retention_model::schema::DocumentFacts,
        crate::facade::certification::document_retention_model::schema::DocumentIdentityField,
        String,
        worth_query_host::facade::declaration::application_schema::ReadOnly,
        worth_query_host::facade::declaration::application_schema::NoApplicationUnit,
    >;

pub struct UnownedMigrationBinding;

impl
    worth_query_host::facade::declaration::application_operation::ApplicationMutationBinding<
        crate::facade::certification::document_retention_model::schema::DocumentRetentionSchema,
    > for UnownedMigrationBinding
{
    fn workflow_execution_requirements(
    ) -> worth_query_decl::facade::application_operation::ApplicationWorkflowExecutionRequirements
    {
        worth_query_decl::facade::application_operation::ApplicationWorkflowExecutionRequirements::not_applicable()
    }

    type Input = SetRetentionInput;
    type InputBinding =
        crate::facade::certification::document_retention_model::schema::SetRetentionInputBinding;
    type Result = DocumentRetentionWritten;
    type ResultBinding = DocumentRetentionWrittenBinding;
    type IdempotencyKey = u64;
    type Operation = crate::facade::certification::document_retention_model::schema::SetRetention;
    type Decision = worth_query_host::facade::primary_graph::WorthQueryInvariantMutationTarget<
        crate::facade::certification::document_retention_model::schema::DocumentRetentionSchema,
        crate::facade::certification::document_retention_model::schema::Document,
    >;
    type Denial = SetRetentionDenial;
    type DenialBinding = SetRetentionDenialBinding;
    type Output =
        worth_query_host::facade::declaration::application_operation::NoApplicationMutationOutputs;
    type ScopeBinding = UnownedMigrationScope;
    type PrincipalBinding =
        crate::facade::certification::document_retention_model::schema::DocumentPrincipalBinding;
    type Mapping = crate::facade::certification::document_retention_model::schema::ExternalMapping;
    type Principal = crate::facade::certification::document_retention_model::schema::Principal;
    type PrincipalIdentity = u64;
    type PrincipalIdentityBinding =
        worth_query_host::facade::declaration::application_schema::U64ApplicationValueBinding;
    type SourceExpectation =
        worth_query_host::facade::declaration::application_operation::NoApplicationMutationSource;

    const IDENTITY: &'static str = "worth.query.certification.unowned-migration-binding.v1";
    const HANDLER_IDENTITY: &'static str = "worth.query.certification.unowned-migration-handler.v1";
    const IDEMPOTENCY_IDENTITY: &'static str =
        "worth.query.certification.unowned-migration-command.v1";

    fn scope_field(
    ) -> worth_query_host::facade::declaration::application_schema::ApplicationFieldRef<
        crate::facade::certification::document_retention_model::schema::DocumentRetentionSchema,
        crate::facade::certification::document_retention_model::schema::Document,
        crate::facade::certification::document_retention_model::schema::DocumentFacts,
        crate::facade::certification::document_retention_model::schema::DocumentIdentityField,
        String,
        worth_query_host::facade::declaration::application_schema::ReadOnly,
        worth_query_host::facade::declaration::application_schema::EqualityPredicate,
        worth_query_host::facade::declaration::application_schema::NoApplicationUnit,
    > {
        crate::facade::certification::document_retention_model::schema::DocumentIdentityField::reference()
    }

    fn principal_binding() -> worth_query_host::facade::declaration::application_schema::ApplicationPrincipalBindingRef<
        crate::facade::certification::document_retention_model::schema::DocumentRetentionSchema,
        crate::facade::certification::document_retention_model::schema::DocumentPrincipalBinding,
        crate::facade::certification::document_retention_model::schema::ExternalMapping,
        crate::facade::certification::document_retention_model::schema::Principal,
        u64,
        worth_query_host::facade::declaration::application_schema::U64ApplicationValueBinding,
    >{
        crate::facade::certification::document_retention_model::schema::DocumentPrincipalBinding::reference()
    }
}

#[derive(Clone)]
pub struct UnownedMigrationIntent {
    pub input: SetRetentionInput,
}

impl
    worth_query_host::facade::declaration::application_operation::ApplicationMutationIntent<
        crate::facade::certification::document_retention_model::schema::DocumentRetentionSchema,
    > for UnownedMigrationIntent
{
    type Binding = UnownedMigrationBinding;

    fn input(&self) -> &SetRetentionInput {
        &self.input
    }

    fn scope_binding(&self) -> UnownedMigrationScope {
        UnownedMigrationScope::new(
            crate::facade::certification::document_retention_model::schema::DocumentIdentityField::reference(),
            self.input.identity.clone(),
        )
    }
}
