//! Fresh workflow binding delegates the existing selected authorization owner.
use super::*;
use crate::application_entry::mutation::{
    authorization, WorthQueryApplicationMutationRequestWithIdempotency,
    WorthQueryApplicationRecoveryRequestDenial,
};
use crate::application_entry::WorthQueryMutationSourcePrepared;

pub(in crate::application_entry::operation) enum WorkflowAdmission {
    Operation {
        required: worth_query_execution::publication_boundary::workflow_advance::RequiredWorkflowOperation,
        actor_context: Option<worth_query_execution::publication_boundary::workflow_advance::WorthQueryWorkflowActorContext>,
    },
    Birth(worth_query_execution::publication_boundary::workflow_birth::RequiredWorkflowBirth),
}

impl<S, I, P, A, M> WorthQuerySourceOperationAdmission<S, I, P, A, M>
where
    S: ApplicationSchema,
    I: ApplicationMutationIntent<S>,
    P: ApplicationProgramDefinition<S>,
{
    pub(in crate::application_entry::operation) fn bind_workflow_view<'a>(
        &self,
        native: &mut WorthQueryApplicationMutationRequestWithIdempotency<
            'a,
            'a,
            'a,
            'a,
            S,
            &'a I,
            WorthQueryMutationSourcePrepared,
        >,
        execution: bool,
    ) {
        match &self.workflow {
            Some(WorkflowAdmission::Operation {
                required,
                actor_context,
            }) => {
                native.workflow_actor_context = actor_context.clone();
                native.workflow_captured_input = required
                    .captured_input_contract(
                        &worth_query_execution::publication_boundary::program_publication_access(),
                    )
                    .cloned();
                native.workflow_transition_identity = Some(*required.transition_identity_bytes());
                if execution {
                    native.workflow_authority = Some(required.authority_slot());
                }
            }
            Some(WorkflowAdmission::Birth(required)) => {
                native.bind_retained_workflow_birth(
                    *required.identity_bytes(),
                    execution.then(|| required.authority_slot()),
                );
            }
            None => {}
        }
    }
}
impl<S, I, P, A, M> WorthQuerySourceOperationAdmission<S, I, P, A, M>
where
    S: ApplicationSchema + 'static,
    I: ApplicationMutationIntent<S>,
    P: ApplicationProgramDefinition<S>,
    A: ApplicationAction<S, I::Binding>,
    M: WorthQuerySourceDeliveryMode,
    I::Binding: ApplicationMutationAdmissionBinding<S>,
    <I::Binding as ApplicationMutationAdmissionBinding<S>>::Admission:
        WorthQueryBindingAdmission<S, I::Binding>,
    MutationInput<S, I>: Clone + Send + Sync,
    MutationScope<S, I>: ApplicationMutationScopeResolution<S, MutationPrincipal<S, I>>,
{
    pub(in crate::application_entry) fn authorize_recovery_view<'a>(
        &'a self,
        phase: &WorthQueryAdvancementPhase<'_>,
        request: &'a WorthQueryApplicationRequest<'_, '_, '_, S>,
        native: &mut WorthQueryApplicationMutationRequestWithIdempotency<
            'a,
            'a,
            'a,
            'a,
            S,
            &'a I,
            WorthQueryMutationSourcePrepared,
        >,
    ) -> Result<
        authorization::PreparedMutation<S, I::Binding>,
        WorthQueryApplicationRecoveryRequestDenial,
    > {
        if self.workflow.is_none() {
            return native.authorize_owning_recovery(phase, &self.runtime);
        }
        self.bind_workflow_view(native, false);
        let (selected, _) = self.select(phase, request)?;
        let staged = native.stage()?;
        let identities = native.identities()?;
        <I::Binding as ApplicationMutationAdmissionBinding<S>>::Admission::prepare_selected(
            phase,
            native,
            &identities,
            staged,
            &selected,
        )
        .map_err(Into::into)
    }
}
