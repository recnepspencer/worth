//! The workflow proposal program's publication adapter. Its preparation holds the
//! selection and hands the attempt its `WorkflowSelectionBasis`.
use crate::domain_computation::primary_graph::application_attempt::WorkflowProposalProgramPublicationScopedOps;
use crate::domain_computation::primary_graph::application_attempt::{
    PreparedWorkflowProposal, WorkflowProposalOutcome, WorkflowProposalPreparationDenial,
};
use crate::domain_computation::primary_graph::vocabulary::application_attempt::workflow_instance_program_publication::PublishedWorkflowInstanceRef;
use crate::domain_computation::primary_graph::{
    WorthQueryAdmittedApplicationOperation, WorthQueryApplicationIdempotencyBinding,
    WorthQueryApplicationIdempotencyResolutionDenial, WorthQuerySelectedProductOperation,
};
use crate::domain_computation::request_execution::WorthQueryAdvancementPhase;
use crate::domain_computation::vocabulary::authorization::denial::WorthQueryOperationAuthorizationRefusalKind as AuthorizationRefusal;
use worth_query_declaration::facade::{
    application_program::ApplicationWorkflowSpec,
    application_schema::{ApplicationOperationMarkerIdentity, ApplicationStructuredValueBinding},
};
use worth_query_installation::facade::{
    ApplicationSchema, WorthQueryInstalledApplicationWorkflowSpec,
};

#[doc(hidden)]
pub struct WorthQueryWorkflowProposalAdapter;

impl WorthQueryWorkflowProposalAdapter {
    pub fn resolve_replay<Schema, Operation, Input, Scope>(
        phase: &crate::domain_computation::request_execution::WorthQueryAdvancementPhase<'_>,
        runtime: &dyn crate::domain_computation::primary_graph::application_attempt::host::ApplicationAttemptHost<Schema>,
        admission: &WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>,
        idempotency: WorthQueryApplicationIdempotencyBinding,
        instance: PublishedWorkflowInstanceRef,
    ) -> Result<Option<WorkflowProposalOutcome>, WorthQueryApplicationIdempotencyResolutionDenial>
    where
        Schema: ApplicationSchema,
        Operation: ApplicationOperationMarkerIdentity<Schema> + 'static,
        Input: Clone + Send + Sync + 'static,
    {
        Self::resolve_replay_with_inputs(
            phase,
            runtime,
            admission,
            idempotency,
            instance,
            &crate::domain_computation::primary_graph::application_attempt::workflow::captured_input::WorkflowCapturedInputs::empty(),
        )
    }

    pub fn resolve_replay_with_inputs<Schema, Operation, Input, Scope>(
        phase: &WorthQueryAdvancementPhase<'_>,
        runtime: &dyn crate::domain_computation::primary_graph::application_attempt::host::ApplicationAttemptHost<Schema>,
        admission: &WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>,
        idempotency: WorthQueryApplicationIdempotencyBinding,
        instance: PublishedWorkflowInstanceRef,
        captured: &crate::domain_computation::primary_graph::application_attempt::workflow::captured_input::WorkflowCapturedInputs,
    ) -> Result<Option<WorkflowProposalOutcome>, WorthQueryApplicationIdempotencyResolutionDenial>
    where
        Schema: ApplicationSchema,
        Operation: ApplicationOperationMarkerIdentity<Schema> + 'static,
        Input: Clone + Send + Sync + 'static,
    {
        let idempotency = idempotency.bind_workflow_captured_inputs(captured.identity);
        let phase_request = match phase.execution_request_for(runtime.product_runtime()) {
            Ok(request) => request,
            Err(_) => return Err(crate::domain_computation::primary_graph::WorthQueryApplicationIdempotencyResolutionDenial::from_authorization(crate::domain_computation::primary_graph::WorthQueryOperationAuthorizationDenial::new(crate::domain_computation::vocabulary::authorization::denial::WorthQueryOperationAuthorizationDenialKind::Refused(AuthorizationRefusal::ForeignPhase), "workflow-phase"))),
        };
        let execution = &phase_request;

        runtime.resolve_workflow_proposal_replay(
            execution,
            admission,
            idempotency,
            instance,
            Operation::IDENTIFIER.to_owned(),
            <Operation::InputBinding as ApplicationStructuredValueBinding>::IDENTITY
                .as_str()
                .to_owned(),
            *idempotency.intent_identity(),
            idempotency.source_identity(),
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn prepare<Schema, Operation, Input, Scope, Spec>(
        phase: &crate::domain_computation::request_execution::WorthQueryAdvancementPhase<'_>,
        selected: &WorthQuerySelectedProductOperation<'_, Schema>,
        installed: &WorthQueryInstalledApplicationWorkflowSpec<Schema, Spec>,
        instance: PublishedWorkflowInstanceRef,
        admission: WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>,
        idempotency: &WorthQueryApplicationIdempotencyBinding,
    ) -> Result<
        PreparedWorkflowProposal<Schema, Operation, Input, Scope>,
        WorkflowProposalPreparationDenial,
    >
    where
        Schema: ApplicationSchema,
        Operation: ApplicationOperationMarkerIdentity<Schema> + 'static,
        Spec: ApplicationWorkflowSpec<Schema = Schema>,
    {
        Self::prepare_with_inputs(
            phase,
            selected,
            installed,
            instance,
            admission,
            idempotency,
            crate::domain_computation::primary_graph::application_attempt::workflow::captured_input::WorkflowCapturedInputs::empty(),
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn prepare_with_inputs<Schema, Operation, Input, Scope, Spec>(
        phase: &WorthQueryAdvancementPhase<'_>,
        selected: &WorthQuerySelectedProductOperation<'_, Schema>,
        installed: &WorthQueryInstalledApplicationWorkflowSpec<Schema, Spec>,
        instance: PublishedWorkflowInstanceRef,
        admission: WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>,
        idempotency: &WorthQueryApplicationIdempotencyBinding,
        captured: crate::domain_computation::primary_graph::application_attempt::workflow::captured_input::WorkflowCapturedInputs,
    ) -> Result<
        PreparedWorkflowProposal<Schema, Operation, Input, Scope>,
        WorkflowProposalPreparationDenial,
    >
    where
        Schema: ApplicationSchema,
        Operation: ApplicationOperationMarkerIdentity<Schema> + 'static,
        Spec: ApplicationWorkflowSpec<Schema = Schema>,
    {
        let phase_request = match phase.execution_request_for(selected.application().product_runtime()) {
            Ok(request) => request,
            Err(_) => return Err(WorkflowProposalPreparationDenial::Attempt(crate::domain_computation::primary_graph::WorthQueryApplicationAttemptDenial::new(crate::domain_computation::vocabulary::primary_graph::kind::WorthQueryApplicationAttemptDenialKind::ForeignPhase, "workflow-phase"))),
        };
        let execution = &phase_request;

        selected.workflow_selection_basis().prepare_workflow_proposal(
            execution,
            installed,
            instance,
            admission,
            *idempotency.intent_identity(),
            idempotency.source_identity(),
            captured,
        )
    }

    pub fn compare_and_commit<Schema, Operation, Input, Scope>(
        phase: &WorthQueryAdvancementPhase<'_>,

        runtime: &dyn crate::domain_computation::primary_graph::application_attempt::host::ApplicationAttemptHost<Schema>,
        prepared: PreparedWorkflowProposal<Schema, Operation, Input, Scope>,
        idempotency: WorthQueryApplicationIdempotencyBinding,
    ) -> WorkflowProposalOutcome
    where
        Schema: ApplicationSchema,
        Operation: 'static,
        Input: Clone + Send + Sync + 'static,
    {
        let phase_request = match phase.execution_request_for(runtime.product_runtime()) {
            Ok(request) => request,
            Err(cause) => return WorkflowProposalOutcome::Application(
                crate::domain_computation::request_execution::WorthQueryAdvancementDenial::from(
                    cause,
                )
                .into_commit_outcome()
                .landed()
                .expect_err("a foreign phase cannot commit"),
            ),
        };
        let execution = &phase_request;

        runtime.compare_and_commit_workflow_proposal(execution, phase, prepared, idempotency)
    }
}
