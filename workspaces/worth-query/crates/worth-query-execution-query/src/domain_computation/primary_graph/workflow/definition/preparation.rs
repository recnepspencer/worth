use worth_query_declaration::facade::application_program::ApplicationWorkflowSpec;
use worth_query_declaration::facade::{
    application_capability::ApplicationCapabilityMarkerIdentity,
    application_schema::ApplicationOperationMarkerIdentity,
};
use worth_query_execution_attempt::facade::ports::AdmittedOperationExecutionPort as _;
use worth_query_execution_attempt::facade::ports::CompleteReadSetWorkflowInternalPort as _;
use worth_query_execution_attempt::facade::ports::ProjectedReadSetInternalPort as _;
use worth_query_execution_runtime::facade::ports::ProductBranchLeaseInternalPort as _;
use worth_query_installation::facade::{
    ApplicationSchema, WorthQueryInstalledWorkflowDefinitionContract,
};

use crate::basis::WorthQueryProductBranchReadIdentity;
use crate::domain_computation::primary_graph::WorthQuerySelectedProductOperation;
use crate::domain_computation::primary_graph::{
    PreparedWorkflowDefinitionPublication, WorthQueryAdmittedApplicationOperation,
};

pub use worth_query_execution_attempt::facade::domain_computation::primary_graph::workflow::definition::preparation::BoundWorkflowDefinitionContract;

/// Why a workflow definition request could not bind to the branch's program: the installed
/// spec belongs to another schema, the adopted program could not be inspected, or the
/// branch runs another program revision.
pub use worth_query_execution_attempt::facade::domain_computation::primary_graph::workflow::definition::preparation::{WorkflowDefinitionBindingDenial, WorkflowDefinitionPreparationDenial};

/// Why a workflow definition publication or retirement did not prepare.

impl<Schema> WorthQuerySelectedProductOperation<'_, Schema>
where
    Schema: ApplicationSchema,
{
    pub fn bind_workflow_definition_contract<Spec>(
        &self,
        contract: WorthQueryInstalledWorkflowDefinitionContract<Schema, Spec>,
    ) -> Result<BoundWorkflowDefinitionContract<Schema, Spec>, WorkflowDefinitionBindingDenial>
    where
        Spec: ApplicationWorkflowSpec<Schema = Schema>,
    {
        if contract.schema_binding() != &self.application().installed_schema().binding_identity() {
            return Err(WorkflowDefinitionBindingDenial::ForeignSchema);
        }
        let selected = self
            .inspect_selected_program()
            .map_err(|_| WorkflowDefinitionBindingDenial::SelectedProgramUnavailable)?;
        if selected.revision() != contract.program_revision() {
            return Err(WorkflowDefinitionBindingDenial::ProgramRevisionChanged);
        }
        Ok(BoundWorkflowDefinitionContract {
            contract,
            branch: self.product().product_branch(),
            occurrence: WorthQueryProductBranchReadIdentity::from_observation(
                self.product().observation(),
            ),
        })
    }

    pub fn prepare_workflow_publication<Capability, Operation, Input, Scope, Spec>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        contract: WorthQueryInstalledWorkflowDefinitionContract<Schema, Spec>,
        expected_predecessor: crate::domain_computation::primary_graph::WorkflowDefinitionExpectedPredecessor,
        admission: WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>,
    ) -> Result<
        PreparedWorkflowDefinitionPublication<Schema, Operation, Input, Scope>,
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
                application.runtime.authority_identity(),
                &application.installed_schema.binding_identity(),
                &application.runtime.installed_operations,
            )
            .and_then(|()| admission.validate_current_authority())
            .map_err(|denial| WorkflowDefinitionPreparationDenial::Projection(denial.into()))?;

        let bound = self
            .bind_workflow_definition_contract(contract)
            .map_err(WorkflowDefinitionPreparationDenial::Binding)?;

        let (_, projection, _) = self
            .application()
            .mutation_projection
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
            .materialize_workflow_definition_publication::<Capability, Spec>(
                execution,
                bound,
                expected_predecessor,
            )
            .map_err(WorkflowDefinitionPreparationDenial::Attempt)
    }
}

mod court_doors_for_test;
