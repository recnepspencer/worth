//! Native acceptance borrows exact completion identity and returns its cleanup custody.
use super::*;
use crate::application_entry::{
    WorthQueryApplicationRequestExt, WorthQueryCompletedWorkflowOperation,
    WorthQueryWorkflowAcceptance,
};
use worth_query_declaration::facade::{
    application_operation::ApplicationMutationAdmissionBinding,
    application_program::{ApplicationAction, ApplicationProgramDefinition},
};
impl<'a, 'p, 's, S, Spec, O, Input, Scope>
    WorthQueryWorkflowAdvanceRequest<'a, 'p, 's, S, Spec, O, Input, Scope>
where
    S: ApplicationSchema + 'static,
    Spec: ApplicationWorkflowSpec<Schema = S>,
    O: 'static,
    Input: Clone + Send + Sync + 'static,
{
    /// Confirms the original effect under fresh selected admission, retaining its cleanup.
    /// Local completion already settled the operation atomically: confirmation replays that
    /// original progress without committing this command key or advancing its successor.
    /// External completion still requires native recovery/inbound evidence and settlement.
    /// Every refusal returns the same completed owner; physical close is a separate step.
    pub fn accept_owned_operation<I, P, A>(
        self,
        operation: WorthQueryCompletedWorkflowOperation<S, I, P, A>,
    ) -> Result<
        WorthQueryWorkflowAcceptance<WorthQueryCompletedWorkflowOperation<S, I, P, A>>,
        (
            WorthQueryCompletedWorkflowOperation<S, I, P, A>,
            WorthQueryWorkflowOperationOwnerAcceptanceDenial,
        ),
    >
    where
        I: ApplicationMutationIntent<S>,
        P: ApplicationProgramDefinition<S>,
        A: ApplicationAction<S, I::Binding>,
        I::Binding: ApplicationMutationAdmissionBinding<S>,
        <I::Binding as ApplicationMutationAdmissionBinding<S>>::Admission:
            crate::application_entry::mutation::authorization::WorthQueryBindingAdmission<
                S,
                I::Binding,
            >,
        <I::Binding as ApplicationMutationBinding<S>>::Input: Clone + Send + Sync + 'static,
        <I::Binding as ApplicationMutationBinding<S>>::ScopeBinding:
            ApplicationMutationScopeResolution<
                S,
                <I::Binding as ApplicationMutationBinding<S>>::PrincipalIdentity,
            >,
    {
        let request = self
            .application
            .request(self.principal, self.scope)
            .on_branch(operation.required.branch());
        let application = self.application;
        let scope = self.scope.clone();
        let result = application
            .with_application_advancement(&scope, |phase| {
                let source = &operation.source.operation.admission;
                let mut view = operation.binding_view(&request).native;
                let identities = view
                    .identities()
                    .map_err(WorthQueryWorkflowOperationOwnerAcceptanceDenial::Request)?;
                view.validate_workflow_operation_binding(
                    self.workflow,
                    &operation.required,
                    identities.input_identity(),
                )
                .map_err(WorthQueryWorkflowOperationOwnerAcceptanceDenial::Binding)?;
                let prepared = source
                    .authorize_recovery_view(&phase, &request, &mut view)
                    .map_err(WorthQueryWorkflowOperationOwnerAcceptanceDenial::Opening)?;
                let custody = WorthQueryWorkflowAdvanceAdapter::resolve_guarded_operation_custody(
                    &phase,
                    application,
                    &prepared.admission,
                    prepared.idempotency,
                    operation.required.transition_identity_bytes(),
                )
                .map_err(WorthQueryWorkflowOperationOwnerAcceptanceDenial::Inspection)?;
                let receipt = match custody {
                    WorthQueryGuardedWorkflowOperationCustody::Committed(receipt)
                    | WorthQueryGuardedWorkflowOperationCustody::ExternallySettled(receipt) => {
                        receipt
                    }
                    WorthQueryGuardedWorkflowOperationCustody::DispatchPending(receipt)
                        if operation.recovery_result.is_some() =>
                    {
                        receipt
                    }
                    WorthQueryGuardedWorkflowOperationCustody::DispatchPending(_) => {
                        return Err(WorthQueryWorkflowOperationOwnerAcceptanceDenial::Owner(
                            WorthQueryWorkflowOperationOwnerPosture::DispatchPending,
                        ))
                    }
                    other => {
                        return Err(WorthQueryWorkflowOperationOwnerAcceptanceDenial::Owner(
                            other_custody(other),
                        ))
                    }
                };
                self.accept_custody::<I::Binding, _, _, _>(
                    &phase,
                    &operation.required,
                    &receipt,
                    operation.recovery_result.as_ref(),
                    &prepared.admission,
                    prepared.idempotency,
                )
                .map_err(WorthQueryWorkflowOperationOwnerAcceptanceDenial::Acceptance)
            })
            .map_err(|cause| {
                WorthQueryWorkflowOperationOwnerAcceptanceDenial::Opening(
                    WorthQueryApplicationRecoveryRequestDenial::advancement(cause),
                )
            })
            .and_then(|outcome| outcome);
        match result {
            Ok(progress) => Ok(WorthQueryWorkflowAcceptance {
                progress,
                operation,
            }),
            Err(cause) => Err((operation, cause)),
        }
    }
}
