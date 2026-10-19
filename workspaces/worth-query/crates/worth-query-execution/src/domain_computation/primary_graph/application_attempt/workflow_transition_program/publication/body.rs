//! One native approval and its declared attestation share one authenticated candidate.
use super::*;
use crate::domain_computation::primary_graph::{
    MutationHandlerExecutionDenial, WorthQueryAdvancementPhase, WorthQueryOperationBodyDecision,
    WorthQueryOperationBodyDenial, WorthQueryPrimaryGraphApplicationRuntime,
};
use worth_execution::ExecutionAllocationPolicy;
use worth_query_declaration::facade::application_operation::{
    ApplicationMutationBinding, ApplicationMutationIdentities, ApplicationMutationScopeBinding,
};
use worth_query_installation::facade::ApplicationSchema;

mod program;
type BodyScope<S, B> = <<B as ApplicationMutationBinding<S>>::ScopeBinding as ApplicationMutationScopeBinding<S>>::Scope;
type BodyProgram<S, B> = WorthQueryApplicationEffectProgram<
    S,
    <B as ApplicationMutationBinding<S>>::Operation,
    <B as ApplicationMutationBinding<S>>::Input,
    BodyScope<S, B>,
>;

/// Preparation either owns the fresh signing request or the actual settled native result.
pub enum WorkflowApprovalBodyPreparation<Prepared> {
    Prepared(Prepared),
    Settled(WorkflowProgressOutcome),
}

/// No new mutation was published by a refused body preparation.
#[derive(Debug)]
pub enum WorkflowApprovalBodyDenial {
    Body(WorthQueryOperationBodyDenial),
    Handler(MutationHandlerExecutionDenial),
    Attempt(super::super::super::WorthQueryApplicationAttemptDenial),
    Signing(super::super::super::signed_creation::WorthQuerySignedCreationPreparationDenial),
    Idempotency(
        crate::domain_computation::primary_graph::WorthQueryApplicationIdempotencyResolutionDenial,
    ),
    Authentication(
        worth_query_admission::facade::authentication_event::WorthQueryAuthenticationEventDenial,
    ),
    UnsupportedProfile,
    Cancelled,
    DeadlineExceeded,
}
impl std::fmt::Display for WorkflowApprovalBodyDenial {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "workflow approval body: {self:?}")
    }
}
impl std::error::Error for WorkflowApprovalBodyDenial {}

impl<Schema, Operation, Input, Scope> PreparedWorkflowAdvance<Schema, Operation, Input, Scope>
where
    Schema: ApplicationSchema,
    Operation: 'static,
    Input: Clone + Send + Sync + 'static,
    Scope: worth_query_declaration::facade::application_schema::ApplicationEntityMarkerIdentity<
        Schema,
    >,
{
    pub(in crate::domain_computation::primary_graph) fn prepare_body<Binding>(
        self,
        access: &crate::publication_boundary::WorthQueryProgramPublicationAccess,
        phase: &WorthQueryAdvancementPhase<'_>,
        runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        identities: &ApplicationMutationIdentities<'_, Schema, Binding>,
        principal_identity: &Binding::PrincipalIdentity,
        idempotency: super::super::super::WorthQueryApplicationIdempotencyBinding,
    ) -> Result<WorkflowApprovalBodyPreparation<Self>, WorkflowApprovalBodyDenial>
    where
        Binding: ApplicationMutationBinding<
            Schema,
            Operation = Operation,
            Input = Input,
            Result = (),
            Denial = WorthQueryOperationBodyDenial,
            Decision = WorthQueryOperationBodyDecision<Schema, Scope>,
        >,
        Binding::ScopeBinding: ApplicationMutationScopeBinding<Schema, Scope = Scope>,
    {
        self.validate_approval_descriptor()
            .map_err(WorkflowApprovalBodyDenial::Authentication)?;
        let request = phase
            .execution_request_for(&runtime.product_runtime)
            .map_err(|cause| {
                WorkflowApprovalBodyDenial::Handler(
                    MutationHandlerExecutionDenial::ForeignAdvancementPhase(cause),
                )
            })?;
        if let Some(settled) = self
            .resolve_transition_replay(&request, runtime, idempotency)
            .map_err(WorkflowApprovalBodyDenial::Idempotency)?
        {
            return Ok(WorkflowApprovalBodyPreparation::Settled(settled));
        }
        if matches!(self, Self::ReplayOnly { .. }) {
            return Ok(WorkflowApprovalBodyPreparation::Settled(
                runtime.compare_and_commit_workflow_advance(&request, phase, self, idempotency),
            ));
        }
        let Self::Transition {
            program,
            program_revision,
            transition_identity,
            transition_identity_bytes,
            transition_identity_locator,
            assessment_identity_locator,
            instance,
            node_path,
            terminal,
            assessment,
            supporting_identity,
            operation_receipt_identity,
            progress_update,
            approval,
            approval_identity,
            mut approval_authentication,
            replays,
        } = self
        else {
            return Err(WorkflowApprovalBodyDenial::UnsupportedProfile);
        };
        if approval.is_none() || approval_authentication.is_none() {
            return Err(WorkflowApprovalBodyDenial::UnsupportedProfile);
        }
        let program = program::compose::<Schema, Binding>(
            access,
            runtime,
            phase,
            identities,
            principal_identity,
            program,
        )?;
        let intent = super::super::super::signed_creation::WorthQuerySignedCreationAdapter::authentication_intent(&program)
            .ok_or(WorkflowApprovalBodyDenial::UnsupportedProfile)?.clone();
        approval_authentication
            .as_mut()
            .expect("native approval authentication")
            .bind_signed_body(intent)
            .map_err(WorkflowApprovalBodyDenial::Authentication)?;
        Ok(WorkflowApprovalBodyPreparation::Prepared(
            Self::Transition {
                program,
                program_revision,
                transition_identity,
                transition_identity_bytes,
                transition_identity_locator,
                assessment_identity_locator,
                instance,
                node_path,
                terminal,
                assessment,
                supporting_identity,
                operation_receipt_identity,
                progress_update,
                approval,
                approval_identity,
                approval_authentication,
                replays,
            },
        ))
    }
}
