//! Restricted ordinary delegation to the existing native operation owner.
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
    Intent: ApplicationMutationIntent<Schema>,
{
    pub fn for_workflow_operation<'workflow, Spec>(
        self,
        workflow: impl Into<WorthQueryWorkflowVocabulary<'workflow, Schema, Spec>>,
        required: &RequiredWorkflowOperation,
    ) -> Result<
        WorthQueryBoundWorkflowOperation<
            'application,
            'principal,
            'scope,
            'key,
            Schema,
            Intent,
            SourcePreparation,
        >,
        WorthQueryWorkflowOperationBindingDenial,
    >
    where
        Spec: ApplicationWorkflowSpec<Schema = Schema>,
        <Intent::Binding as ApplicationMutationBinding<Schema>>::Input: Clone,
    {
        self.native
            .for_workflow_operation(workflow, required)
            .map(|native| WorthQueryBoundWorkflowOperation { native })
    }
    pub fn for_workflow_operation_recovery<'workflow, Spec>(
        self,
        workflow: impl Into<WorthQueryWorkflowVocabulary<'workflow, Schema, Spec>>,
        required: &RequiredWorkflowOperation,
    ) -> Result<
        WorthQueryBoundWorkflowOperation<
            'application,
            'principal,
            'scope,
            'key,
            Schema,
            Intent,
            SourcePreparation,
        >,
        WorthQueryWorkflowOperationBindingDenial,
    >
    where
        Spec: ApplicationWorkflowSpec<Schema = Schema>,
        <Intent::Binding as ApplicationMutationBinding<Schema>>::Input: Clone,
    {
        self.native
            .for_workflow_operation_recovery(workflow, required)
            .map(|native| WorthQueryBoundWorkflowOperation { native })
    }
}
