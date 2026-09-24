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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorkflowProposalBindingDenial {
    ForeignSchema,
    SelectedProgramUnavailable,
    ProgramRevisionChanged,
    SelectedOccurrenceChanged,
}

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
        Program,
    >(
        &self,
        installed: &WorthQueryInstalledApplicationWorkflowSpec<Schema, Spec, Program>,
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
            .project_admitted_operation(&admission, |_, _| {})
            .map_err(WorkflowProposalPreparationDenial::Projection)?
            .into_parts();
        let read_set = self
            .application()
            .begin_projected_application_read_attempt(admission, projection)
            .map_err(WorkflowProposalPreparationDenial::Attempt)?
            .complete_projected_dependencies()
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
            .materialize_workflow_proposal(installed, instance, input_identity, source_identity)
            .map_err(WorkflowProposalPreparationDenial::Attempt)
    }
}

#[doc(hidden)]
pub struct WorthQueryWorkflowProposalAdapter;

impl WorthQueryWorkflowProposalAdapter {
    pub fn resolve_replay<Schema, Operation, Input, Scope>(
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
        runtime.resolve_workflow_proposal_replay(
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
    pub fn prepare<Schema, Operation, Input, Scope, Spec, Program>(
        selected: &WorthQuerySelectedProductOperation<'_, Schema>,
        installed: &WorthQueryInstalledApplicationWorkflowSpec<Schema, Spec, Program>,
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
        selected.prepare_workflow_proposal(
            installed,
            instance,
            admission,
            *idempotency.intent_identity(),
            idempotency.source_identity(),
        )
    }

    pub fn compare_and_commit<Schema, Operation, Input, Scope>(
        runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        prepared: PreparedWorkflowProposal<Schema, Operation, Input, Scope>,
        idempotency: WorthQueryApplicationIdempotencyBinding,
    ) -> WorkflowProposalOutcome
    where
        Schema: ApplicationSchema,
        Operation: 'static,
        Input: Clone + Send + Sync + 'static,
    {
        runtime.compare_and_commit_workflow_proposal(prepared, idempotency)
    }
}
