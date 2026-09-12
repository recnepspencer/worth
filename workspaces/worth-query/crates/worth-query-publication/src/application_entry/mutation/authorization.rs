use worth_query_declaration::facade::{
    application_operation::{
        ApplicationMutationBinding, ApplicationMutationIntent, ApplicationMutationScopeBinding,
        ApplicationMutationScopeResolution,
    },
    application_schema::TypedMutationPreconditions,
};
use worth_query_execution::facade::primary_graph::{
    WorthQueryAdmittedApplicationOperation, WorthQueryApplicationIdempotencyBinding,
    WorthQueryPrincipalResolutionMode,
};
use worth_query_installation::facade::ApplicationSchema;

use super::WorthQueryApplicationMutationRequestWithIdempotency;
use crate::application_entry::WorthQueryApplicationRequestMutationDenial;

type IntentBinding<Schema, Intent> = <Intent as ApplicationMutationIntent<Schema>>::Binding;
type MutationScope<Schema, Binding> =
    <<Binding as ApplicationMutationBinding<Schema>>::ScopeBinding as ApplicationMutationScopeBinding<
        Schema,
    >>::Scope;

pub(super) struct PreparedMutation<Schema, Binding>
where
    Schema: ApplicationSchema,
    Binding: ApplicationMutationBinding<Schema>,
{
    pub(super) admission: WorthQueryAdmittedApplicationOperation<
        Schema,
        Binding::Operation,
        Binding::Input,
        MutationScope<Schema, Binding>,
    >,
    pub(super) idempotency: WorthQueryApplicationIdempotencyBinding,
}

pub(super) fn prepare<Schema, Intent>(
    request: &WorthQueryApplicationMutationRequestWithIdempotency<'_, '_, '_, '_, Schema, Intent>,
) -> Result<PreparedMutation<Schema, IntentBinding<Schema, Intent>>, WorthQueryApplicationRequestMutationDenial>
where
    Schema: ApplicationSchema,
    Intent: ApplicationMutationIntent<Schema>,
    <IntentBinding<Schema, Intent> as ApplicationMutationBinding<Schema>>::ScopeBinding:
        ApplicationMutationScopeResolution<
            Schema,
            <IntentBinding<Schema, Intent> as ApplicationMutationBinding<Schema>>::PrincipalIdentity,
        >,
{
    let binding = request
        .request
        .application
        .installed_schema()
        .installed_mutation_binding::<IntentBinding<Schema, Intent>>()
        .map_err(WorthQueryApplicationRequestMutationDenial::BindingInstallation)?;
    let selected = request
        .request
        .application
        .on_branch(request.request.application.current_world())
        .select()
        .map_err(WorthQueryApplicationRequestMutationDenial::ProductSelection)?;
    let principal = selected
        .resolve_authenticated_principal(
            binding.principal_binding(),
            request.request.principal,
            request.request.scope,
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .map_err(WorthQueryApplicationRequestMutationDenial::PrincipalResolution)?;
    let (scope_field, scope_value) = request
        .request
        .intent
        .scope_binding()
        .into_field_parts(principal.principal_identity());
    let scope = selected
        .resolve_entity(
            scope_field,
            scope_value,
            request.request.scope,
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .map_err(WorthQueryApplicationRequestMutationDenial::ScopeResolution)?;
    let admission = selected
        .authorize_operation(
            &principal,
            &scope,
            binding.operation(),
            TypedMutationPreconditions::default(),
            request.request.scope,
        )
        .map_err(WorthQueryApplicationRequestMutationDenial::Authorization)?;
    let idempotency = WorthQueryApplicationIdempotencyBinding::new(
        IntentBinding::<Schema, Intent>::idempotency_key_identity(request.key),
        IntentBinding::<Schema, Intent>::input_identity(&request.request.intent),
    );
    Ok(PreparedMutation {
        admission,
        idempotency,
    })
}
