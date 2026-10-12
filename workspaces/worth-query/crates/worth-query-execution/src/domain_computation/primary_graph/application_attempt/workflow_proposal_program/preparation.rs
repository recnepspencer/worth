use crate::domain_computation::primary_graph::WorthQueryAdvancementPhase;
use worth_query_declaration::facade::{
    application_program::ApplicationWorkflowSpec,
    application_schema::{ApplicationOperationMarkerIdentity, ApplicationStructuredValueBinding},
};
use worth_query_installation::facade::{
    ApplicationSchema, WorthQueryInstalledApplicationWorkflowSpec,
};

use super::{PreparedWorkflowProposal, WorkflowProposalOutcome};
use crate::domain_computation::primary_graph::{
    PublishedWorkflowInstanceRef, WorthQueryAdmittedApplicationOperation,
    WorthQueryApplicationAttemptDenial, WorthQueryApplicationIdempotencyBinding,
    WorthQueryApplicationIdempotencyResolutionDenial, WorthQueryOperationProjectionDenial,
    WorthQueryPrimaryGraphApplicationRuntime, WorthQuerySelectedProductOperation,
};

/// Why a workflow proposal could not bind to the branch's program: the installed spec
/// belongs to another schema, the adopted program could not be inspected, the branch runs
/// another program revision, or the branch advanced between selection and read-set
/// assembly.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorkflowProposalBindingDenial {
    ForeignSchema,
    SelectedProgramUnavailable,
    ProgramRevisionChanged,
    SelectedOccurrenceChanged,
}

/// Why a workflow proposal did not prepare.
#[derive(Debug)]
pub enum WorkflowProposalPreparationDenial {
    Binding(WorkflowProposalBindingDenial),
    Projection(WorthQueryOperationProjectionDenial),
    Attempt(WorthQueryApplicationAttemptDenial),
}

impl std::fmt::Display for WorkflowProposalPreparationDenial {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "workflow proposal preparation denied: {self:?}")
    }
}

impl std::error::Error for WorkflowProposalPreparationDenial {}

impl<Schema> WorthQuerySelectedProductOperation<'_, Schema>
where
    Schema: ApplicationSchema,
{
    #[allow(clippy::too_many_arguments)]
    pub(in crate::domain_computation::primary_graph) fn prepare_workflow_proposal<
        Operation,
        Input,
        Scope,
        Spec,
    >(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        installed: &WorthQueryInstalledApplicationWorkflowSpec<Schema, Spec>,
        instance: PublishedWorkflowInstanceRef,
        admission: WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>,
        input_identity: [u8; 32],
        source_identity: Option<[u8; 32]>,
    ) -> Result<
        PreparedWorkflowProposal<Schema, Operation, Input, Scope>,
        WorkflowProposalPreparationDenial,
    >
    where
        Operation: ApplicationOperationMarkerIdentity<Schema> + 'static,
        Spec: ApplicationWorkflowSpec<Schema = Schema>,
    {
        if installed.schema_binding() != &self.application().installed_schema().binding_identity() {
            return Err(WorkflowProposalPreparationDenial::Binding(
                WorkflowProposalBindingDenial::ForeignSchema,
            ));
        }
        let selected = self.inspect_selected_program().map_err(|_| {
            WorkflowProposalPreparationDenial::Binding(
                WorkflowProposalBindingDenial::SelectedProgramUnavailable,
            )
        })?;
        if selected.revision() != installed.program_revision() {
            return Err(WorkflowProposalPreparationDenial::Binding(
                WorkflowProposalBindingDenial::ProgramRevisionChanged,
            ));
        }
        let selected_occurrence =
            crate::basis::WorthQueryProductBranchReadIdentity::from_observation(
                self.product().observation(),
            );
        let (_, projection, _) = self
            .application()
            .mutation_projection
            .project_admitted_operation_with_request(
                execution,
                &admission,
                |_, _| {},
                worth_execution::ExecutionAllocationPolicy::SystemAllocation,
            )
            .map_err(WorkflowProposalPreparationDenial::Projection)?
            .into_parts();
        let read_set = self
            .application()
            .begin_projected_application_read_attempt_with_request(
                execution,
                admission,
                projection,
                worth_execution::ExecutionAllocationPolicy::SystemAllocation,
            )
            .map_err(WorkflowProposalPreparationDenial::Attempt)?
            .complete_projected_dependencies_with_request(
                execution,
                crate::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
            )
            .map_err(WorkflowProposalPreparationDenial::Attempt)?;
        if selected_occurrence
            != crate::basis::WorthQueryProductBranchReadIdentity::from_observation(
                read_set.lease.product().observation(),
            )
        {
            return Err(WorkflowProposalPreparationDenial::Binding(
                WorkflowProposalBindingDenial::SelectedOccurrenceChanged,
            ));
        }
        read_set
            .materialize_workflow_proposal(
                execution,
                installed,
                instance,
                input_identity,
                source_identity,
                &self.application().authorization_clock,
            )
            .map_err(WorkflowProposalPreparationDenial::Attempt)
    }
}

#[doc(hidden)]
pub struct WorthQueryWorkflowProposalAdapter;

impl WorthQueryWorkflowProposalAdapter {
    pub fn resolve_replay<Schema, Operation, Input, Scope>(
        phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
        runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        admission: &WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>,
        idempotency: WorthQueryApplicationIdempotencyBinding,
        instance: PublishedWorkflowInstanceRef,
    ) -> Result<Option<WorkflowProposalOutcome>, WorthQueryApplicationIdempotencyResolutionDenial>
    where
        Schema: ApplicationSchema,
        Operation: ApplicationOperationMarkerIdentity<Schema> + 'static,
        Input: Clone + Send + Sync + 'static,
    {
        let phase_request = match phase.execution_request_for(&runtime.product_runtime) {
            Ok(request) => request,
            Err(_) => return Err(crate::domain_computation::primary_graph::WorthQueryApplicationIdempotencyResolutionDenial::from_authorization(crate::domain_computation::primary_graph::WorthQueryOperationAuthorizationDenial::new(crate::domain_computation::primary_graph::WorthQueryOperationAuthorizationDenialKind::ForeignPhase, "workflow-phase"))),
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
        phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
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
        let phase_request = match phase.execution_request_for(&selected.application().product_runtime) {
            Ok(request) => request,
            Err(_) => return Err(WorkflowProposalPreparationDenial::Attempt(crate::domain_computation::primary_graph::WorthQueryApplicationAttemptDenial::new(crate::domain_computation::primary_graph::WorthQueryApplicationAttemptDenialKind::ForeignPhase, "workflow-phase"))),
        };
        let execution = &phase_request;

        selected.prepare_workflow_proposal(
            execution,
            installed,
            instance,
            admission,
            *idempotency.intent_identity(),
            idempotency.source_identity(),
        )
    }

    pub fn compare_and_commit<Schema, Operation, Input, Scope>(
        phase: &WorthQueryAdvancementPhase<'_>,

        runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        prepared: PreparedWorkflowProposal<Schema, Operation, Input, Scope>,
        idempotency: WorthQueryApplicationIdempotencyBinding,
    ) -> WorkflowProposalOutcome
    where
        Schema: ApplicationSchema,
        Operation: 'static,
        Input: Clone + Send + Sync + 'static,
    {
        let phase_request =
            match phase.execution_request_for(&runtime.product_runtime) {
                Ok(request) => request,
                Err(cause) => return WorkflowProposalOutcome::Application(
                    crate::domain_computation::primary_graph::WorthQueryAdvancementDenial::from(
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
