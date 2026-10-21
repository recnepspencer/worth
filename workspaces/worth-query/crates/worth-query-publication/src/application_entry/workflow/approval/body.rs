//! The explicit ordinary entrance for an approval's declared signed body.
use super::*;
use worth_query_execution::facade::primary_graph::{
    WorthQueryOperationBodyDecision, WorthQueryOperationBodyDenial,
};
use worth_query_execution::publication_boundary::{
    program_publication_access, workflow_advance::WorkflowApprovalBodyPreparation,
};

type PreparedBody<'a, 'p, 's, S, W, I> = Result<
    WorkflowApprovalBodyPreparation<ApprovalRequest<'a, 'p, 's, S, W, I>>,
    WorthQueryWorkflowAdvancePreparationDenial,
>;

impl<'application, 'principal, 'scope, 'key, Schema, Intent, SourcePreparation>
    WorthQueryApplicationMutationRequestWithIdempotency<'application, 'principal, 'scope, 'key, Schema, Intent, SourcePreparation>
where
    Schema: ApplicationSchema,
    Intent: ApplicationMutationIntent<Schema> + Clone,
    IntentBinding<Schema, Intent>: ApplicationCapabilityMutationBinding<Schema>
        + ApplicationMutationBinding<Schema, Result = (), Denial = WorthQueryOperationBodyDenial,
            Decision = WorthQueryOperationBodyDecision<Schema, MutationScope<Schema, IntentBinding<Schema, Intent>>>>,
    <IntentBinding<Schema, Intent> as ApplicationMutationBinding<Schema>>::ScopeBinding: ApplicationMutationScopeResolution<Schema,
        <IntentBinding<Schema, Intent> as ApplicationMutationBinding<Schema>>::PrincipalIdentity>,
    MutationInput<Schema, Intent>: Clone + Send + Sync + 'static + ApplicationCapabilityRequest<Schema,
        <IntentBinding<Schema, Intent> as ApplicationCapabilityMutationBinding<Schema>>::Capability,
        Scope = MutationScope<Schema, IntentBinding<Schema, Intent>>>,
{
    /// Prepares one atomic native approval+attestation candidate. Exact settled
    /// replay returns before body evaluation and needs no new signing event.
    pub fn prepare_workflow_approval_with_body<Spec>(
        self,
        workflow: impl Into<WorthQueryWorkflowVocabulary<'application, Schema, Spec>>,
        instance: PublishedWorkflowInstanceRef,
        required: &RequiredWorkflowApproval,
        proposal: &PublishedWorkflowProposalRef,
        decision: WorkflowApprovalDecision,
    ) -> PreparedBody<'application, 'principal, 'scope, Schema, Spec, Intent>
    where Spec: ApplicationWorkflowSpec<Schema = Schema>,
    {
        self.prepare_approval_using(workflow, instance, required, proposal, decision, ApprovalBodyPosture::WithBody,
            |phase, identities, principal_identity, signing| {
                let WorthQueryWorkflowAdvanceRequest { application, principal, scope, branch, workflow, prepared, idempotency } = signing.request;
                match WorthQueryWorkflowAdvanceAdapter::prepare_body::<Schema, IntentBinding<Schema, Intent>>(
                    &program_publication_access(), phase, application, prepared, identities, &principal_identity, idempotency,
                ).map_err(WorthQueryWorkflowAdvancePreparationDenial::ApprovalBody)? {
                    WorkflowApprovalBodyPreparation::Prepared(prepared) => Ok(WorkflowApprovalBodyPreparation::Prepared(
                        WorthQueryWorkflowApprovalSigningRequest { request: WorthQueryWorkflowAdvanceRequest {
                            application, principal, scope, branch, workflow, prepared, idempotency,
                        } },
                    )),
                    WorkflowApprovalBodyPreparation::Settled(outcome) => Ok(WorkflowApprovalBodyPreparation::Settled(outcome)),
                }
            })
    }
}

impl<'application, 'principal, 'scope, 'key, Schema, Intent, SourcePreparation>
    crate::application_entry::WorthQueryPendingApplicationLane<'application, 'principal, 'scope, 'key, Schema, Intent, SourcePreparation>
where
    Schema: ApplicationSchema,
    Intent: ApplicationMutationIntent<Schema> + Clone,
    IntentBinding<Schema, Intent>: ApplicationCapabilityMutationBinding<Schema>
        + ApplicationMutationBinding<Schema, Result = (), Denial = WorthQueryOperationBodyDenial,
            Decision = WorthQueryOperationBodyDecision<Schema, MutationScope<Schema, IntentBinding<Schema, Intent>>>>,
    <IntentBinding<Schema, Intent> as ApplicationMutationBinding<Schema>>::ScopeBinding: ApplicationMutationScopeResolution<Schema,
        <IntentBinding<Schema, Intent> as ApplicationMutationBinding<Schema>>::PrincipalIdentity>,
    MutationInput<Schema, Intent>: Clone + Send + Sync + 'static + ApplicationCapabilityRequest<Schema,
        <IntentBinding<Schema, Intent> as ApplicationCapabilityMutationBinding<Schema>>::Capability,
        Scope = MutationScope<Schema, IntentBinding<Schema, Intent>>>,
{
    pub fn prepare_workflow_approval_with_body<Spec>(
        self,
        workflow: impl Into<WorthQueryWorkflowVocabulary<'application, Schema, Spec>>,
        instance: PublishedWorkflowInstanceRef,
        required: &RequiredWorkflowApproval,
        proposal: &PublishedWorkflowProposalRef,
        decision: WorkflowApprovalDecision,
    ) -> PreparedBody<'application, 'principal, 'scope, Schema, Spec, Intent>
    where Spec: ApplicationWorkflowSpec<Schema = Schema>,
    {
        self.native.prepare_workflow_approval_with_body(workflow, instance, required, proposal, decision)
    }
}
