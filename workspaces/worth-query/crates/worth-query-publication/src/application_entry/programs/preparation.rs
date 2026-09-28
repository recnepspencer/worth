use worth_query_installation::facade::{ApplicationSchema, WorthQueryProgramAdoptionRequirements};

use super::WorthQueryApplicationProgramsRequest;

pub use worth_query_execution::facade::primary_graph::{
    WorthQueryPreparedBranchAdoption, WorthQueryPreparedProgramMigration,
    WorthQueryWorkflowAdoptionInventory, WorthQueryWorkflowCompatibility,
    WorthQueryWorkflowDefinitionDisposition, WorthQueryWorkflowDefinitionOccurrence,
    WorthQueryWorkflowDispositionDenial, WorthQueryWorkflowDispositions,
    WorthQueryWorkflowIncompatibility, WorthQueryWorkflowInstanceCustody,
    WorthQueryWorkflowInstanceDisposition, WorthQueryWorkflowInstanceOccurrence,
};

/// Why a program adoption did not prepare.
#[derive(Debug)]
pub enum WorthQueryApplicationProgramAdoptionPreparationDenial {
    ProductSelection(
        worth_query_execution::facade::primary_graph::WorthQueryProductBranchAdmissionDenial,
    ),
    Adoption(
        worth_query_execution::facade::primary_graph::WorthQueryBranchAdoptionPreparationDenial,
    ),
}

/// Adoption toward the target its owner-computed requirements name, with no
/// effects performed.
/// Only `prepare` can mint the move-only value whose `publish` method exists.
///
/// A branch holding workflow facts needs one explicit decision per current
/// definition and live instance: read [`Self::workflow_inventory`], decide it
/// into [`WorthQueryWorkflowDispositions`], and pass them to [`Self::workflow`].
///
/// ```compile_fail,E0451
/// use worth_query_execution::facade::primary_graph::WorthQueryPreparedBranchAdoption;
///
/// fn callers_cannot_forge_prepared_adoption() {
///     let _forged = WorthQueryPreparedBranchAdoption {
///         source: todo!(),
///         target: todo!(),
///         requirements: todo!(),
///         selected_entity_count: 0,
///         selection_work_units: 0,
///         publication: todo!(),
///     };
/// }
/// ```
pub struct WorthQueryApplicationProgramAdoptionRequest<
    'application,
    'principal,
    'scope,
    'requirements,
    Schema,
> {
    pub(super) programs:
        WorthQueryApplicationProgramsRequest<'application, 'principal, 'scope, Schema>,
    pub(super) requirements: &'requirements WorthQueryProgramAdoptionRequirements,
    pub(super) migration: Option<WorthQueryPreparedProgramMigration>,
    pub(super) workflow: Option<WorthQueryWorkflowDispositions>,
}

impl<'application, 'principal, 'scope, 'requirements, Schema>
    WorthQueryApplicationProgramAdoptionRequest<
        'application,
        'principal,
        'scope,
        'requirements,
        Schema,
    >
where
    Schema: ApplicationSchema,
{
    pub fn migration(mut self, migration: WorthQueryPreparedProgramMigration) -> Self {
        self.migration = Some(migration);
        self
    }

    /// Every current workflow definition and live instance this branch
    /// incarnation holds, each with its legal dispositions under the target.
    pub fn workflow_inventory(
        &self,
        maximum_work_units: usize,
    ) -> Result<
        WorthQueryWorkflowAdoptionInventory,
        WorthQueryApplicationProgramAdoptionPreparationDenial,
    > {
        self.programs
            .application
            .on_branch(self.programs.branch)
            .select()
            .map_err(WorthQueryApplicationProgramAdoptionPreparationDenial::ProductSelection)?
            .workflow_adoption_inventory(
                self.requirements.target(),
                self.requirements,
                maximum_work_units,
            )
            .map_err(WorthQueryApplicationProgramAdoptionPreparationDenial::Adoption)
    }

    /// Decisions for the inventory they were built from. Preparation refuses
    /// them if owner truth moved since.
    pub fn workflow(mut self, dispositions: WorthQueryWorkflowDispositions) -> Self {
        self.workflow = Some(dispositions);
        self
    }

    pub fn prepare(
        self,
        maximum_selection_work: usize,
    ) -> Result<
        WorthQueryPreparedBranchAdoption,
        WorthQueryApplicationProgramAdoptionPreparationDenial,
    > {
        self.programs
            .application
            .on_branch(self.programs.branch)
            .select()
            .map_err(WorthQueryApplicationProgramAdoptionPreparationDenial::ProductSelection)?
            .prepare_branch_adoption_with_choices(
                self.requirements.target(),
                self.requirements,
                self.migration,
                self.workflow,
                maximum_selection_work,
                self.programs.scope,
            )
            .map_err(WorthQueryApplicationProgramAdoptionPreparationDenial::Adoption)
    }
}
