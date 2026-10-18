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
    pub fn workflow<'workflow, Spec>(
        self,
        phase: &worth_query_execution::facade::application_contribution::WorthQueryAdvancementPhase<
            '_,
        >,
        workflow: impl Into<WorthQueryWorkflowVocabulary<'workflow, Schema, Spec>>,
        authored: AuthoredWorkflowDefinition<Spec>,
    ) -> Result<
        WorthQueryOrdinaryWorkflowDraft<'application, 'principal, 'scope, Schema, Intent, Spec>,
        WorthQueryOrdinaryWorkflowPublicationDenial,
    >
    where
        Spec: ApplicationWorkflowSpec<Schema = Schema>,
    {
        self.native.workflow(phase, workflow, authored)
    }
}
