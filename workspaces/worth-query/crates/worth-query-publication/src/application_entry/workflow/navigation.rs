use worth_query_declaration::facade::{
    application_capability::ApplicationCapabilityRequest,
    application_operation::{
        ApplicationCapabilityMutationBinding, ApplicationMutationBinding,
        ApplicationMutationIntent, ApplicationMutationScopeResolution,
    },
    application_program::ApplicationWorkflowSpec,
};
use worth_query_execution::facade::application_installation::WorthQueryWorkflowVocabulary;
use worth_query_execution::publication_boundary::workflow_advance::{
    PerformedWorkflowTransition, PublishedWorkflowInstanceRef, WorkflowProgressOutcome,
};
use worth_query_installation::facade::ApplicationSchema;

use super::progress::{
    IntentBinding, MutationInput, MutationOperation, MutationScope, WorkflowRequestedAction,
    WorthQueryWorkflowAdvancePreparationDenial, WorthQueryWorkflowAdvanceRequest,
};
use crate::application_entry::mutation::WorthQueryApplicationMutationRequestWithIdempotency;

type WorkflowNavigateBackPreparationResult<'application, 'principal, 'scope, Schema, Spec, Intent> =
    Result<
        WorthQueryWorkflowNavigateBackRequest<
            'application,
            'principal,
            'scope,
            Schema,
            Spec,
            MutationOperation<Schema, Intent>,
            MutationInput<Schema, Intent>,
            MutationScope<Schema, IntentBinding<Schema, Intent>>,
        >,
        WorthQueryWorkflowAdvancePreparationDenial,
    >;

/// A prepared backward navigation. `execute` returns the landed transition, or the progress
/// outcome when none landed.
pub struct WorthQueryWorkflowNavigateBackRequest<
    'application,
    'principal,
    'scope,
    Schema,
    Spec,
    Operation,
    Input,
    Scope,
>(
    WorthQueryWorkflowAdvanceRequest<
        'application,
        'principal,
        'scope,
        Schema,
        Spec,
        Operation,
        Input,
        Scope,
    >,
)
where
    Schema: ApplicationSchema,
    Spec: ApplicationWorkflowSpec<Schema = Schema>;

impl<'application, 'principal, 'scope, Schema, Spec, Operation, Input, Scope>
    WorthQueryWorkflowNavigateBackRequest<
        'application,
        'principal,
        'scope,
        Schema,
        Spec,
        Operation,
        Input,
        Scope,
    >
where
    Schema: ApplicationSchema,
    Spec: ApplicationWorkflowSpec<Schema = Schema>,
    Operation: 'static,
    Input: Clone + Send + Sync + 'static,
{
    /// Publishes one Back settlement. The instance head and authority are
    /// rechecked at the owner commit; no effect or prior receipt is replayed.
    pub fn execute(self) -> Result<PerformedWorkflowTransition, WorkflowProgressOutcome> {
        match self.0.execute() {
            WorkflowProgressOutcome::Completed(performed) => Ok(performed),
            other => Err(other),
        }
    }
}

impl<'application, 'principal, 'scope, 'key, Schema, Intent, SourcePreparation>
    WorthQueryApplicationMutationRequestWithIdempotency<
        'application, 'principal, 'scope, 'key, Schema, Intent, SourcePreparation,
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
    pub fn prepare_workflow_navigate_back<Spec>(
        self,
        workflow: impl Into<WorthQueryWorkflowVocabulary<'application, Schema, Spec>>,
        instance: PublishedWorkflowInstanceRef,
    ) -> WorkflowNavigateBackPreparationResult<'application, 'principal, 'scope, Schema, Spec, Intent>
    where
        Spec: ApplicationWorkflowSpec<Schema = Schema>,
    {
        let workflow = workflow.into();
        self.prepare_workflow_request(workflow, instance, WorkflowRequestedAction::NavigateBack)
            .map(WorthQueryWorkflowNavigateBackRequest)
    }
}
