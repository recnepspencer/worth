use crate::domain_computation::primary_graph::application_attempt::ReadSetScopedOps;
use worth_query_declaration::facade::{
    application_program::ApplicationWorkflowSpec,
    application_schema::ApplicationOperationMarkerIdentity,
};
use worth_query_installation::facade::{
    ApplicationSchema, WorthQueryInstalledApplicationWorkflowSpec,
};

use crate::domain_computation::primary_graph::WorthQueryAdmittedApplicationOperation;
use crate::domain_computation::primary_graph::WorthQueryOperationProjectionDenial;
use crate::domain_computation::primary_graph::application_attempt::WorkflowSelectionBasis;
use crate::domain_computation::primary_graph::vocabulary::application_attempt::attempt_denial::WorthQueryApplicationAttemptDenial;
use crate::domain_computation::primary_graph::vocabulary::application_attempt::workflow_instance_program_publication::PublishedWorkflowInstanceRef;
use super::PreparedWorkflowProposal;

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

impl<Schema> WorkflowSelectionBasis<'_, Schema>
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
        captured: crate::domain_computation::primary_graph::application_attempt::workflow::captured_input::WorkflowCapturedInputs,
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
            .mutation_projection()
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
                captured,
                self.application().authorization_clock(),
            )
            .map_err(WorkflowProposalPreparationDenial::Attempt)
    }
}
