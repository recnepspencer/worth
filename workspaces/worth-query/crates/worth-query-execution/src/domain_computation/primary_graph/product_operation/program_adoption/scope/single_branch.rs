use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;
use worth_query_declaration::facade::application_program::ApplicationProgramRevision;
use worth_query_installation::facade::{ApplicationSchema, WorthQueryProgramAdoptionRequirements};

use super::super::{
    preparation, WorthQueryBranchAdoptionPreparationDenial, WorthQueryPreparedBranchAdoption,
    WorthQueryPreparedProgramMigration,
};
use crate::domain_computation::primary_graph::product_operation::WorthQuerySelectedProductOperation;
use crate::domain_computation::primary_graph::workflow::{
    WorthQueryWorkflowAdoptionInventory, WorthQueryWorkflowDispositions,
};

impl<Schema: ApplicationSchema> WorthQuerySelectedProductOperation<'_, Schema> {
    /// Describes the owner-computed requirements for moving this exact branch
    /// occurrence to `target`. The returned value is evidence, not a permit;
    /// preparation recomputes and compares it.
    pub fn branch_adoption_requirements(
        &self,
        target: &ApplicationProgramRevision,
    ) -> Result<WorthQueryProgramAdoptionRequirements, WorthQueryBranchAdoptionPreparationDenial>
    {
        preparation::requirements(self, target)
    }

    /// Reads every workflow definition and live instance this exact branch
    /// incarnation must decide before moving to `target`, with the
    /// dispositions the adoption law leaves open for each.
    pub fn workflow_adoption_inventory(
        &self,
        target: &ApplicationProgramRevision,
        expected_requirements: &WorthQueryProgramAdoptionRequirements,
        maximum_work_units: usize,
    ) -> Result<WorthQueryWorkflowAdoptionInventory, WorthQueryBranchAdoptionPreparationDenial>
    {
        preparation::workflow_inventory(self, target, expected_requirements, maximum_work_units)
    }

    /// Prepares one branch-local move to `target` against the exact selected
    /// occurrence and the caller-visible requirements admitted beforehand.
    pub fn prepare_branch_adoption(
        &self,
        target: &ApplicationProgramRevision,
        expected_requirements: &WorthQueryProgramAdoptionRequirements,
        maximum_selection_work: usize,
        request: &WorthQueryRequestScope,
    ) -> Result<WorthQueryPreparedBranchAdoption, WorthQueryBranchAdoptionPreparationDenial> {
        self.prepare_branch_adoption_inner(
            target,
            expected_requirements,
            None,
            None,
            maximum_selection_work,
            request,
        )
    }

    pub fn prepare_branch_adoption_with_migration(
        &self,
        target: &ApplicationProgramRevision,
        expected_requirements: &WorthQueryProgramAdoptionRequirements,
        migration: WorthQueryPreparedProgramMigration,
        maximum_selection_work: usize,
        request: &WorthQueryRequestScope,
    ) -> Result<WorthQueryPreparedBranchAdoption, WorthQueryBranchAdoptionPreparationDenial> {
        self.prepare_branch_adoption_inner(
            target,
            expected_requirements,
            Some(migration),
            None,
            maximum_selection_work,
            request,
        )
    }

    /// Prepares one branch-local move with explicit workflow dispositions,
    /// decided against [`Self::workflow_adoption_inventory`], and an optional
    /// admitted migration.
    pub fn prepare_branch_adoption_with_choices(
        &self,
        target: &ApplicationProgramRevision,
        expected_requirements: &WorthQueryProgramAdoptionRequirements,
        migration: Option<WorthQueryPreparedProgramMigration>,
        workflow: Option<WorthQueryWorkflowDispositions>,
        maximum_selection_work: usize,
        request: &WorthQueryRequestScope,
    ) -> Result<WorthQueryPreparedBranchAdoption, WorthQueryBranchAdoptionPreparationDenial> {
        self.prepare_branch_adoption_inner(
            target,
            expected_requirements,
            migration,
            workflow,
            maximum_selection_work,
            request,
        )
    }

    fn prepare_branch_adoption_inner(
        &self,
        target: &ApplicationProgramRevision,
        expected_requirements: &WorthQueryProgramAdoptionRequirements,
        migration: Option<WorthQueryPreparedProgramMigration>,
        workflow: Option<WorthQueryWorkflowDispositions>,
        maximum_selection_work: usize,
        request: &WorthQueryRequestScope,
    ) -> Result<WorthQueryPreparedBranchAdoption, WorthQueryBranchAdoptionPreparationDenial> {
        let application = self.application();
        let gate = application
            .product_runtime
            .activations
            .gate(self.product().branch_identity())
            .map_err(|denial| {
                WorthQueryBranchAdoptionPreparationDenial::ProductActivation(denial.into())
            })?;
        gate.publish(|| {
            preparation::prepare(
                self,
                target,
                expected_requirements,
                migration,
                workflow,
                maximum_selection_work,
                request,
            )
        })
        .map_err(|denial| {
            WorthQueryBranchAdoptionPreparationDenial::ProductActivation(denial.into())
        })?
    }
}
