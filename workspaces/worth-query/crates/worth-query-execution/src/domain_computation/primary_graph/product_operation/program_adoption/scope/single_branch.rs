use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;
use worth_query_declaration::facade::application_program::ApplicationProgramRevision;
use worth_query_installation::facade::{ApplicationSchema, WorthQueryProgramAdoptionRequirements};

use super::super::{
    preparation, WorthQueryBranchAdoptionPreparationDenial, WorthQueryPreparedBranchAdoption,
    WorthQueryPreparedProgramMigration,
};
use crate::domain_computation::primary_graph::product_operation::WorthQuerySelectedProductOperation;

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
            maximum_selection_work,
            request,
        )
    }

    fn prepare_branch_adoption_inner(
        &self,
        target: &ApplicationProgramRevision,
        expected_requirements: &WorthQueryProgramAdoptionRequirements,
        migration: Option<WorthQueryPreparedProgramMigration>,
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
                maximum_selection_work,
                request,
            )
        })
        .map_err(|denial| {
            WorthQueryBranchAdoptionPreparationDenial::ProductActivation(denial.into())
        })?
    }
}
