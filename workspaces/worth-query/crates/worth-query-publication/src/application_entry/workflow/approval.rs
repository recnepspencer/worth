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

type ApprovalRequest<'application, 'principal, 'scope, Schema, Spec, Intent> =
    WorthQueryWorkflowApprovalSigningRequest<
        'application,
        'principal,
        'scope,
        Schema,
        Spec,
        MutationOperation<Schema, Intent>,
        MutationInput<Schema, Intent>,
        MutationScope<Schema, IntentBinding<Schema, Intent>>,
    >;
type WorkflowApprovalPreparationResult<'application, 'principal, 'scope, Schema, Spec, Intent> =
    Result<
        ApprovalRequest<'application, 'principal, 'scope, Schema, Spec, Intent>,
        WorthQueryWorkflowAdvancePreparationDenial,
    >;

#[derive(Clone, Copy)]
enum ApprovalBodyPosture {
    SignOnly,
    WithBody,
}

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
        self,
        workflow: impl Into<WorthQueryWorkflowVocabulary<'application, Schema, Spec>>,
        instance: PublishedWorkflowInstanceRef,
        required: &RequiredWorkflowApproval,
        proposal: &PublishedWorkflowProposalRef,
        decision: WorkflowApprovalDecision,
    ) -> WorkflowApprovalPreparationResult<'application, 'principal, 'scope, Schema, Spec, Intent>
    where
        Spec: ApplicationWorkflowSpec<Schema = Schema>,
    {
        self.prepare_approval_using(workflow, instance, required, proposal, decision, ApprovalBodyPosture::SignOnly,
            |_, _, _, request| Ok(request))
    }

    #[allow(clippy::too_many_arguments)]
    fn prepare_approval_using<Spec, Output>(
        mut self,
        workflow: impl Into<WorthQueryWorkflowVocabulary<'application, Schema, Spec>>,
        instance: PublishedWorkflowInstanceRef,
        required: &RequiredWorkflowApproval,
        proposal: &PublishedWorkflowProposalRef,
        decision: WorkflowApprovalDecision,
        posture: ApprovalBodyPosture,
        complete: impl FnOnce(
            &worth_query_execution::facade::application_contribution::WorthQueryAdvancementPhase<'_>,
            &worth_query_declaration::facade::application_operation::ApplicationMutationIdentities<'_, Schema, IntentBinding<Schema, Intent>>,
            <IntentBinding<Schema, Intent> as ApplicationMutationBinding<Schema>>::PrincipalIdentity,
            ApprovalRequest<'application, 'principal, 'scope, Schema, Spec, Intent>,
        ) -> Result<Output, WorthQueryWorkflowAdvancePreparationDenial>,
    ) -> Result<Output, WorthQueryWorkflowAdvancePreparationDenial>
    where Spec: ApplicationWorkflowSpec<Schema = Schema>,
    {
        let request_scope = self.request_scope().clone();
        let runtime = self.application_runtime();
        runtime.with_application_advancement(&request_scope, |phase| {

        let workflow = workflow.into();
        let application = self.application_runtime();
        if !std::ptr::eq(application, workflow.runtime()) {
            return Err(WorthQueryWorkflowAdvancePreparationDenial::RuntimeMismatch);
        }
        let selected = application
            .on_branch(self.product_branch())
            .select(&phase, )
            .map_err(WorthQueryApplicationRequestMutationDenial::ProductSelection)
            .map_err(WorthQueryWorkflowAdvancePreparationDenial::RequestAdmission)?;
        let staged = self.stage().map_err(WorthQueryWorkflowAdvancePreparationDenial::RequestAdmission)?;
        self.workflow_actor_context = WorthQueryWorkflowAdvanceAdapter::bind_approval_actor_context::<Schema,Spec,IntentBinding<Schema,Intent>>(
            &worth_query_execution::publication_boundary::program_publication_access(),
            &selected, workflow, &instance, required, proposal,
        ).map_err(|cause| WorthQueryWorkflowAdvancePreparationDenial::TransitionPreparation(
            worth_query_execution::publication_boundary::workflow_advance::WorkflowTransitionPreparationDenial::Attempt(cause)))?;
        let identities = self
            .identities()
            .map_err(WorthQueryWorkflowAdvancePreparationDenial::RequestAdmission)?;
        let mutation = authorization::prepare_capability_selected(&phase, &self, &identities, staged, &selected)
            .map_err(WorthQueryWorkflowAdvancePreparationDenial::RequestAdmission)?;
        let body = mutation.admission.allowed_graph_contract().body();
        match posture {
            ApprovalBodyPosture::SignOnly if body.is_some() => return Err(WorthQueryWorkflowAdvancePreparationDenial::ApprovalBodyRequired),
            ApprovalBodyPosture::WithBody if body.and_then(|body| body.creation()).and_then(|creation| creation.signing()).is_none() =>
                return Err(WorthQueryWorkflowAdvancePreparationDenial::ApprovalBodyUnavailable),
            _ => {}
        }
        let prepared = WorthQueryWorkflowAdvanceAdapter::prepare_approval::<
            Schema,
            <IntentBinding<Schema, Intent> as ApplicationCapabilityMutationBinding<Schema>>::Capability,
            MutationOperation<Schema, Intent>,
            MutationInput<Schema, Intent>,
            MutationScope<Schema, IntentBinding<Schema, Intent>>,
            Spec,
        >(&phase,
            &selected,
            workflow,
            instance,
            required,
            proposal,
            decision,
            mutation.admission,
        )
        .map_err(WorthQueryWorkflowAdvancePreparationDenial::TransitionPreparation)?;
        complete(&phase, &identities, mutation.principal_identity, WorthQueryWorkflowApprovalSigningRequest {
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

        }).map_err(|cause| WorthQueryWorkflowAdvancePreparationDenial::RequestAdmission(WorthQueryApplicationRequestMutationDenial::ExecutionRequest(cause)))?
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

mod body;
mod ordinary;
