//! Restricted ordinary delegation to the existing native proposal owner.
use super::*;

impl<'application, 'principal, 'scope, 'key, Schema, Intent, SourcePreparation>
    crate::application_entry::WorthQueryPendingApplicationLane<'application, 'principal, 'scope, 'key, Schema, Intent, SourcePreparation>
where
    Schema: ApplicationSchema,
    Intent: ApplicationMutationIntent<Schema> + Clone,
    IntentBinding<Schema, Intent>: ApplicationCapabilityMutationBinding<Schema>,
    <IntentBinding<Schema, Intent> as ApplicationMutationBinding<Schema>>::ScopeBinding:
        ApplicationMutationScopeResolution<Schema, <IntentBinding<Schema, Intent> as ApplicationMutationBinding<Schema>>::PrincipalIdentity>,
    MutationInput<Schema, Intent>: Clone + Send + Sync + 'static
        + ApplicationCapabilityRequest<
            Schema,
            <IntentBinding<Schema, Intent> as ApplicationCapabilityMutationBinding<Schema>>::Capability,
            Scope = MutationScope<Schema, IntentBinding<Schema, Intent>>,
        >,
{
    pub fn prepare_workflow_proposal<'workflow, Spec>(
        self,
        workflow: impl Into<WorthQueryWorkflowVocabulary<'workflow, Schema, Spec>>,
        instance: PublishedWorkflowInstanceRef,
    ) -> Result<WorthQueryWorkflowProposalRequest<'application, Schema, MutationOperation<Schema, Intent>, MutationInput<Schema, Intent>, MutationScope<Schema, IntentBinding<Schema, Intent>>>, WorthQueryWorkflowProposalPreparationDenial>
    where
        Spec: ApplicationWorkflowSpec<Schema = Schema>,
    {
        self.native.prepare_workflow_proposal(workflow, instance)
    }

    pub fn prepare_workflow_proposal_with_inputs<'workflow, Spec, Program, R>(
        self,
        workflow: impl Into<WorthQueryWorkflowVocabulary<'workflow, Schema, Spec>>,
        instance: PublishedWorkflowInstanceRef,
        inputs: &crate::application_entry::WorthQueryWorkflowOperationInputs<Schema, Program, R>,
    ) -> Result<WorthQueryWorkflowProposalRequest<'application, Schema, MutationOperation<Schema, Intent>, MutationInput<Schema, Intent>, MutationScope<Schema, IntentBinding<Schema, Intent>>>, WorthQueryWorkflowProposalPreparationDenial>
    where Schema: 'static,
        Spec: ApplicationWorkflowSpec<Schema = Schema>,
        Program: worth_query_declaration::facade::application_program::ApplicationProgramDefinition<Schema>,
        R: crate::application_entry::WorthQueryWorkflowInputList<Schema, Program>,
    {
        self.native.prepare_workflow_proposal_with_inputs(workflow, instance, inputs)
    }
}
