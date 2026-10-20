use super::*;
use worth_query_declaration::facade::application_capability::ApplicationCapabilityRef;
use worth_query_declaration::facade::application_operation::{
    ApplicationEncodedInput, ApplicationMutationIdentityDenial,
};

pub(in crate::application_entry) fn prepare_capability_selected<Schema, Intent, SourcePreparation>(
phase: &worth_query_execution::facade::application_contribution::WorthQueryAdvancementPhase<'_>,
    request: &Request<'_, '_, '_, '_, Schema, Intent, SourcePreparation>,
    identities: &Identities<'_, Schema, Intent>,
    staged: WorthQueryStagedMutation<Schema, Intent>,
    selected: &WorthQuerySelectedProductOperation<'_, Schema>,
) -> Result<PreparedMutation<Schema, IntentBinding<Schema, Intent>>, WorthQueryApplicationRequestMutationDenial>
where
    Schema: ApplicationSchema,
    Intent: ApplicationMutationIntent<Schema>,
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
    let authorized = authorize_capability_selected(
        phase,
        &request.request,
        match request.operation_input {
            Some(Ok(input)) => input.clone(),
            Some(Err(denial)) => {
                return Err(WorthQueryApplicationRequestMutationDenial::Identity(
                    ApplicationMutationIdentityDenial::Input(denial.clone()),
                ))
            }
            None => identities.encoded_input(),
        },
        request
            .borrowed_preconditions
            .unwrap_or(&staged.preconditions),
        selected,
        request.workflow_actor_context.as_ref(),
    )?;
    prepare_authorized(request, identities, staged.source, authorized)
}

pub(in crate::application_entry::mutation) fn admit_capability_for_assessment<
    Schema,
    Intent,
    SourcePreparation,
>(
    phase: &AdvancementPhase<'_>,
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
    AdmittedMutation<Schema, IntentBinding<Schema, Intent>>,
    WorthQueryApplicationRequestMutationDenial,
>
where
    Schema: ApplicationSchema,
    Intent: ApplicationMutationIntent<Schema>,
    Intent::Binding: ApplicationCapabilityMutationBinding<Schema>,
    <Intent::Binding as ApplicationMutationBinding<Schema>>::ScopeBinding:
        ApplicationMutationScopeResolution<Schema, IntentPrincipal<Schema, Intent>>,
    <Intent::Binding as ApplicationMutationBinding<Schema>>::Input:
        Clone
            + worth_query_declaration::facade::application_capability::ApplicationCapabilityRequest<
                Schema,
                <Intent::Binding as ApplicationCapabilityMutationBinding<Schema>>::Capability,
                Scope = MutationScope<Schema, Intent::Binding>,
            >,
{
    let input =
        ApplicationEncodedInput::encode(request.intent.input().clone()).map_err(|cause| {
            WorthQueryApplicationRequestMutationDenial::Identity(
                ApplicationMutationIdentityDenial::Input(cause),
            )
        })?;
    let preconditions = std::mem::take(&mut request.preconditions);
    authorize_capability_selected(phase, request, input, &preconditions, selected, None)
        .map(|authorized| authorized.admission)
}

fn authorize_capability_selected<Schema, Intent, SourcePreparation>(
    phase: &AdvancementPhase<'_>,
    request: &WorthQueryApplicationMutationRequest<'_, '_, '_, Schema, Intent, SourcePreparation>,
    input: ApplicationEncodedInput<
        <Intent::Binding as ApplicationMutationBinding<Schema>>::InputBinding,
    >,
    preconditions: &Preconditions<Schema, Intent>,
    selected: &WorthQuerySelectedProductOperation<'_, Schema>,
    actor_context: Option<&worth_query_execution::publication_boundary::workflow_advance::WorthQueryWorkflowActorContext>,
) -> Result<AuthorizedMutation<Schema, Intent::Binding>, WorthQueryApplicationRequestMutationDenial>
where
    Schema: ApplicationSchema,
    Intent: ApplicationMutationIntent<Schema>,
    Intent::Binding: ApplicationCapabilityMutationBinding<Schema>,
    <Intent::Binding as ApplicationMutationBinding<Schema>>::ScopeBinding:
        ApplicationMutationScopeResolution<Schema, IntentPrincipal<Schema, Intent>>,
    <Intent::Binding as ApplicationMutationBinding<Schema>>::Input:
        worth_query_declaration::facade::application_capability::ApplicationCapabilityRequest<
            Schema,
            <Intent::Binding as ApplicationCapabilityMutationBinding<Schema>>::Capability,
            Scope = MutationScope<Schema, Intent::Binding>,
        >,
{
    let binding = request
        .application
        .installed_schema()
        .installed_mutation_binding::<IntentBinding<Schema, Intent>>()
        .map_err(WorthQueryApplicationRequestMutationDenial::BindingInstallation)?;
    let principal = selected
        .resolve_authenticated_principal(
            phase,
            binding.principal_binding(),
            request.principal,
            request.scope,
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .map_err(WorthQueryApplicationRequestMutationDenial::PrincipalResolution)?;
    let capability =
        request
            .application
            .installed_schema()
            .capability(
                ApplicationCapabilityRef::<
                    Schema,
                    <IntentBinding<Schema, Intent> as ApplicationCapabilityMutationBinding<
                        Schema,
                    >>::Capability,
                >::from_declaration(),
                request.intent.operation(),
            )
            .map_err(WorthQueryApplicationRequestMutationDenial::CapabilityInstallation)?;
    let access = selected
        .admit_encoded_capability_access(
            phase,
            &worth_query_execution::publication_boundary::program_publication_access(),
            &principal,
            &capability,
            input,
            request.scope,
            actor_context,
        )
        .map_err(WorthQueryApplicationRequestMutationDenial::Authorization)?;

    let principal_identity = principal.principal_identity().clone();
    let admission = request
        .application
        .authorize_borrowed_capability_operation(phase, access, binding.operation(), preconditions)
        .map_err(WorthQueryApplicationRequestMutationDenial::Authorization)?;
    Ok(AuthorizedMutation {
        principal_identity,
        admission,
    })
}
