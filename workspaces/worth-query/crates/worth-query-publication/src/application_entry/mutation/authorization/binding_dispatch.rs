//! Binding-selected composition of the existing native admission owners.
use super::*;
use worth_query_declaration::facade::application_capability::{
    ApplicationCapabilityMarkerIdentity, ApplicationCapabilityRequest,
};
use worth_query_declaration::facade::application_operation::{
    ApplicationCapabilityAdmission, ApplicationConventionalAdmission, ApplicationEncodedInput,
    ApplicationMutationAdmissionMode,
};
pub trait WorthQueryBindingAdmission<
    Schema: ApplicationSchema,
    Binding: ApplicationMutationBinding<Schema>,
>: ApplicationMutationAdmissionMode
{
    fn assess_selected<Intent, SourcePreparation>(
        phase: &AdvancementPhase<'_>,
        request: &Request<'_, '_, '_, '_, Schema, Intent, SourcePreparation>,
        input: ApplicationEncodedInput<Binding::InputBinding>,
        selected: &WorthQuerySelectedProductOperation<'_, Schema>,
    ) -> Result<(), WorthQueryApplicationRequestMutationDenial>
    where
        Intent: ApplicationMutationIntent<Schema, Binding = Binding>,
        Binding::ScopeBinding:
            ApplicationMutationScopeResolution<Schema, Binding::PrincipalIdentity>;

    fn prepare_selected<Intent, SourcePreparation>(
        phase: &AdvancementPhase<'_>,
        request: &Request<'_, '_, '_, '_, Schema, Intent, SourcePreparation>,
        identities: &Identities<'_, Schema, Intent>,
        staged: WorthQueryStagedMutation<Schema, Intent>,
        selected: &WorthQuerySelectedProductOperation<'_, Schema>,
    ) -> Result<PreparedMutation<Schema, Binding>, WorthQueryApplicationRequestMutationDenial>
    where
        Intent: ApplicationMutationIntent<Schema, Binding = Binding>,
        Binding::ScopeBinding:
            ApplicationMutationScopeResolution<Schema, Binding::PrincipalIdentity>;
}
impl<Schema: ApplicationSchema, Binding: ApplicationMutationBinding<Schema>>
    WorthQueryBindingAdmission<Schema, Binding> for ApplicationConventionalAdmission
{
    fn assess_selected<Intent, SourcePreparation>(
        phase: &AdvancementPhase<'_>,
        request: &Request<'_, '_, '_, '_, Schema, Intent, SourcePreparation>,
        _input: ApplicationEncodedInput<Binding::InputBinding>,
        selected: &WorthQuerySelectedProductOperation<'_, Schema>,
    ) -> Result<(), WorthQueryApplicationRequestMutationDenial>
    where
        Intent: ApplicationMutationIntent<Schema, Binding = Binding>,
        Binding::ScopeBinding:
            ApplicationMutationScopeResolution<Schema, Binding::PrincipalIdentity>,
    {
        super::authorize_selected(
            phase,
            &request.request,
            request
                .borrowed_preconditions
                .unwrap_or(&request.request.preconditions),
            selected,
        )
        .map(|_| ())
    }

    fn prepare_selected<Intent, SourcePreparation>(
        phase: &AdvancementPhase<'_>,
        request: &Request<'_, '_, '_, '_, Schema, Intent, SourcePreparation>,
        identities: &Identities<'_, Schema, Intent>,
        staged: WorthQueryStagedMutation<Schema, Intent>,
        selected: &WorthQuerySelectedProductOperation<'_, Schema>,
    ) -> Result<PreparedMutation<Schema, Binding>, WorthQueryApplicationRequestMutationDenial>
    where
        Intent: ApplicationMutationIntent<Schema, Binding = Binding>,
        Binding::ScopeBinding:
            ApplicationMutationScopeResolution<Schema, Binding::PrincipalIdentity>,
    {
        super::prepare_selected(phase, request, identities, staged, selected)
    }
}
impl<Schema, Binding, Capability> WorthQueryBindingAdmission<Schema, Binding>
    for ApplicationCapabilityAdmission<Capability>
where
    Schema: ApplicationSchema,
    Binding: ApplicationCapabilityMutationBinding<Schema, Capability = Capability>,
    Capability: ApplicationCapabilityMarkerIdentity<Schema = Schema> + 'static,
    Binding::Input: Clone
        + ApplicationCapabilityRequest<Schema, Capability, Scope = MutationScope<Schema, Binding>>,
{
    fn assess_selected<Intent, SourcePreparation>(
        phase: &AdvancementPhase<'_>,
        request: &Request<'_, '_, '_, '_, Schema, Intent, SourcePreparation>,
        input: ApplicationEncodedInput<Binding::InputBinding>,
        selected: &WorthQuerySelectedProductOperation<'_, Schema>,
    ) -> Result<(), WorthQueryApplicationRequestMutationDenial>
    where
        Intent: ApplicationMutationIntent<Schema, Binding = Binding>,
        Binding::ScopeBinding:
            ApplicationMutationScopeResolution<Schema, Binding::PrincipalIdentity>,
    {
        super::capability::authorize_capability_selected(
            phase,
            &request.request,
            input,
            request
                .borrowed_preconditions
                .unwrap_or(&request.request.preconditions),
            selected,
            request.workflow_actor_context.as_ref(),
        )
        .map(|_| ())
    }

    fn prepare_selected<Intent, SourcePreparation>(
        phase: &AdvancementPhase<'_>,
        request: &Request<'_, '_, '_, '_, Schema, Intent, SourcePreparation>,
        identities: &Identities<'_, Schema, Intent>,
        staged: WorthQueryStagedMutation<Schema, Intent>,
        selected: &WorthQuerySelectedProductOperation<'_, Schema>,
    ) -> Result<PreparedMutation<Schema, Binding>, WorthQueryApplicationRequestMutationDenial>
    where
        Intent: ApplicationMutationIntent<Schema, Binding = Binding>,
        Binding::ScopeBinding:
            ApplicationMutationScopeResolution<Schema, Binding::PrincipalIdentity>,
    {
        super::prepare_capability_selected(phase, request, identities, staged, selected)
    }
}
