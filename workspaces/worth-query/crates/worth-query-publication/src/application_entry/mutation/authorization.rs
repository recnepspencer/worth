use worth_query_declaration::facade::application_operation::{
    ApplicationCapabilityMutationBinding, ApplicationMutationBinding, ApplicationMutationIntent,
    ApplicationMutationScopeBinding, ApplicationMutationScopeResolution,
    ApplicationMutationSourceExpectation,
};
use worth_query_execution::facade::primary_graph::{
    WorthQueryAdmittedApplicationOperation, WorthQueryApplicationIdempotencyBinding,
    WorthQueryPrincipalResolutionMode,
};
use worth_query_installation::facade::ApplicationSchema;

use super::{
    WorthQueryApplicationMutationRequest, WorthQueryApplicationMutationRequestWithIdempotency,
};
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
    pub(super) principal_identity: Binding::PrincipalIdentity,
    pub(super) admission: WorthQueryAdmittedApplicationOperation<
        Schema,
        Binding::Operation,
        Binding::Input,
        MutationScope<Schema, Binding>,
    >,
    pub(super) idempotency: WorthQueryApplicationIdempotencyBinding,
}

struct AuthorizedMutation<Schema, Binding>
where
    Schema: ApplicationSchema,
    Binding: ApplicationMutationBinding<Schema>,
{
    principal_identity: Binding::PrincipalIdentity,
    admission: WorthQueryAdmittedApplicationOperation<
        Schema,
        Binding::Operation,
        Binding::Input,
        MutationScope<Schema, Binding>,
    >,
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
        ApplicationMutationScopeResolution<
            Schema,
            <IntentBinding<Schema, Intent> as ApplicationMutationBinding<Schema>>::PrincipalIdentity,
        >,
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
        ApplicationMutationScopeResolution<
            Schema,
            <IntentBinding<Schema, Intent> as ApplicationMutationBinding<Schema>>::PrincipalIdentity,
        >,
{
    let binding = request
        .application
        .installed_schema()
        .installed_mutation_binding::<IntentBinding<Schema, Intent>>()
        .map_err(WorthQueryApplicationRequestMutationDenial::BindingInstallation)?;
    let selected = request
        .application
        .on_branch(request.branch)
        .select()
        .map_err(WorthQueryApplicationRequestMutationDenial::ProductSelection)?;
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

pub(super) fn prepare<Schema, Intent, SourcePreparation>(
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
    let principal_identity = authorized.principal_identity;
    let mut admission = authorized.admission;
    let expected_source = <<IntentBinding<Schema, Intent> as ApplicationMutationBinding<
        Schema,
    >>::SourceExpectation as ApplicationMutationSourceExpectation<Schema>>::QUERY_IDENTIFIER;
    let mut source_identity = None;
    match (expected_source, request.request.source.take()) {
        (Some(_expected), Some(source)) => {
            source_identity = Some(
                request
                    .request
                    .application
                    .bind_application_source_expectation::<IntentBinding<Schema, Intent>, _>(
                        &mut admission,
                        source,
                    )
                    .map_err(WorthQueryApplicationRequestMutationDenial::SourceExpectation)?,
            );
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
    )
    .bind_source(source_identity.as_ref());
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
    use worth_query_declaration::facade::application_capability::ApplicationCapabilityRef;

    let binding = request
        .request
        .application
        .installed_schema()
        .installed_mutation_binding::<IntentBinding<Schema, Intent>>()
        .map_err(WorthQueryApplicationRequestMutationDenial::BindingInstallation)?;
    let selected = request
        .request
        .application
        .on_branch(request.request.branch)
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
    let capability =
        request
            .request
            .application
            .installed_schema()
            .capability(
                ApplicationCapabilityRef::<
                    Schema,
                    <IntentBinding<Schema, Intent> as ApplicationCapabilityMutationBinding<
                        Schema,
                    >>::Capability,
                >::from_declaration(),
                request.request.intent.operation(),
            )
            .map_err(WorthQueryApplicationRequestMutationDenial::CapabilityInstallation)?;
    let access = selected
        .admit_capability_access(
            &principal,
            &capability,
            request.request.intent.input().clone(),
            request.request.scope,
        )
        .map_err(WorthQueryApplicationRequestMutationDenial::Authorization)?;
    let principal_identity = principal.principal_identity().clone();
    let mut admission = request
        .request
        .application
        .authorize_capability_operation(
            access,
            binding.operation(),
            std::mem::take(&mut request.request.preconditions),
        )
        .map_err(WorthQueryApplicationRequestMutationDenial::Authorization)?;
    let expected_source = <<IntentBinding<Schema, Intent> as ApplicationMutationBinding<
        Schema,
    >>::SourceExpectation as ApplicationMutationSourceExpectation<Schema>>::QUERY_IDENTIFIER;
    let mut source_identity = None;
    match (expected_source, request.request.source.take()) {
        (Some(_), Some(source)) => {
            source_identity = Some(
                request
                    .request
                    .application
                    .bind_application_source_expectation::<IntentBinding<Schema, Intent>, _>(
                        &mut admission,
                        source,
                    )
                    .map_err(WorthQueryApplicationRequestMutationDenial::SourceExpectation)?,
            );
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
    )
    .bind_source(source_identity.as_ref());
    Ok(PreparedMutation {
        principal_identity,
        admission,
        idempotency,
    })
}
