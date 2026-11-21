use worth_query_declaration::facade::application_program::ApplicationWorkflowSpec;
use worth_query_declaration::facade::{
    application_capability::ApplicationCapabilityMarkerIdentity,
    application_schema::ApplicationOperationMarkerIdentity,
};
use worth_query_installation::facade::{
    ApplicationSchema, WorthQueryInstalledWorkflowDefinitionContract,
};

use crate::basis::{WorthQueryProductBranch, WorthQueryProductBranchReadIdentity};
use crate::domain_computation::primary_graph::{
    WorthQueryAdmittedApplicationOperation,
    WorthQueryApplicationAttemptDenial, WorthQueryOperationProjectionDenial,
};

/// Owner-bound definition contract for one exact selected branch occurrence.
///
/// This is deliberately not the public prepared publication. It only carries
/// the selected-program and branch affinity needed by the application-attempt
/// owner to prepare the real governed mutation.
pub struct BoundWorkflowDefinitionContract<
    Schema,
    Spec,
> where
    Schema: ApplicationSchema,
    Spec: ApplicationWorkflowSpec<Schema = Schema>,
{
    pub contract:
        WorthQueryInstalledWorkflowDefinitionContract<Schema, Spec>,
    pub branch: WorthQueryProductBranch,
    pub occurrence: WorthQueryProductBranchReadIdentity,
}

impl<Schema, Spec> BoundWorkflowDefinitionContract<Schema, Spec>
where
    Schema: ApplicationSchema,
    Spec: ApplicationWorkflowSpec<Schema = Schema>,
{
    pub fn belongs_to_branch(
        &self,
        branch: WorthQueryProductBranch,
    ) -> bool {
        self.branch == branch
    }

    pub fn belongs_to_occurrence(
        &self,
        occurrence: &crate::basis::WorthQueryProductBranchLease,
    ) -> bool {
        self.occurrence
            == WorthQueryProductBranchReadIdentity::from_observation(occurrence.observation())
    }
}

/// Why a workflow definition request could not bind to the branch's program: the installed
/// spec belongs to another schema, the adopted program could not be inspected, or the
/// branch runs another program revision.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorkflowDefinitionBindingDenial {
    ForeignSchema,
    SelectedProgramUnavailable,
    ProgramRevisionChanged,
}

/// Why a workflow definition publication or retirement did not prepare.
#[derive(Debug)]
pub enum WorkflowDefinitionPreparationDenial {
    DurableStateAdmission(crate::domain_computation::execution_runtime::product_world::WorthQueryDurableStateAdmissionDenial),
    ProductSelection(crate::basis::WorthQueryProductBranchAdmissionDenial),
    Binding(WorkflowDefinitionBindingDenial),
    Projection(WorthQueryOperationProjectionDenial),
    Attempt(WorthQueryApplicationAttemptDenial),
}

impl std::fmt::Display for WorkflowDefinitionPreparationDenial {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DurableStateAdmission(denial) => denial.fmt(formatter),
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

