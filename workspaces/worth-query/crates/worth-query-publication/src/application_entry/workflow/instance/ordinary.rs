//! Restricted ordinary delegation to the existing native instance owner.
use super::*;

impl<'application, 'principal, 'scope, 'key, Schema, Intent, SourcePreparation>
    crate::application_entry::WorthQueryPendingApplicationLane<
        'application,
        'principal,
        'scope,
        'key,
        Schema,
        Intent,
        SourcePreparation,
    >
where
    Schema: ApplicationSchema,
    Intent: ApplicationMutationIntent<Schema> + Clone,
    IntentBinding<Schema, Intent>: ApplicationCapabilityMutationBinding<Schema>,
    <IntentBinding<Schema, Intent> as ApplicationMutationBinding<Schema>>::ScopeBinding:
        ApplicationMutationScopeResolution<
            Schema,
            <IntentBinding<Schema, Intent> as ApplicationMutationBinding<Schema>>::PrincipalIdentity,
        >,
    MutationInput<Schema, Intent>: Clone
        + ApplicationCapabilityRequest<
            Schema,
            <IntentBinding<Schema, Intent> as ApplicationCapabilityMutationBinding<Schema>>::Capability,
            Scope = MutationScope<Schema, IntentBinding<Schema, Intent>>,
        >,
{
    pub fn prepare_workflow_instance_start<'workflow, Spec>(
        self,
        workflow: impl Into<WorthQueryWorkflowVocabulary<'workflow, Schema, Spec>>,
        definition: PublishedWorkflowDefinitionRef,
    ) -> Result<
        WorthQueryWorkflowInstanceStartRequest<
            'application,
            Schema,
            MutationOperation<Schema, Intent>,
            MutationInput<Schema, Intent>,
            MutationScope<Schema, IntentBinding<Schema, Intent>>,
        >,
        WorthQueryWorkflowInstancePreparationDenial,
    >
    where
        Spec: ApplicationWorkflowSpec<Schema = Schema>,
    {
        self.native.prepare_workflow_instance_start(workflow, definition)
    }
    pub fn prepare_workflow_instance_migration<'workflow, Spec>(
        self,
        workflow: impl Into<WorthQueryWorkflowVocabulary<'workflow, Schema, Spec>>,
        instance: PublishedWorkflowInstanceRef,
        target: PublishedWorkflowDefinitionRef,
        resume_at: &str,
    ) -> Result<
        WorthQueryWorkflowInstanceStartRequest<
            'application,
            Schema,
            MutationOperation<Schema, Intent>,
            MutationInput<Schema, Intent>,
            MutationScope<Schema, IntentBinding<Schema, Intent>>,
        >,
        WorthQueryWorkflowInstancePreparationDenial,
    >
    where
        Spec: ApplicationWorkflowSpec<Schema = Schema>,
    {
        self.native.prepare_workflow_instance_migration(workflow, instance, target, resume_at)
    }
    pub fn prepare_workflow_fork_continuation<'workflow, Spec>(
        self,
        workflow: impl Into<WorthQueryWorkflowVocabulary<'workflow, Schema, Spec>>,
        instance: PublishedWorkflowInstanceRef,
        target: PublishedWorkflowDefinitionRef,
        resume_at: &str,
    ) -> Result<
        WorthQueryWorkflowInstanceStartRequest<
            'application,
            Schema,
            MutationOperation<Schema, Intent>,
            MutationInput<Schema, Intent>,
            MutationScope<Schema, IntentBinding<Schema, Intent>>,
        >,
        WorthQueryWorkflowInstancePreparationDenial,
    >
    where
        Spec: ApplicationWorkflowSpec<Schema = Schema>,
    {
        self.native.prepare_workflow_fork_continuation(workflow, instance, target, resume_at)
    }
}
