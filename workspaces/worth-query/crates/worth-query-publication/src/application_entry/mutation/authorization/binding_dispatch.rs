//! Binding-selected composition of the existing native admission owners.
use super::*;
use worth_query_declaration::facade::application_operation::{
    ApplicationCapabilityAdmission, ApplicationConventionalAdmission,
    ApplicationMutationAdmissionMode,
};
pub trait WorthQueryBindingAdmission<
    Schema: ApplicationSchema,
    Binding: ApplicationMutationBinding<Schema>,
>: ApplicationMutationAdmissionMode
{
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
impl<Schema, Binding, Capability> WorthQueryBindingAdmission<Schema, Binding> for ApplicationCapabilityAdmission<Capability>
where Schema: ApplicationSchema, Binding: ApplicationCapabilityMutationBinding<Schema, Capability=Capability>, Capability: worth_query_declaration::facade::application_capability::ApplicationCapabilityMarkerIdentity<Schema=Schema> + 'static,
Binding::Input: Clone + worth_query_declaration::facade::application_capability::ApplicationCapabilityRequest<Schema, Capability, Scope=MutationScope<Schema, Binding>> {
    fn prepare_selected<Intent, SourcePreparation>(phase: &AdvancementPhase<'_>, request: &Request<'_, '_, '_, '_, Schema, Intent, SourcePreparation>, identities: &Identities<'_, Schema, Intent>, staged: WorthQueryStagedMutation<Schema, Intent>, selected: &WorthQuerySelectedProductOperation<'_, Schema>) -> Result<PreparedMutation<Schema, Binding>, WorthQueryApplicationRequestMutationDenial>
    where Intent: ApplicationMutationIntent<Schema, Binding=Binding>, Binding::ScopeBinding: ApplicationMutationScopeResolution<Schema, Binding::PrincipalIdentity> {
        super::prepare_capability_selected(phase, request, identities, staged, selected)
    }
}
