use worth_query_admission::facade::authentication_event::WorthQueryAuthenticationEvent;
use worth_query_declaration::facade::{
    application_capability::ApplicationCapabilityRequest,
    application_operation::{
        ApplicationCapabilityMutationBinding, ApplicationMutationBinding,
        ApplicationMutationIntent, ApplicationMutationScopeBinding,
        ApplicationMutationScopeResolution,
    },
    application_program::ApplicationWorkflowSpec,
};
use worth_query_execution::facade::application_installation::WorthQueryWorkflowVocabulary;
use worth_query_execution::publication_boundary::workflow_advance::{
    PublishedWorkflowInstanceRef, PublishedWorkflowProposalRef, RequiredWorkflowApproval,
    WorkflowApprovalDecision, WorthQueryWorkflowAdvanceAdapter,
};
use worth_query_installation::facade::ApplicationSchema;

use super::{WorthQueryWorkflowAdvancePreparationDenial, WorthQueryWorkflowAdvanceRequest};
use crate::application_entry::{
    mutation::{authorization, WorthQueryApplicationMutationRequestWithIdempotency},
    WorthQueryApplicationRequestMutationDenial,
};

type IntentBinding<Schema, Intent> = <Intent as ApplicationMutationIntent<Schema>>::Binding;
type MutationScope<Schema, Binding> =
    <<Binding as ApplicationMutationBinding<Schema>>::ScopeBinding as ApplicationMutationScopeBinding<
        Schema,
    >>::Scope;
type MutationOperation<Schema, Intent> =
    <IntentBinding<Schema, Intent> as ApplicationMutationBinding<Schema>>::Operation;
type MutationInput<Schema, Intent> =
    <IntentBinding<Schema, Intent> as ApplicationMutationBinding<Schema>>::Input;

type WorkflowApprovalPreparationResult<'application, 'principal, 'scope, Schema, Spec, Intent> =
    Result<
        WorthQueryWorkflowApprovalSigningRequest<
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
    #[allow(clippy::too_many_arguments)]
    pub fn prepare_workflow_approval<Spec>(
        mut self,
        workflow: impl Into<WorthQueryWorkflowVocabulary<'application, Schema, Spec>>,
        instance: PublishedWorkflowInstanceRef,
        required: &RequiredWorkflowApproval,
        proposal: &PublishedWorkflowProposalRef,
        decision: WorkflowApprovalDecision,
    ) -> WorkflowApprovalPreparationResult<'application, 'principal, 'scope, Schema, Spec, Intent>
    where
        Spec: ApplicationWorkflowSpec<Schema = Schema>,
    {
        let workflow = workflow.into();
        let application = self.application_runtime();
        if !std::ptr::eq(application, workflow.runtime()) {
            return Err(WorthQueryWorkflowAdvancePreparationDenial::RuntimeMismatch);
        }
        let selected = application
            .on_branch(self.product_branch())
            .select()
            .map_err(WorthQueryApplicationRequestMutationDenial::ProductSelection)
            .map_err(WorthQueryWorkflowAdvancePreparationDenial::RequestAdmission)?;
        let staged = self.stage().map_err(WorthQueryWorkflowAdvancePreparationDenial::RequestAdmission)?;
        let identities = self
            .identities()
            .map_err(WorthQueryWorkflowAdvancePreparationDenial::RequestAdmission)?;
        let mutation = authorization::prepare_capability_selected(&self, &identities, staged, &selected)
            .map_err(WorthQueryWorkflowAdvancePreparationDenial::RequestAdmission)?;
        let prepared = WorthQueryWorkflowAdvanceAdapter::prepare_approval::<
            Schema,
            <IntentBinding<Schema, Intent> as ApplicationCapabilityMutationBinding<Schema>>::Capability,
            MutationOperation<Schema, Intent>,
            MutationInput<Schema, Intent>,
            MutationScope<Schema, IntentBinding<Schema, Intent>>,
            Spec,
        >(
            &selected,
            workflow,
            instance,
            required,
            proposal,
            decision,
            mutation.admission,
        )
        .map_err(WorthQueryWorkflowAdvancePreparationDenial::TransitionPreparation)?;
        Ok(WorthQueryWorkflowApprovalSigningRequest {
            request: WorthQueryWorkflowAdvanceRequest {
                application,
                principal: self.authenticated_principal(),
                scope: self.request_scope(),
                branch: self.product_branch(),
                workflow,
                prepared,
                idempotency: mutation.idempotency,
            },
        })
    }
}

/// Prepared Query inputs and exact observed signing intent. The ordinary
/// advance request is unavailable until the installed owner admits an event.
pub struct WorthQueryWorkflowApprovalSigningRequest<
    'application,
    'principal,
    'scope,
    Schema,
    Spec,
    Operation,
    Input,
    Scope,
> where
    Schema: ApplicationSchema,
    Spec: ApplicationWorkflowSpec<Schema = Schema>,
{
    request: WorthQueryWorkflowAdvanceRequest<
        'application,
        'principal,
        'scope,
        Schema,
        Spec,
        Operation,
        Input,
        Scope,
    >,
}

impl<'application, 'principal, 'scope, Schema, Spec, Operation, Input, Scope>
    WorthQueryWorkflowApprovalSigningRequest<
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
{
    pub fn authentication_intent(
        &self,
    ) -> Option<
        &worth_query_admission::facade::authentication_event::WorthQueryAuthenticationEventIntent,
    > {
        self.request.prepared.approval_authentication_intent()
    }

    pub fn sign(
        mut self,
        event: &WorthQueryAuthenticationEvent<Schema>,
    ) -> Result<
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
        WorthQueryWorkflowAdvancePreparationDenial,
    > {
        self.request.prepared = self
            .request
            .prepared
            .sign_approval(event, self.request.principal, self.request.scope)
            .map_err(WorthQueryWorkflowAdvancePreparationDenial::Authentication)?;
        Ok(self.request)
    }

    /// A settled approval can be recovered without retaining its historical
    /// authentication event. This cannot publish a new approval transition.
    pub fn execute_replay(
        self,
    ) -> Result<
        worth_query_execution::publication_boundary::workflow_advance::WorkflowProgressOutcome,
        WorthQueryWorkflowAdvancePreparationDenial,
    >
    where
        Operation: 'static,
        Input: Clone + Send + Sync + 'static,
    {
        if !matches!(
            &self.request.prepared,
            worth_query_execution::publication_boundary::workflow_advance::PreparedWorkflowAdvance::ReplayOnly {
                approval: Some(_),
                ..
            }
        ) {
            return Err(WorthQueryWorkflowAdvancePreparationDenial::Authentication(
                worth_query_admission::facade::authentication_event::WorthQueryAuthenticationEventDenial::MissingSigningProof,
            ));
        }
        Ok(self.request.execute())
    }
}
