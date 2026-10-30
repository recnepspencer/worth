//! Exact target-binding ownership hostility for migration preparation.

pub use worth_query_host::facade::application_entry::{
    WorthQueryApplicationProgramMigrationPreparationDenial, WorthQueryApplicationRequestExt,
};
pub use worth_query_host::facade::application_installation::WorthQueryProgramOwner;

pub use worth_query_testkit_document_retention::facade::document_retention_model::host::publish_on_first_program;
pub use worth_query_testkit_document_retention::facade::document_retention_model::operator_identity::{
    authenticate_operator, request_scope,
};
pub use worth_query_testkit_document_retention::facade::document_retention_model::programs::RetentionProgramP1;
pub use worth_query_testkit_document_retention::facade::document_retention_model::retention_entry::{
    DocumentRetentionWritten, DocumentRetentionWrittenBinding, SetRetentionDenial,
    SetRetentionDenialBinding, DOCUMENT_IDENTITY,
};
pub use worth_query_testkit_document_retention::facade::document_retention_model::schema::SetRetentionInput;

pub type UnownedMigrationScope =
    worth_query_host::facade::declaration::application_operation::ApplicationMutationFieldScope<
        worth_query_testkit_document_retention::facade::document_retention_model::schema::DocumentRetentionSchema,
        worth_query_testkit_document_retention::facade::document_retention_model::schema::Document,
        worth_query_testkit_document_retention::facade::document_retention_model::schema::DocumentFacts,
        worth_query_testkit_document_retention::facade::document_retention_model::schema::DocumentIdentityField,
        String,
        worth_query_host::facade::declaration::application_schema::ReadOnly,
        worth_query_host::facade::declaration::application_schema::NoApplicationUnit,
    >;

pub struct UnownedMigrationBinding;

impl
    worth_query_host::facade::declaration::application_operation::ApplicationMutationBinding<
        worth_query_testkit_document_retention::facade::document_retention_model::schema::DocumentRetentionSchema,
    > for UnownedMigrationBinding
{
    fn workflow_execution_requirements(
    ) -> worth_query_decl::facade::application_operation::ApplicationWorkflowExecutionRequirements
    {
        worth_query_decl::facade::application_operation::ApplicationWorkflowExecutionRequirements::not_applicable()
    }

    type Input = SetRetentionInput;
    type InputBinding =
        worth_query_testkit_document_retention::facade::document_retention_model::schema::SetRetentionInputBinding;
    type Result = DocumentRetentionWritten;
    type ResultBinding = DocumentRetentionWrittenBinding;
    type IdempotencyKey = u64;
    type Operation = worth_query_testkit_document_retention::facade::document_retention_model::schema::SetRetention;
    type Decision = worth_query_host::facade::primary_graph::WorthQueryInvariantMutationTarget<
        worth_query_testkit_document_retention::facade::document_retention_model::schema::DocumentRetentionSchema,
        worth_query_testkit_document_retention::facade::document_retention_model::schema::Document,
    >;
    type Denial = SetRetentionDenial;
    type DenialBinding = SetRetentionDenialBinding;
    type Output =
        worth_query_host::facade::declaration::application_operation::NoApplicationMutationOutputs;
    type ScopeBinding = UnownedMigrationScope;
    type PrincipalBinding =
        worth_query_testkit_document_retention::facade::document_retention_model::schema::DocumentPrincipalBinding;
    type Mapping = worth_query_testkit_document_retention::facade::document_retention_model::schema::ExternalMapping;
    type Principal = worth_query_testkit_document_retention::facade::document_retention_model::schema::Principal;
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
        worth_query_testkit_document_retention::facade::document_retention_model::schema::DocumentRetentionSchema,
        worth_query_testkit_document_retention::facade::document_retention_model::schema::Document,
        worth_query_testkit_document_retention::facade::document_retention_model::schema::DocumentFacts,
        worth_query_testkit_document_retention::facade::document_retention_model::schema::DocumentIdentityField,
        String,
        worth_query_host::facade::declaration::application_schema::ReadOnly,
        worth_query_host::facade::declaration::application_schema::EqualityPredicate,
        worth_query_host::facade::declaration::application_schema::NoApplicationUnit,
    > {
        worth_query_testkit_document_retention::facade::document_retention_model::schema::DocumentIdentityField::reference()
    }

    fn principal_binding() -> worth_query_host::facade::declaration::application_schema::ApplicationPrincipalBindingRef<
        worth_query_testkit_document_retention::facade::document_retention_model::schema::DocumentRetentionSchema,
        worth_query_testkit_document_retention::facade::document_retention_model::schema::DocumentPrincipalBinding,
        worth_query_testkit_document_retention::facade::document_retention_model::schema::ExternalMapping,
        worth_query_testkit_document_retention::facade::document_retention_model::schema::Principal,
        u64,
        worth_query_host::facade::declaration::application_schema::U64ApplicationValueBinding,
    >{
        worth_query_testkit_document_retention::facade::document_retention_model::schema::DocumentPrincipalBinding::reference()
    }
}

#[derive(Clone)]
pub struct UnownedMigrationIntent {
    pub input: SetRetentionInput,
}

impl
    worth_query_host::facade::declaration::application_operation::ApplicationMutationIntent<
        worth_query_testkit_document_retention::facade::document_retention_model::schema::DocumentRetentionSchema,
    > for UnownedMigrationIntent
{
    type Binding = UnownedMigrationBinding;

    fn input(&self) -> &SetRetentionInput {
        &self.input
    }

    fn scope_binding(&self) -> UnownedMigrationScope {
        UnownedMigrationScope::new(
            worth_query_testkit_document_retention::facade::document_retention_model::schema::DocumentIdentityField::reference(),
            self.input.identity.clone(),
        )
    }
}
