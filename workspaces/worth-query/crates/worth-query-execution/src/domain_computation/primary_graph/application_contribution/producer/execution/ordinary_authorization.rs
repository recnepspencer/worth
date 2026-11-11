//! Ordinary authorization returns before producer completion and publication.
use super::*;
use crate::domain_computation::primary_graph::{
    SharedSelectedProductOperation, WorthQueryAdmittedApplicationOperation,
    WorthQueryApplicationEntityIdentity, WorthQueryAuthenticatedPrincipal,
    WorthQuerySelectedProductOperation,
};
use worth_query_declaration::facade::application_operation::{
    ApplicationMutationBinding, ApplicationMutationScopeBinding,
};
type Principal<S, B> = WorthQueryAuthenticatedPrincipal<
    S,
    <Operation<S, B> as ApplicationMutationBinding<S>>::Principal,
    <Operation<S, B> as ApplicationMutationBinding<S>>::PrincipalIdentity,
>;
type Admission<S, B> = WorthQueryAdmittedApplicationOperation<S,
    <Operation<S, B> as ApplicationMutationBinding<S>>::Operation,
    <Operation<S, B> as ApplicationMutationBinding<S>>::Input,
    <<Operation<S, B> as ApplicationMutationBinding<S>>::ScopeBinding as ApplicationMutationScopeBinding<S>>::Scope>;

/// The actual selected operation and its freshly admitted request authority.
pub(super) struct OrdinaryProducerAuthorization<'runtime, Schema, Binding>
where
    Schema: ApplicationSchema + 'static,
    Binding: WorthQueryApplicationProducerBinding<Schema>,
{
    pub(super) shared: SharedSelectedProductOperation<'runtime, Schema>,
    pub(super) principal: Principal<Schema, Binding>,
    pub(super) admission: Admission<Schema, Binding>,
    pub(super) scope: WorthQueryApplicationEntityIdentity<
        Schema,
        <<Operation<Schema, Binding> as ApplicationMutationBinding<Schema>>::ScopeBinding as ApplicationMutationScopeBinding<Schema>>::Scope,
    >,
}

#[inline(never)]
pub(super) fn prepare<'runtime, Schema, Binding>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    selected: WorthQuerySelectedProductOperation<'runtime, Schema>,
    installed: &TypedInstalledProducer<Schema, Binding>,
    external_principal: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
    request_scope: &WorthQueryRequestScope,
    input: &<Operation<Schema, Binding> as ApplicationMutationBinding<Schema>>::Input,
) -> Result<OrdinaryProducerAuthorization<'runtime, Schema, Binding>, ProducerExecutionStop>
where
    Schema: ApplicationSchema + 'static,
    Binding: WorthQueryApplicationProducerBinding<Schema>,
    SourceValue<Schema, Binding>:
        crate::domain_computation::primary_graph::WorthQueryApplicationProjection<
            Schema,
            SourceQuery<Schema, Binding>,
        >,
{
    // The producer installed this exact binding once. Ordinary authorization
    // below still checks its current runtime/schema/operation authority; fresh
    // principal and graph-work sessions are issued for this request. Rebuilding
    // a second installed object here split cold and selected descriptor custody.
    let binding = &installed.mutation;
    let principal = selected
        .resolve_authenticated_principal_with_request(
            execution,
            binding.principal_binding(),
            external_principal,
            request_scope,
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .map_err(|error| principal_rejected(Binding::IDENTITY, error))?;
    let (scope_field, scope_value) = <_ as ApplicationMutationInputScope<
        Schema,
        Operation<Schema, Binding>,
    >>::scope_for_binding(input)
    .into_field_parts(principal.principal_identity());
    let scope = selected
        .resolve_entity_with_request(
            execution,
            scope_field,
            scope_value,
            request_scope,
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .map_err(|error| scope_rejected(Binding::IDENTITY, error))?;
    let admission = selected
        .authorize_operation_with_request(
            execution,
            &principal,
            &scope,
            binding.operation(),
            TypedMutationPreconditions::default(),
            request_scope,
        )
        .map_err(|error| request_authority_denied(Binding::IDENTITY, error))?;
    let shared = selected
        .prepare_shared_query_basis()
        .map_err(|(_, stop)| source_readmission::ready_resource_denial(Binding::IDENTITY, stop))?;
    Ok(OrdinaryProducerAuthorization {
        shared,
        principal,
        admission,
        scope,
    })
}
