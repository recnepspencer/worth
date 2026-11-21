//! Selected-product workflow approval preparation using L6 workflow vocabulary.
use super::*;
use worth_query_execution_attempt::facade::ports::CompleteReadSetWorkflowInternalPort as _;
use worth_query_execution_attempt::facade::ports::ProjectedReadSetInternalPort as _;
use worth_query_execution_runtime::facade::ports::ProductBranchLeaseInternalPort as _;

impl<Schema: ApplicationSchema> WorthQuerySelectedProductOperation<'_, Schema> {
    #[allow(clippy::too_many_arguments)]
    pub(in crate::domain_computation::primary_graph) fn prepare_workflow_approval<
        Capability,
        Operation,
        Input,
        Scope,
        Spec,
    >(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        workflow: WorthQueryWorkflowVocabulary<'_, Schema, Spec>,
        instance: PublishedWorkflowInstanceRef,
        required: &RequiredWorkflowApproval,
        proposal: &PublishedWorkflowProposalRef,
        decision: WorkflowApprovalDecision,
        admission: WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>,
    ) -> Result<
        PreparedWorkflowAdvance<Schema, Operation, Input, Scope>,
        WorkflowTransitionPreparationDenial,
    >
    where
        Capability: ApplicationCapabilityMarkerIdentity<Schema = Schema> + 'static,
        Operation: ApplicationOperationMarkerIdentity<Schema> + 'static,
        Spec: ApplicationWorkflowSpec<Schema = Schema>,
    {
        let installed = workflow.workflow_spec_for(self);
        if installed.schema_binding() != &self.application().installed_schema().binding_identity() {
            return Err(WorkflowTransitionPreparationDenial::Binding(
                WorkflowTransitionBindingDenial::ForeignSchema,
            ));
        }
        let selected = self.inspect_selected_program().map_err(|_| {
            WorkflowTransitionPreparationDenial::Binding(
                WorkflowTransitionBindingDenial::SelectedProgramUnavailable,
            )
        })?;
        if selected.revision() != installed.program_revision() {
            return Err(WorkflowTransitionPreparationDenial::Binding(
                WorkflowTransitionBindingDenial::ProgramRevisionChanged,
            ));
        }
        let selected_occurrence =
            crate::basis::WorthQueryProductBranchReadIdentity::from_observation(
                self.product().observation(),
            );
        let (_, projection, _) = self
            .application()
            .inner
            .mutation_projection
            .project_admitted_operation_with_request(
                execution,
                &admission,
                |_, _| {},
                worth_execution::ExecutionAllocationPolicy::SystemAllocation,
            )
            .map_err(WorkflowTransitionPreparationDenial::Projection)?
            .into_parts();
        let read_set = self
            .application()
            .begin_projected_application_read_attempt_with_request(
                execution,
                admission,
                projection,
                worth_execution::ExecutionAllocationPolicy::SystemAllocation,
            )
            .map_err(WorkflowTransitionPreparationDenial::Attempt)?
            .complete_projected_dependencies_with_request(
                execution,
                crate::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
            )
            .map_err(WorkflowTransitionPreparationDenial::Attempt)?;
        use worth_query_execution_attempt::facade::ports::CompleteReadSetInternalPort as _;
        if selected_occurrence
            != crate::basis::WorthQueryProductBranchReadIdentity::from_observation(
                read_set.lease().product().observation(),
            )
        {
            return Err(WorkflowTransitionPreparationDenial::Binding(
                WorkflowTransitionBindingDenial::SelectedOccurrenceChanged,
            ));
        }
        read_set
            .materialize_workflow_approval::<Capability, Spec>(
                execution,
                &workflow,
                installed,
                instance,
                required,
                proposal,
                decision,
                &self.application().inner.runtime.authorization_clock,
            )
            .map_err(WorkflowTransitionPreparationDenial::Attempt)
    }
}
