//! Local business actions, their completion and actor-context binding: each entry that
//! holds a selection lends the attempt its basis; the attempt keeps the native admission,
//! settlement and context construction.
use super::*;
use crate::domain_computation::primary_graph::vocabulary::application_attempt::reference::PublishedWorkflowProposalRef;
use crate::domain_computation::primary_graph::vocabulary::application_attempt::workflow_instance_program_publication::PublishedWorkflowInstanceRef;
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationAttemptDenial, WorthQueryApplicationCommitReceipt,
};
use worth_query_declaration::facade::application_operation::{
    ApplicationMutationAdmissionBinding, ApplicationMutationBinding, ApplicationMutationIdentities,
    ApplicationMutationScopeBinding,
};
use worth_query_declaration::facade::application_schema::ApplicationStructuredValueBinding;

impl WorthQueryWorkflowAdvanceAdapter {
    #[allow(clippy::too_many_arguments)]
    pub fn prepare_local_action<S, O, I, Scope, Spec, B>(
        phase: &crate::domain_computation::request_execution::WorthQueryAdvancementPhase<'_>,
        selected: &WorthQuerySelectedProductOperation<'_, S>,
        installed: &WorthQueryInstalledApplicationWorkflowSpec<S, Spec>,
        instance: PublishedWorkflowInstanceRef,
        admission: WorthQueryAdmittedApplicationOperation<S, O, I, Scope>,
        identities: &ApplicationMutationIdentities<'_, S, B>,
        kind: WorkflowLocalActionKind,
    ) -> Result<PreparedWorkflowAdvance<S, O, I, Scope>, WorkflowTransitionPreparationDenial>
    where
        S: ApplicationSchema,
        O: ApplicationOperationMarkerIdentity<S> + 'static,
        Spec: ApplicationWorkflowSpec<Schema = S>,
        B: ApplicationMutationAdmissionBinding<S, Operation = O, Input = I>,
        I: 'static,
        O::InputBinding: ApplicationStructuredValueBinding<Value = I>,
    {
        crate::domain_computation::primary_graph::application_attempt::workflow_transition_program::prepare_local_action::<S, O, I, Scope, Spec, B>(
            phase,
            &selected.workflow_selection_basis(),
            installed,
            instance,
            admission,
            identities,
            kind,
        )
    }

    /// Reobserve the original route only after exact committed effect evidence.
    /// No old operation authority is restored or made executable again.
    #[allow(clippy::too_many_arguments)]
    pub fn complete_local_operation<S, Spec, B>(
        phase: &crate::domain_computation::request_execution::WorthQueryAdvancementPhase<'_>,
        selected: &WorthQuerySelectedProductOperation<'_, S>,
        workflow: WorthQueryWorkflowVocabulary<'_, S, Spec>,
        required: &RequiredWorkflowOperation,
        admission: WorthQueryAdmittedApplicationOperation<
            S,
            B::Operation,
            B::Input,
            <B::ScopeBinding as ApplicationMutationScopeBinding<S>>::Scope,
        >,
        idempotency: WorthQueryApplicationIdempotencyBinding,
        receipt: &WorthQueryApplicationCommitReceipt,
        recovery: Option<
            &crate::domain_computation::application_aftermath::WorthQueryRecoverySafeRetryAdmission,
        >,
    ) -> Result<WorkflowProgressOutcome, WorkflowTransitionPreparationDenial>
    where
        S: ApplicationSchema,
        Spec: ApplicationWorkflowSpec<Schema = S>,
        B: ApplicationMutationAdmissionBinding<S>,
        B::Operation: ApplicationOperationMarkerIdentity<S>,
        B::Input: Clone + Send + Sync + 'static,
        <B::Operation as ApplicationOperationMarkerIdentity<S>>::InputBinding:
            ApplicationStructuredValueBinding<Value = B::Input>,
    {
        crate::domain_computation::primary_graph::application_attempt::workflow_transition_program::complete_local_operation::<S, Spec, B>(
            phase,
            &selected.workflow_selection_basis(),
            workflow,
            required,
            admission,
            idempotency,
            receipt,
            recovery,
        )
    }

    /// Own only original locators from the exact installed vocabulary; authorization stays fresh.
    pub fn bind_owned_operation_actor_context<Schema, Spec, Binding>(
        access: &crate::publication_boundary::WorthQueryProgramPublicationAccess,
        workflow: WorthQueryWorkflowVocabulary<'_, Schema, Spec>,
        required: &RequiredWorkflowOperation,
    ) -> Result<Option<crate::domain_computation::primary_graph::application_attempt::WorthQueryWorkflowActorContext>, WorthQueryApplicationAttemptDenial>
    where
        Schema: ApplicationSchema,
        Spec: ApplicationWorkflowSpec<Schema = Schema>,
        Binding: ApplicationMutationBinding<Schema>,
{
        crate::domain_computation::primary_graph::application_attempt::WorthQueryWorkflowActorContext::bind_owned_operation_actor_context::<Schema, Spec, Binding>(
            access, workflow, required,
        )
    }

    /// Publication alone supplies the original typed command before admission.
    /// The context is an untrusted locator; native selection checks its facts.
    pub fn bind_approval_actor_context<Schema, Spec, Binding>(
        access: &crate::publication_boundary::WorthQueryProgramPublicationAccess,
        selected: &WorthQuerySelectedProductOperation<'_, Schema>,
        workflow: WorthQueryWorkflowVocabulary<'_, Schema, Spec>,
        instance: &PublishedWorkflowInstanceRef,
        required: &RequiredWorkflowApproval,
        proposal: &PublishedWorkflowProposalRef,
    ) -> Result<Option<crate::domain_computation::primary_graph::application_attempt::WorthQueryWorkflowActorContext>, WorthQueryApplicationAttemptDenial>
    where
        Schema: ApplicationSchema,
        Spec: ApplicationWorkflowSpec<Schema = Schema>,
        Binding: ApplicationMutationBinding<Schema>,
{
        crate::domain_computation::primary_graph::application_attempt::WorthQueryWorkflowActorContext::bind_approval_actor_context::<Schema, Spec, Binding>(
            access,
            selected.application().authority_identity().as_u64(),
            workflow.workflow_spec_for(selected),
            instance,
            required,
            proposal,
        )
    }

    pub fn bind_operation_actor_context<Schema, Spec, Binding>(
        access: &crate::publication_boundary::WorthQueryProgramPublicationAccess,
        installed: &WorthQueryInstalledApplicationWorkflowSpec<Schema, Spec>,
        required: &RequiredWorkflowOperation,
    ) -> Result<Option<crate::domain_computation::primary_graph::application_attempt::WorthQueryWorkflowActorContext>, WorthQueryApplicationAttemptDenial>
    where
        Schema: ApplicationSchema,
        Spec: ApplicationWorkflowSpec<Schema = Schema>,
        Binding: ApplicationMutationBinding<Schema>,
{
        crate::domain_computation::primary_graph::application_attempt::WorthQueryWorkflowActorContext::bind_operation_actor_context::<Schema, Spec, Binding>(access, installed, required)
    }
}
