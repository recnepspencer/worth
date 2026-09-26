//! Explicit cancellation of a live workflow instance.

use worth_query_declaration::facade::{
    application_capability::ApplicationCapabilityRequest,
    application_operation::{
        ApplicationCapabilityMutationBinding, ApplicationMutationBinding,
        ApplicationMutationIntent, ApplicationMutationScopeResolution,
    },
    application_program::ApplicationWorkflowSpec,
};
use worth_query_execution::facade::{
    application_installation::WorthQueryWorkflowVocabulary,
    workflow_instance_start::{
        PreparedWorkflowInstanceCancellation, PublishedWorkflowInstanceRef,
        WorkflowInstanceCancellationOutcome, WorthQueryWorkflowInstanceStartAdapter,
    },
};
use worth_query_installation::facade::ApplicationSchema;

use super::{
    IntentBinding, MutationInput, MutationOperation, MutationScope,
    WorthQueryWorkflowInstanceStartPreparationDenial,
};
use crate::application_entry::mutation::WorthQueryApplicationMutationRequestWithIdempotency;

impl<'application, 'principal, 'scope, 'key, Schema, Intent, SourcePreparation>
    WorthQueryApplicationMutationRequestWithIdempotency<
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
    /// Ends `instance` where it stands, on the request's branch. Issue it on
    /// the branch whose copy of the instance it ends.
    ///
    /// Cancellation is not rollback: every effect the instance performed
    /// remains, and the outcome names each one. A step admitted before the
    /// cancellation commits goes stale before its effect, and a cancellation
    /// prepared before a step settles goes stale in turn. The start
    /// capability authorizes cancellation. The same key replays exactly; any
    /// other request for a cancelled instance is refused as cancelled.
    pub fn prepare_workflow_instance_cancellation<'workflow, Spec, Program: 'workflow>(
        self,
        workflow: impl Into<WorthQueryWorkflowVocabulary<'workflow, Schema, Spec, Program>>,
        instance: PublishedWorkflowInstanceRef,
    ) -> Result<
        WorthQueryWorkflowInstanceCancellationRequest<
            'application,
            Schema,
            MutationOperation<Schema, Intent>,
            MutationInput<Schema, Intent>,
            MutationScope<Schema, IntentBinding<Schema, Intent>>,
        >,
        WorthQueryWorkflowInstanceStartPreparationDenial,
    >
    where
        Spec: ApplicationWorkflowSpec<Schema = Schema>,
    {
        let (application, prepared, idempotency) =
            self.prepare_instance(workflow, |selected, installed, key, admission| {
                WorthQueryWorkflowInstanceStartAdapter::prepare_cancellation::<
                    Schema,
                    <IntentBinding<Schema, Intent> as ApplicationCapabilityMutationBinding<Schema>>::Capability,
                    MutationOperation<Schema, Intent>,
                    MutationInput<Schema, Intent>,
                    MutationScope<Schema, IntentBinding<Schema, Intent>>,
                    Spec,
                    Program,
                >(selected, installed, instance, key, admission)
            })?;
        Ok(WorthQueryWorkflowInstanceCancellationRequest {
            application,
            prepared,
            idempotency,
        })
    }
}

pub struct WorthQueryWorkflowInstanceCancellationRequest<
    'application,
    Schema,
    Operation,
    Input,
    Scope,
> where
    Schema: ApplicationSchema,
{
    application: &'application worth_query_execution::facade::primary_graph::WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    prepared: PreparedWorkflowInstanceCancellation<Schema, Operation, Input, Scope>,
    idempotency:
        worth_query_execution::facade::primary_graph::WorthQueryApplicationIdempotencyBinding,
}

impl<Schema, Operation, Input, Scope>
    WorthQueryWorkflowInstanceCancellationRequest<'_, Schema, Operation, Input, Scope>
where
    Schema: ApplicationSchema,
    Operation: 'static,
    Input: Clone + Send + Sync + 'static,
{
    pub fn execute(self) -> WorkflowInstanceCancellationOutcome {
        WorthQueryWorkflowInstanceStartAdapter::compare_and_commit_cancellation(
            self.application,
            self.prepared,
            self.idempotency,
        )
    }
}
