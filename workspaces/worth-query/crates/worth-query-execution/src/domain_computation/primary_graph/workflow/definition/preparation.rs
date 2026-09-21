use worth_query_declaration::facade::application_program::ApplicationWorkflowSpec;
use worth_query_declaration::facade::{
    application_capability::ApplicationCapabilityMarkerIdentity,
    application_schema::ApplicationOperationMarkerIdentity,
};
use worth_query_installation::facade::{
    ApplicationSchema, WorthQueryInstalledWorkflowDefinitionContract,
};

use crate::basis::{WorthQueryProductBranch, WorthQueryProductBranchReadIdentity};
use crate::domain_computation::primary_graph::WorthQuerySelectedProductOperation;
use crate::domain_computation::primary_graph::{
    PreparedWorkflowDefinitionPublication, WorthQueryAdmittedApplicationOperation,
    WorthQueryApplicationAttemptDenial, WorthQueryOperationProjectionDenial,
};

/// Owner-bound definition contract for one exact selected branch occurrence.
///
/// This is deliberately not the public prepared publication. It only carries
/// the selected-program and branch affinity needed by the application-attempt
/// owner to prepare the real governed mutation.
pub(in crate::domain_computation::primary_graph) struct BoundWorkflowDefinitionContract<
    Schema,
    Spec,
    Program,
> where
    Schema: ApplicationSchema,
    Spec: ApplicationWorkflowSpec<Schema = Schema>,
{
    pub(in crate::domain_computation::primary_graph) contract:
        WorthQueryInstalledWorkflowDefinitionContract<Schema, Spec, Program>,
    branch: WorthQueryProductBranch,
    occurrence: WorthQueryProductBranchReadIdentity,
}

impl<Schema, Spec, Program> BoundWorkflowDefinitionContract<Schema, Spec, Program>
where
    Schema: ApplicationSchema,
    Spec: ApplicationWorkflowSpec<Schema = Schema>,
{
    pub(in crate::domain_computation::primary_graph) fn belongs_to_branch(
        &self,
        branch: WorthQueryProductBranch,
    ) -> bool {
        self.branch == branch
    }

    pub(in crate::domain_computation::primary_graph) fn belongs_to_occurrence(
        &self,
        occurrence: &crate::basis::WorthQueryProductBranchLease,
    ) -> bool {
        self.occurrence
            == WorthQueryProductBranchReadIdentity::from_observation(occurrence.observation())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorkflowDefinitionBindingDenial {
    ForeignSchema,
    SelectedProgramUnavailable,
    ProgramRevisionChanged,
}

#[derive(Debug)]
pub enum WorkflowDefinitionPreparationDenial {
    ProductSelection(crate::basis::WorthQueryProductBranchAdmissionDenial),
    Binding(WorkflowDefinitionBindingDenial),
    Projection(WorthQueryOperationProjectionDenial),
    Attempt(WorthQueryApplicationAttemptDenial),
}

impl std::fmt::Display for WorkflowDefinitionPreparationDenial {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ProductSelection(denial) => {
                write!(formatter, "workflow product selection: {denial:?}")
            }
            Self::Binding(denial) => write!(formatter, "workflow definition binding: {denial:?}"),
            Self::Projection(denial) => denial.fmt(formatter),
            Self::Attempt(denial) => denial.fmt(formatter),
        }
    }
}

impl std::error::Error for WorkflowDefinitionPreparationDenial {}

impl<Schema> WorthQuerySelectedProductOperation<'_, Schema>
where
    Schema: ApplicationSchema,
{
    pub(in crate::domain_computation::primary_graph) fn bind_workflow_definition_contract<
        Spec,
        Program,
    >(
        &self,
        contract: WorthQueryInstalledWorkflowDefinitionContract<Schema, Spec, Program>,
    ) -> Result<
        BoundWorkflowDefinitionContract<Schema, Spec, Program>,
        WorkflowDefinitionBindingDenial,
    >
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

    pub(in crate::domain_computation::primary_graph) fn prepare_workflow_publication<
        Capability,
        Operation,
        Input,
        Scope,
        Spec,
        Program,
    >(
        &self,
        contract: WorthQueryInstalledWorkflowDefinitionContract<Schema, Spec, Program>,
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
        let bound = self
            .bind_workflow_definition_contract(contract)
            .map_err(WorkflowDefinitionPreparationDenial::Binding)?;
        let (_, projection, _) = self
            .application()
            .mutation_projection
            .project_admitted_operation(&admission, |_, _| {})
            .map_err(WorkflowDefinitionPreparationDenial::Projection)?
            .into_parts();
        let read_set = self
            .application()
            .begin_projected_application_read_attempt(admission, projection)
            .map_err(WorkflowDefinitionPreparationDenial::Attempt)?
            .complete_projected_dependencies()
            .map_err(WorkflowDefinitionPreparationDenial::Attempt)?;
        read_set
            .materialize_workflow_definition_publication::<Capability, Spec, Program>(
                bound,
                expected_predecessor,
            )
            .map_err(WorkflowDefinitionPreparationDenial::Attempt)
    }
}
