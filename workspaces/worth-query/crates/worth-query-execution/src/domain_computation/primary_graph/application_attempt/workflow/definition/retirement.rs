//! Definition retirement preparation for one selected branch occurrence.

use crate::domain_computation::primary_graph::application_attempt::ReadSetScopedOps;
use worth_query_declaration::facade::{
    application_capability::ApplicationCapabilityMarkerIdentity,
    application_program::ApplicationWorkflowSpec,
    application_schema::ApplicationOperationMarkerIdentity,
};
use worth_query_installation::facade::{
    ApplicationSchema, WorthQueryInstalledApplicationWorkflowSpec,
};

use super::{WorkflowDefinitionBindingDenial, WorkflowDefinitionPreparationDenial};
use crate::basis::WorthQueryProductBranchReadIdentity;
use crate::domain_computation::primary_graph::PreparedWorkflowDefinitionRetirement;
use crate::domain_computation::primary_graph::WorthQueryAdmittedApplicationOperation;
use crate::domain_computation::primary_graph::application_attempt::WorkflowSelectionBasis;
use crate::domain_computation::primary_graph::vocabulary::application_attempt::publication::PublishedWorkflowDefinitionRef;

impl<Schema> WorkflowSelectionBasis<'_, Schema>
where
    Schema: ApplicationSchema,
{
    pub(in crate::domain_computation::primary_graph) fn prepare_workflow_definition_retirement<Capability, Operation, Input, Scope, Spec>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        installed: &WorthQueryInstalledApplicationWorkflowSpec<Schema, Spec>,
        definition: PublishedWorkflowDefinitionRef,
        admission: WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>,
    ) -> Result<
        PreparedWorkflowDefinitionRetirement<Schema, Operation, Input, Scope>,
        WorkflowDefinitionPreparationDenial,
    >
    where
        Capability: ApplicationCapabilityMarkerIdentity<Schema = Schema> + 'static,
        Operation: ApplicationOperationMarkerIdentity<Schema> + 'static,
        Spec: ApplicationWorkflowSpec<Schema = Schema>,
    {
        let application = self.application();
        admission
            .validate_installed_entry(
                application.authority_identity(),
                &application.installed_schema().binding_identity(),
                application.installed_operations(),
            )
            .and_then(|()| admission.validate_current_authority())
            .map_err(|denial| WorkflowDefinitionPreparationDenial::Projection(denial.into()))?;

        if installed.schema_binding() != &self.application().installed_schema().binding_identity() {
            return Err(WorkflowDefinitionPreparationDenial::Binding(
                WorkflowDefinitionBindingDenial::ForeignSchema,
            ));
        }
        let selected = self.inspect_selected_program().map_err(|_| {
            WorkflowDefinitionPreparationDenial::Binding(
                WorkflowDefinitionBindingDenial::SelectedProgramUnavailable,
            )
        })?;
        if selected.revision() != installed.program_revision() {
            return Err(WorkflowDefinitionPreparationDenial::Binding(
                WorkflowDefinitionBindingDenial::ProgramRevisionChanged,
            ));
        }
        let selected_occurrence =
            WorthQueryProductBranchReadIdentity::from_observation(self.product().observation());
        let (_, projection, _) = self
            .application()
            .mutation_projection()
            .project_admitted_operation_with_request(
                execution,
                &admission,
                |_, _| {},
                worth_execution::ExecutionAllocationPolicy::SystemAllocation,
            )
            .map_err(WorkflowDefinitionPreparationDenial::Projection)?
            .into_parts();
        let read_set = self
            .application()
            .begin_projected_application_read_attempt_with_request(
                execution,
                admission,
                projection,
                worth_execution::ExecutionAllocationPolicy::SystemAllocation,
            )
            .map_err(WorkflowDefinitionPreparationDenial::Attempt)?
            .complete_projected_dependencies_with_request(
                execution,
                crate::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
            )
            .map_err(WorkflowDefinitionPreparationDenial::Attempt)?;
        read_set
            .materialize_workflow_definition_retirement::<Capability, Spec>(
                execution,
                installed,
                definition,
                &selected_occurrence,
            )
            .map_err(WorkflowDefinitionPreparationDenial::Attempt)
    }
}
