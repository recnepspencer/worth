//! Fresh observation retains authentic admission only within the current opening.
use super::*;
use crate::application_entry::mutation::authorization::AdmittedMutation;

impl<S, I, P, A, Mode> WorthQuerySourceOperationAdmission<S, I, P, A, Mode>
where
    S: ApplicationSchema + 'static,
    I: ApplicationMutationIntent<S>,
    P: ApplicationProgramDefinition<S>,
    A: ApplicationAction<S, I::Binding>,
    Mode: WorthQuerySourceDeliveryMode,
    I::Binding: ApplicationMutationAdmissionBinding<S>,
    <I::Binding as ApplicationMutationAdmissionBinding<S>>::Admission:
        WorthQueryBindingAdmission<S, I::Binding>,
    MutationInput<S, I>: Clone + Send + Sync,
    MutationScope<S, I>: ApplicationMutationScopeResolution<S, MutationPrincipal<S, I>>,
{
    pub(in crate::application_entry::operation) fn admit_for_assessment<'owner>(
        &'owner self,
        phase: &WorthQueryAdvancementPhase<'_>,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    ) -> Result<
        (
            AdmittedMutation<S, I::Binding>,
            worth_query_execution::facade::primary_graph::WorthQuerySelectedProductOperation<
                'owner,
                S,
            >,
        ),
        WorthQueryApplicationRequestMutationDenial,
    > {
        let (selected, _owner) = self.select(phase, request)?;
        let input = self
            .description
            .encoded_input
            .as_ref()
            .map_err(|cause| {
                WorthQueryApplicationRequestMutationDenial::Identity(
                    ApplicationMutationIdentityDenial::Input(cause.clone()),
                )
            })?
            .clone();
        let mut native = self
            .description
            .request_view(request, &self.installed_action);
        self.bind_workflow_view(&mut native, false);
        let admitted = <I::Binding as ApplicationMutationAdmissionBinding<S>>::Admission::admit_for_assessment_selected(
            phase, &native, input, &selected,
        )?;
        Ok((admitted, selected))
    }

    pub(in crate::application_entry::operation) fn assess_current_authorization(
        &self,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    ) -> Result<WorthQueryCurrentAuthorizationAssessment, WorthQueryApplicationRequestMutationDenial>
    {
        request
            .application
            .with_application_advancement(request.scope, |phase| {
                self.admit_for_assessment(&phase, request)?;
                Ok(WorthQueryCurrentAuthorizationAssessment::observed())
            })
            .map_err(WorthQueryApplicationRequestMutationDenial::ExecutionRequest)?
    }
}
