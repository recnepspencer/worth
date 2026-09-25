use worth_query_declaration::facade::application_operation::{
    ApplicationCapabilityMutationBinding, ApplicationMutationBinding, ApplicationMutationIntent,
    ApplicationMutationScopeBinding, ApplicationMutationScopeResolution,
    ApplicationMutationSourceExpectation,
};
use worth_query_execution::facade::primary_graph::{
    WorthQueryAdmittedApplicationOperation, WorthQueryApplicationIdempotencyBinding,
    WorthQueryPrincipalResolutionMode, WorthQuerySelectedProductOperation,
};
use worth_query_installation::facade::ApplicationSchema;

use super::{
    request::WorthQueryMutationExpectedSource, WorthQueryApplicationMutationRequest,
    WorthQueryApplicationMutationRequestWithIdempotency,
};
use crate::application_entry::WorthQueryApplicationRequestMutationDenial;

#[path = "authorization/authorized.rs"]
mod authorized;
use authorized::AuthorizedMutation;
mod capability;
pub(in crate::application_entry) use capability::prepare_capability_selected;

type IntentBinding<Schema, Intent> = <Intent as ApplicationMutationIntent<Schema>>::Binding;
type IntentPrincipal<Schema, Intent> =
    <IntentBinding<Schema, Intent> as ApplicationMutationBinding<Schema>>::PrincipalIdentity;
type MutationScope<Schema, Binding> =
    <<Binding as ApplicationMutationBinding<Schema>>::ScopeBinding as ApplicationMutationScopeBinding<
        Schema,
    >>::Scope;

pub(in crate::application_entry) struct PreparedMutation<Schema, Binding>
where
    Schema: ApplicationSchema,
    Binding: ApplicationMutationBinding<Schema>,
{
    pub(in crate::application_entry) principal_identity: Binding::PrincipalIdentity,
    pub(in crate::application_entry) admission: WorthQueryAdmittedApplicationOperation<
        Schema,
        Binding::Operation,
        Binding::Input,
        MutationScope<Schema, Binding>,
    >,
    pub(in crate::application_entry) idempotency: WorthQueryApplicationIdempotencyBinding,
}

pub(super) fn assess<Schema, Intent, SourcePreparation>(
    request: &mut WorthQueryApplicationMutationRequest<
        '_,
        '_,
        '_,
        Schema,
        Intent,
        SourcePreparation,
    >,
) -> Result<(), WorthQueryApplicationRequestMutationDenial>
where
    Schema: ApplicationSchema,
    Intent: ApplicationMutationIntent<Schema>,
    <IntentBinding<Schema, Intent> as ApplicationMutationBinding<Schema>>::ScopeBinding:
        ApplicationMutationScopeResolution<Schema, IntentPrincipal<Schema, Intent>>,
{
    authorize(request).map(|_| ())
}

fn authorize<Schema, Intent, SourcePreparation>(
    request: &mut WorthQueryApplicationMutationRequest<
        '_,
        '_,
        '_,
        Schema,
        Intent,
        SourcePreparation,
    >,
) -> Result<
    AuthorizedMutation<Schema, IntentBinding<Schema, Intent>>,
    WorthQueryApplicationRequestMutationDenial,
>
where
    Schema: ApplicationSchema,
    Intent: ApplicationMutationIntent<Schema>,
    <IntentBinding<Schema, Intent> as ApplicationMutationBinding<Schema>>::ScopeBinding:
        ApplicationMutationScopeResolution<Schema, IntentPrincipal<Schema, Intent>>,
{
    let selected = request
        .application
        .on_branch(request.branch)
        .select()
        .map_err(WorthQueryApplicationRequestMutationDenial::ProductSelection)?;
    authorize_selected(request, &selected)
}

fn authorize_selected<Schema, Intent, SourcePreparation>(
    request: &mut WorthQueryApplicationMutationRequest<
        '_,
        '_,
        '_,
        Schema,
        Intent,
        SourcePreparation,
    >,
    selected: &WorthQuerySelectedProductOperation<'_, Schema>,
) -> Result<
    AuthorizedMutation<Schema, IntentBinding<Schema, Intent>>,
    WorthQueryApplicationRequestMutationDenial,
>
where
    Schema: ApplicationSchema,
    Intent: ApplicationMutationIntent<Schema>,
    <IntentBinding<Schema, Intent> as ApplicationMutationBinding<Schema>>::ScopeBinding:
        ApplicationMutationScopeResolution<Schema, IntentPrincipal<Schema, Intent>>,
{
    let binding = request
        .application
        .installed_schema()
        .installed_mutation_binding::<IntentBinding<Schema, Intent>>()
        .map_err(WorthQueryApplicationRequestMutationDenial::BindingInstallation)?;
    let principal = selected
        .resolve_authenticated_principal(
            binding.principal_binding(),
            request.principal,
            request.scope,
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .map_err(WorthQueryApplicationRequestMutationDenial::PrincipalResolution)?;
    let (scope_field, scope_value) = request
        .intent
        .scope_binding()
        .into_field_parts(principal.principal_identity());
    let principal_identity = principal.principal_identity().clone();
    let scope = selected
        .resolve_entity(
            scope_field,
            scope_value,
            request.scope,
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .map_err(WorthQueryApplicationRequestMutationDenial::ScopeResolution)?;
    let admission = selected
        .authorize_operation(
            &principal,
            &scope,
            binding.operation(),
            std::mem::take(&mut request.preconditions),
            request.scope,
        )
        .map_err(WorthQueryApplicationRequestMutationDenial::Authorization)?;
    Ok(AuthorizedMutation {
        principal_identity,
        admission,
    })
}

pub(in crate::application_entry) fn prepare<Schema, Intent, SourcePreparation>(
    request: &mut WorthQueryApplicationMutationRequestWithIdempotency<
        '_,
        '_,
        '_,
        '_,
        Schema,
        Intent,
        SourcePreparation,
    >,
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
    let authorized = authorize(&mut request.request)?;
    prepare_authorized(request, authorized)
}

pub(super) fn prepare_selected<Schema, Intent, SourcePreparation>(
    request: &mut WorthQueryApplicationMutationRequestWithIdempotency<
        '_,
        '_,
        '_,
        '_,
        Schema,
        Intent,
        SourcePreparation,
    >,
    selected: &WorthQuerySelectedProductOperation<'_, Schema>,
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
    let authorized = authorize_selected(&mut request.request, selected)?;
    prepare_authorized(request, authorized)
}

fn prepare_authorized<Schema, Intent, SourcePreparation>(
    request: &mut WorthQueryApplicationMutationRequestWithIdempotency<
        '_,
        '_,
        '_,
        '_,
        Schema,
        Intent,
        SourcePreparation,
    >,
    authorized: AuthorizedMutation<Schema, IntentBinding<Schema, Intent>>,
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
    let principal_identity = authorized.principal_identity;
    let mut admission = authorized.admission;
    let expected_source = <<IntentBinding<Schema, Intent> as ApplicationMutationBinding<
        Schema,
    >>::SourceExpectation as ApplicationMutationSourceExpectation<Schema>>::QUERY_IDENTIFIER;
    let mut bound_source = None;
    match (expected_source, request.request.source.take()) {
        (Some(_expected), Some(source)) => {
            bound_source = Some(match source {
                WorthQueryMutationExpectedSource::Row(source) => request
                    .request
                    .application
                    .bind_application_source_expectation::<IntentBinding<Schema, Intent>, _>(
                        &mut admission,
                        source,
                    ),
                WorthQueryMutationExpectedSource::ResultSet(source) => request
                    .request
                    .application
                    .bind_application_result_set_expectation::<IntentBinding<Schema, Intent>, _>(
                        &mut admission,
                        source,
                    ),
            }
            .map_err(WorthQueryApplicationRequestMutationDenial::SourceExpectation)?);
        }
        (Some(expected), None) => {
            return Err(WorthQueryApplicationRequestMutationDenial::SourceExpectation(
                worth_query_execution::facade::primary_graph::WorthQuerySourceExpectationDenial::new_missing(expected),
            ));
        }
        (None, None) => {}
        (None, Some(_)) => unreachable!("a no-source binding has no constructible source marker"),
    }
    let idempotency = WorthQueryApplicationIdempotencyBinding::new(
        IntentBinding::<Schema, Intent>::idempotency_key_identity(request.key),
        IntentBinding::<Schema, Intent>::input_identity(request.request.intent.input()),
    );
    let idempotency = match bound_source {
        Some(source) => source.bind_idempotency(idempotency),
        None => idempotency,
    };
    let idempotency = request.workflow_transition_identity.map_or(idempotency, |identity| {
        worth_query_execution::facade::workflow_advance::WorthQueryWorkflowAdvanceAdapter::bind_operation_idempotency_raw(
            idempotency,
            &identity,
        )
    });
    Ok(PreparedMutation {
        principal_identity,
        admission,
        idempotency,
    })
}

pub(super) fn prepare_capability<Schema, Intent, SourcePreparation>(
    request: &mut WorthQueryApplicationMutationRequestWithIdempotency<
        '_,
        '_,
        '_,
        '_,
        Schema,
        Intent,
        SourcePreparation,
    >,
) -> Result<PreparedMutation<Schema, IntentBinding<Schema, Intent>>, WorthQueryApplicationRequestMutationDenial>
where
    Schema: ApplicationSchema,
    Intent: ApplicationMutationIntent<Schema> + Clone,
    IntentBinding<Schema, Intent>: ApplicationCapabilityMutationBinding<Schema>,
    <IntentBinding<Schema, Intent> as ApplicationMutationBinding<Schema>>::ScopeBinding:
        ApplicationMutationScopeResolution<
            Schema,
            <IntentBinding<Schema, Intent> as ApplicationMutationBinding<Schema>>::PrincipalIdentity,
        >,
    <IntentBinding<Schema, Intent> as ApplicationMutationBinding<Schema>>::Input:
        Clone
        +
        worth_query_declaration::facade::application_capability::ApplicationCapabilityRequest<
            Schema,
            <IntentBinding<Schema, Intent> as ApplicationCapabilityMutationBinding<
                Schema,
            >>::Capability,
            Scope = MutationScope<Schema, IntentBinding<Schema, Intent>>,
        >,
{
    let selected = request
        .request
        .application
        .on_branch(request.request.branch)
        .select()
        .map_err(WorthQueryApplicationRequestMutationDenial::ProductSelection)?;
    prepare_capability_selected(request, &selected)
}
