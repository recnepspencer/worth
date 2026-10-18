//! Ordinary workflow delegation preserves the existing native workflow owner.
use super::*;

impl<'application, 'principal, 'scope, Schema, Intent>
    crate::application_entry::WorthQueryOperationBuilder<
        'application,
        'principal,
        'scope,
        Schema,
        Intent,
    >
where
    Schema: ApplicationSchema,
    Intent: ApplicationMutationIntent<Schema>,
{
    pub fn run_workflow<Spec>(
        self,
        workflow: impl Into<WorthQueryWorkflowVocabulary<'application, Schema, Spec>>,
        instance: PublishedWorkflowInstanceRef,
    ) -> WorthQueryOrdinaryWorkflowRun<'application, 'principal, 'scope, Schema, Intent, Spec>
    where
        Spec: ApplicationWorkflowSpec<Schema = Schema>,
    {
        self.native.run_workflow(workflow, instance)
    }
}
