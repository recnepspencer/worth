use worth_query_declaration::facade::application_operation::{
    ApplicationMutationBinding, ApplicationMutationScopeBinding,
};
use worth_query_execution::facade::primary_graph::WorthQueryAdmittedApplicationOperation;
use worth_query_installation::facade::ApplicationSchema;

pub(super) struct AuthorizedMutation<Schema, Binding>
where
    Schema: ApplicationSchema,
    Binding: ApplicationMutationBinding<Schema>,
{
    pub(super) principal_identity: Binding::PrincipalIdentity,
    pub(super) admission: WorthQueryAdmittedApplicationOperation<
        Schema,
        Binding::Operation,
        Binding::Input,
        <<Binding as ApplicationMutationBinding<Schema>>::ScopeBinding as ApplicationMutationScopeBinding<
            Schema,
        >>::Scope,
    >,
}
