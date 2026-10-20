//! Completed local actions settle with their original fresh capability and exact source owner.
use super::*;
use crate::application_entry::WorthQueryWorkflowOperationOwnerAcceptanceDenial as Denial;
use worth_query_declaration::facade::{
    application_capability::ApplicationCapabilityRequest,
    application_operation::{
        ApplicationCapabilityMutationBinding, ApplicationMutationAdmissionBinding,
        ApplicationMutationScopeBinding,
    },
};
use worth_query_execution::{
    facade::application_installation::WorthQueryWorkflowVocabulary,
    publication_boundary::workflow_advance::{
        WorthQueryGuardedWorkflowOperationCustody, WorthQueryWorkflowAdvanceAdapter,
    },
};

impl<S, I, P, A> WorthQueryCompletedWorkflowOperation<S, I, P, A>
where
    S: ApplicationSchema + 'static,
    I: ApplicationMutationIntent<S>,
    P: ApplicationProgramDefinition<S>,
    A: ApplicationAction<S, I::Binding>,
    I::Binding: ApplicationMutationAdmissionBinding<S> + ApplicationCapabilityMutationBinding<S>,
    <I::Binding as ApplicationMutationAdmissionBinding<S>>::Admission:
        crate::application_entry::mutation::authorization::WorthQueryBindingAdmission<S, I::Binding>,
    <I::Binding as ApplicationMutationBinding<S>>::Input: Clone + Send + Sync + 'static
        + ApplicationCapabilityRequest<
            S,
            <I::Binding as ApplicationCapabilityMutationBinding<S>>::Capability,
            Scope = <<I::Binding as ApplicationMutationBinding<S>>::ScopeBinding as ApplicationMutationScopeBinding<S>>::Scope,
        >,
    <I::Binding as ApplicationMutationBinding<S>>::ScopeBinding:
        ApplicationMutationScopeResolution<S, <I::Binding as ApplicationMutationBinding<S>>::PrincipalIdentity>,
{
    /// Accept an original Back/termination owner without borrowing Advance permission.
    /// Fresh original-binding admission precedes exact effect and settlement inspection.
    /// Every refusal returns this same completed owner and its independent cleanup.
    pub fn accept_local_action<'w, Spec>(
        self,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
        workflow: impl Into<WorthQueryWorkflowVocabulary<'w, S, Spec>>,
    ) -> Result<WorthQueryWorkflowAcceptance<Self>, (Self, Denial)>
    where
        Spec: ApplicationWorkflowSpec<Schema = S>,
    {
        let workflow = workflow.into();
        let source = &self.source.operation.admission;
        let result = source.runtime.runtime()
            .with_application_advancement(request.scope, |phase| {
                let (selected, _) = source.select(&phase, request)
                    .map_err(|cause| Denial::Opening(cause.into()))?;
                if !std::ptr::eq(workflow.runtime(), source.runtime.runtime()) {
                    return Err(Denial::Binding(
                        crate::application_entry::WorthQueryWorkflowOperationBindingDenial::RuntimeMismatch,
                    ));
                }
                let mut view = source.description.request_view(request, &source.installed_action);
                source.bind_workflow_view(&mut view, false);
                let identities = view.identities().map_err(Denial::Request)?;
                view.validate_workflow_operation_binding(workflow, &self.required, identities.input_identity())
                    .map_err(Denial::Binding)?;
                let prepared = source.authorize_recovery_view(&phase, request, &mut view)
                    .map_err(Denial::Opening)?;
                let receipt = match self.evidence() {
                    WorthQuerySourceCompletion::Fresh { receipt, .. }
                    | WorthQuerySourceCompletion::Recovered { receipt, .. } => receipt,
                };
                let custody = WorthQueryWorkflowAdvanceAdapter::resolve_guarded_operation_custody(
                    &phase, source.runtime.runtime(), &prepared.admission, prepared.idempotency,
                    self.required.transition_identity_bytes(), Some(receipt),
                ).map_err(Denial::Inspection)?;
                let receipt = match custody {
                    WorthQueryGuardedWorkflowOperationCustody::Committed(receipt)
                    | WorthQueryGuardedWorkflowOperationCustody::ExternallySettled(receipt) => receipt,
                    WorthQueryGuardedWorkflowOperationCustody::DispatchPending(receipt)
                        if self.recovery_result.is_some() => receipt,
                    other => return Err(Denial::Owner(
                        crate::application_entry::workflow::operation::owner::other_custody(other),
                    )),
                };
                WorthQueryWorkflowAdvanceAdapter::complete_local_operation::<S, Spec, I::Binding>(
                    &phase, &selected, workflow, &self.required, prepared.admission,
                    prepared.idempotency, &receipt, self.recovery_result.as_ref(),
                ).map_err(Denial::LocalActionPreparation)
            })
            .map_err(|cause| Denial::Opening(
                crate::application_entry::WorthQueryApplicationRecoveryRequestDenial::advancement(cause),
            ))
            .and_then(|result| result);
        match result {
            Ok(progress) => Ok(WorthQueryWorkflowAcceptance { progress, operation: self }),
            Err(cause) => Err((self, cause)),
        }
    }
}
