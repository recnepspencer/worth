//! Business Back and global endings use their binding's native admission owner.
use super::progress::{IntentBinding, MutationInput};
use crate::application_entry::{
    mutation::{
        authorization::WorthQueryBindingAdmission,
        WorthQueryApplicationMutationRequestWithIdempotency,
    },
    WorthQueryApplicationRequestMutationDenial, WorthQueryWorkflowAdvancePreparationDenial,
};
use worth_query_declaration::facade::{
    application_operation::{
        ApplicationMutationAdmissionBinding, ApplicationMutationBinding, ApplicationMutationIntent,
        ApplicationMutationScopeResolution,
    },
    application_program::ApplicationWorkflowSpec,
};
use worth_query_execution::{
    facade::application_installation::WorthQueryWorkflowVocabulary,
    publication_boundary::workflow_advance::{
        PublishedWorkflowInstanceRef, RequiredWorkflowOperation, WorkflowLocalActionKind,
        WorthQueryWorkflowAdvanceAdapter,
    },
};
use worth_query_installation::facade::ApplicationSchema;

impl<'a, 'p, 's, 'k, S, I, Source>
    WorthQueryApplicationMutationRequestWithIdempotency<'a, 'p, 's, 'k, S, I, Source>
where
    S: ApplicationSchema,
    I: ApplicationMutationIntent<S> + Clone,
    I::Binding: ApplicationMutationAdmissionBinding<S>,
    <I::Binding as ApplicationMutationAdmissionBinding<S>>::Admission:
        WorthQueryBindingAdmission<S, I::Binding>,
    <I::Binding as ApplicationMutationBinding<S>>::ScopeBinding: ApplicationMutationScopeResolution<
        S,
        <I::Binding as ApplicationMutationBinding<S>>::PrincipalIdentity,
    >,
    MutationInput<S, I>: Clone,
{
    /// Resolve the declared business return using this binding's fresh native admission.
    /// Preparation changes nothing; retain the requirement and execute the original intent.
    pub fn prepare_workflow_back_action<Spec>(
        self,
        workflow: impl Into<WorthQueryWorkflowVocabulary<'a, S, Spec>>,
        instance: PublishedWorkflowInstanceRef,
    ) -> Result<RequiredWorkflowOperation, WorthQueryWorkflowAdvancePreparationDenial>
    where
        Spec: ApplicationWorkflowSpec<Schema = S>,
    {
        self.prepare_local_workflow_action(
            workflow.into(),
            instance,
            WorkflowLocalActionKind::Return,
        )
    }

    /// Resolve the declared global ending under the original binding's native admission.
    /// Effectful actions retain custody until actual effect completion and native settlement.
    pub fn prepare_workflow_termination_action<Spec>(
        self,
        workflow: impl Into<WorthQueryWorkflowVocabulary<'a, S, Spec>>,
        instance: PublishedWorkflowInstanceRef,
    ) -> Result<RequiredWorkflowOperation, WorthQueryWorkflowAdvancePreparationDenial>
    where
        Spec: ApplicationWorkflowSpec<Schema = S>,
    {
        self.prepare_local_workflow_action(
            workflow.into(),
            instance,
            WorkflowLocalActionKind::Terminate,
        )
    }

    fn prepare_local_workflow_action<Spec>(
        mut self,
        workflow: WorthQueryWorkflowVocabulary<'a, S, Spec>,
        instance: PublishedWorkflowInstanceRef,
        kind: WorkflowLocalActionKind,
    ) -> Result<RequiredWorkflowOperation, WorthQueryWorkflowAdvancePreparationDenial>
    where
        Spec: ApplicationWorkflowSpec<Schema = S>,
    {
        use WorthQueryWorkflowAdvancePreparationDenial as D;
        let runtime = self.application_runtime();
        if !std::ptr::eq(runtime, workflow.runtime()) {
            return Err(D::RuntimeMismatch);
        }
        let scope = self.request_scope().clone();
        runtime.with_application_advancement(&scope, |phase| {
            let selected = runtime.on_branch(self.product_branch()).select(&phase)
                .map_err(WorthQueryApplicationRequestMutationDenial::ProductSelection)
                .map_err(D::RequestAdmission)?;
            let staged = self.stage().map_err(D::RequestAdmission)?;
            let identities = self.identities().map_err(D::RequestAdmission)?;
            let mutation = <I::Binding as ApplicationMutationAdmissionBinding<S>>::Admission::prepare_selected(
                &phase, &self, &identities, staged, &selected).map_err(D::RequestAdmission)?;
            let prepared = WorthQueryWorkflowAdvanceAdapter::prepare_local_action::<
                S, _, _, _, Spec, IntentBinding<S, I>>(
                &phase, &selected, workflow.workflow_spec_for(&selected), instance, mutation.admission, &identities, kind)
                .map_err(D::TransitionPreparation)?;
            let unavailable = match kind {
                WorkflowLocalActionKind::Return => D::BackActionUnavailable,
                WorkflowLocalActionKind::Terminate => D::TerminationActionUnavailable,
            };
            required_local_action(prepared, unavailable)
        }).map_err(|cause| D::RequestAdmission(
            WorthQueryApplicationRequestMutationDenial::ExecutionRequest(cause)))?
    }
}

fn required_local_action<Schema, Operation, Input, Scope>(
    prepared: worth_query_execution::publication_boundary::workflow_advance::PreparedWorkflowAdvance<Schema, Operation, Input, Scope>,
    unavailable: WorthQueryWorkflowAdvancePreparationDenial,
) -> Result<
    worth_query_execution::publication_boundary::workflow_advance::RequiredWorkflowOperation,
    WorthQueryWorkflowAdvancePreparationDenial,
> {
    use worth_query_execution::publication_boundary::workflow_advance::{
        PreparedWorkflowAdvance, WorkflowTransitionPreparationDenial,
    };
    match prepared {
        PreparedWorkflowAdvance::AwaitingOperation(operation) => Ok(operation.into_required()),
        PreparedWorkflowAdvance::ReplayOnly { denial, .. } => Err(
            WorthQueryWorkflowAdvancePreparationDenial::TransitionPreparation(
                WorkflowTransitionPreparationDenial::Attempt(denial),
            ),
        ),
        _ => Err(unavailable),
    }
}
