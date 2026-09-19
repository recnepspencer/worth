use super::*;

pub(super) fn authorize_typed<Schema, Binding>(
    runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    external_principal: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
    request_scope: &WorthQueryRequestScope,
    branch: WorthQueryProductBranch,
    input: <Operation<Schema, Binding> as ApplicationMutationBinding<Schema>>::Input,
) -> Result<(), WorthQueryOutputDemandDenial>
where
    Schema: ApplicationSchema + 'static,
    Binding: WorthQueryApplicationProducerBinding<Schema>,
{
    let binding = runtime
        .installed_schema()
        .installed_mutation_binding::<Operation<Schema, Binding>>()
        .map_err(|error| failed(Binding::IDENTITY, error))?;
    let selected = runtime
        .on_branch(branch)
        .select()
        .map_err(|error| failed(Binding::IDENTITY, error))?;
    let principal = selected
        .resolve_authenticated_principal(
            binding.principal_binding(),
            external_principal,
            request_scope,
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .map_err(|error| failed(Binding::IDENTITY, error))?;
    let (scope_field, scope_value) = input
        .scope_binding()
        .into_field_parts(principal.principal_identity());
    let scope = selected
        .resolve_entity(
            scope_field,
            scope_value,
            request_scope,
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .map_err(|error| failed(Binding::IDENTITY, error))?;
    selected
        .authorize_operation(
            &principal,
            &scope,
            binding.operation(),
            TypedMutationPreconditions::default(),
            request_scope,
        )
        .map(|_| ())
        .map_err(|error| failed(Binding::IDENTITY, error))
}
