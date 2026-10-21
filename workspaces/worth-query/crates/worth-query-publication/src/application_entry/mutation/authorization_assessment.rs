use worth_query_declaration::facade::application_operation::{
    ApplicationEncodedInput, ApplicationMutationAdmissionBinding, ApplicationMutationBinding,
    ApplicationMutationIntent, ApplicationMutationScopeResolution,
};
use worth_query_execution::facade::{
    application_contribution::WorthQueryAdvancementPhase,
    primary_graph::WorthQuerySelectedProductOperation,
};
use worth_query_installation::facade::ApplicationSchema;

use super::{
    authorization::WorthQueryBindingAdmission, WorthQueryApplicationMutationRequest,
    WorthQueryApplicationMutationRequestWithIdempotency,
};
use crate::application_entry::WorthQueryApplicationRequestMutationDenial;

/// A fresh authorization observation. It contains no admission or execution authority.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorthQueryCurrentAuthorizationAssessment {
    private: (),
}

/// Observe the binding-selected authorization of an owner's frozen description.
/// No candidate, idempotency claim, recovery or workflow grant is advanced here.
pub(in crate::application_entry) fn assess_owned<Schema, Intent, SourcePreparation>(
    phase: &WorthQueryAdvancementPhase<'_>,
    request: &WorthQueryApplicationMutationRequestWithIdempotency<
        '_,
        '_,
        '_,
        '_,
        Schema,
        Intent,
        SourcePreparation,
    >,
    input: ApplicationEncodedInput<
        <Intent::Binding as ApplicationMutationBinding<Schema>>::InputBinding,
    >,
    selected: &WorthQuerySelectedProductOperation<'_, Schema>,
) -> Result<WorthQueryCurrentAuthorizationAssessment, WorthQueryApplicationRequestMutationDenial>
where
    Schema: ApplicationSchema,
    Intent: ApplicationMutationIntent<Schema>,
    Intent::Binding: ApplicationMutationAdmissionBinding<Schema>,
    <Intent::Binding as ApplicationMutationAdmissionBinding<Schema>>::Admission:
        WorthQueryBindingAdmission<Schema, Intent::Binding>,
    <Intent::Binding as ApplicationMutationBinding<Schema>>::ScopeBinding:
        ApplicationMutationScopeResolution<
            Schema,
            <Intent::Binding as ApplicationMutationBinding<Schema>>::PrincipalIdentity,
        >,
{
    <Intent::Binding as ApplicationMutationAdmissionBinding<Schema>>::Admission::assess_selected(
        phase, request, input, selected,
    )?;
    Ok(WorthQueryCurrentAuthorizationAssessment { private: () })
}

impl<Schema, Intent, SourcePreparation>
    WorthQueryApplicationMutationRequest<'_, '_, '_, Schema, Intent, SourcePreparation>
where
    Schema: ApplicationSchema,
    Intent: ApplicationMutationIntent<Schema>,
    <Intent::Binding as ApplicationMutationBinding<Schema>>::ScopeBinding:
        ApplicationMutationScopeResolution<
            Schema,
            <Intent::Binding as ApplicationMutationBinding<Schema>>::PrincipalIdentity,
        >,
{
    pub fn assess_current_authorization(
        mut self,
    ) -> Result<WorthQueryCurrentAuthorizationAssessment, WorthQueryApplicationRequestMutationDenial>
    {
        let application = self.application;
        let scope = self.scope.clone();
        application
            .with_application_advancement(&scope, |phase| {
                super::authorization::assess(&phase, &mut self)?;
                Ok(WorthQueryCurrentAuthorizationAssessment { private: () })
            })
            .map_err(WorthQueryApplicationRequestMutationDenial::ExecutionRequest)?
    }
}
