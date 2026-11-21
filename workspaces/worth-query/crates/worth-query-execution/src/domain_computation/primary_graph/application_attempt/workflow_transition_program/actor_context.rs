//! Original typed command locators bound to L6 workflow vocabulary.
use super::*;
use worth_query_execution_attempt::facade::domain_computation::primary_graph::application_attempt::workflow_transition_program::actor_context::Command;

impl WorthQueryWorkflowAdvanceAdapter {
    /// Own only original locators from the exact installed vocabulary; authorization stays fresh.
    pub fn bind_owned_operation_actor_context<Schema, Spec, Binding>(
        access: &crate::publication_boundary::WorthQueryProgramPublicationAccess,
        workflow: WorthQueryWorkflowVocabulary<'_, Schema, Spec>,
        required: &RequiredWorkflowOperation,
    ) -> Result<Option<WorthQueryWorkflowActorContext>, WorthQueryApplicationAttemptDenial>
    where
        Schema: ApplicationSchema,
        Spec: ApplicationWorkflowSpec<Schema = Schema>,
        Binding: ApplicationMutationBinding<Schema>,
    {
        if required.original_runtime()
            != workflow.runtime().inner.runtime.authority_identity().as_u64()
        {
            return Err(affinity("foreign workflow actor runtime"));
        }
        let installed = workflow
            .workflow_spec_at_revision(required.original_program_revision())
            .ok_or_else(|| {
                affinity("original workflow actor vocabulary revision is unavailable")
            })?;
        Self::bind_operation_actor_context::<Schema, Spec, Binding>(access, installed, required)
    }

    /// Publication alone supplies the original typed command before admission.
    /// The context is an untrusted locator; native selection checks its facts.
    pub fn bind_approval_actor_context<Schema, Spec, Binding>(
        _: &crate::publication_boundary::WorthQueryProgramPublicationAccess,
        selected: &WorthQuerySelectedProductOperation<'_, Schema>,
        workflow: WorthQueryWorkflowVocabulary<'_, Schema, Spec>,
        instance: &PublishedWorkflowInstanceRef,
        required: &RequiredWorkflowApproval,
        proposal: &PublishedWorkflowProposalRef,
    ) -> Result<Option<WorthQueryWorkflowActorContext>, WorthQueryApplicationAttemptDenial>
    where
        Schema: ApplicationSchema,
        Spec: ApplicationWorkflowSpec<Schema = Schema>,
        Binding: ApplicationMutationBinding<Schema>,
    {
        let command = Command::Approval {
            required: required.clone(),
            proposal: proposal.clone(),
        };
        WorthQueryWorkflowActorContext::build::<Schema, Spec, Binding>(
            worth_query_execution_attempt::facade::ports::WorthQueryAttemptCrateSeal::new(),
            selected
                .application()
                .inner
                .runtime
                .authority_identity()
                .as_u64(),
            workflow.workflow_spec_for(selected),
            instance,
            command,
        )
    }
}

fn affinity(subject: &str) -> WorthQueryApplicationAttemptDenial {
    WorthQueryApplicationAttemptDenial::new(
        WorthQueryApplicationAttemptDenialKind::WorkflowTransitionAffinityMismatch,
        subject,
    )
}
