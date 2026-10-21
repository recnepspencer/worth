//! Exact selected-program and product observations shared by controls and local business actions.
use super::*;
use crate::domain_computation::primary_graph::{
    WorthQueryCompleteApplicationReadSet, WorthQueryProjectedApplicationMutation,
};

impl<S: ApplicationSchema> WorthQuerySelectedProductOperation<'_, S> {
    pub(super) fn prepare_transition_observation<O, I, Scope, Spec>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        installed: &WorthQueryInstalledApplicationWorkflowSpec<S, Spec>,
        admission: WorthQueryAdmittedApplicationOperation<S, O, I, Scope>,
    ) -> Result<
        WorthQueryCompleteApplicationReadSet<
            S,
            O,
            I,
            Scope,
            WorthQueryProjectedApplicationMutation,
        >,
        WorkflowTransitionPreparationDenial,
    >
    where
        O: ApplicationOperationMarkerIdentity<S> + 'static,
        Spec: ApplicationWorkflowSpec<Schema = S>,
    {
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
        if selected_occurrence
            != crate::basis::WorthQueryProductBranchReadIdentity::from_observation(
                read_set.lease.product().observation(),
            )
        {
            return Err(WorkflowTransitionPreparationDenial::Binding(
                WorkflowTransitionBindingDenial::SelectedOccurrenceChanged,
            ));
        }
        Ok(read_set)
    }
    pub(super) fn prepare_local_workflow_transition<O, I, Scope, Spec>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        installed: &WorthQueryInstalledApplicationWorkflowSpec<S, Spec>,
        instance: PublishedWorkflowInstanceRef,
        admission: WorthQueryAdmittedApplicationOperation<S, O, I, Scope>,
        request_kind: WorkflowTransitionRequestKind,
    ) -> Result<PreparedWorkflowAdvance<S, O, I, Scope>, WorkflowTransitionPreparationDenial>
    where
        O: ApplicationOperationMarkerIdentity<S> + 'static,
        Spec: ApplicationWorkflowSpec<Schema = S>,
    {
        let read_set = self.prepare_transition_observation(execution, installed, admission)?;
        read_set
            .materialize_admitted_workflow_transition::<Spec>(
                execution,
                installed,
                instance,
                request_kind,
                &self.application().authorization_clock,
            )
            .map_err(WorkflowTransitionPreparationDenial::Attempt)
    }
}
