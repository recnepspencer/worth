use super::*;
use worth_query_declaration::facade::application_capability::ApplicationCapabilityRef;

pub(in crate::application_entry) fn prepare_capability_selected<Schema, Intent, SourcePreparation>(
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
    let binding = request
        .request
        .application
        .installed_schema()
        .installed_mutation_binding::<IntentBinding<Schema, Intent>>()
        .map_err(WorthQueryApplicationRequestMutationDenial::BindingInstallation)?;
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
    let admission = request
        .request
        .application
        .authorize_capability_operation(
            access,
            binding.operation(),
            std::mem::take(&mut request.request.preconditions),
        )
        .map_err(WorthQueryApplicationRequestMutationDenial::Authorization)?;
    prepare_authorized(
        request,
        AuthorizedMutation {
            principal_identity,
            admission,
        },
    )
}
