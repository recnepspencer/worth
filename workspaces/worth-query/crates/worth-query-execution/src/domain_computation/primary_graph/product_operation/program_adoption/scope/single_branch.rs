use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;
use worth_query_declaration::facade::application_program::ApplicationProgramRevision;
use worth_query_installation::facade::{ApplicationSchema, WorthQueryProgramAdoptionRequirements};

use super::super::{
    preparation, WorthQueryBranchAdoptionPreparationDenial, WorthQueryPreparedBranchAdoption,
    WorthQueryPreparedProgramMigration,
};
use crate::domain_computation::primary_graph::product_operation::WorthQuerySelectedProductOperation;
use crate::domain_computation::primary_graph::workflow::WorthQueryWorkflowDispositions;

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
        principal: &worth_query_admission::facade::authenticated_principal::WorthQueryAuthenticatedExternalPrincipal<Schema>,
        request: &WorthQueryRequestScope,
    ) -> Result<WorthQueryPreparedBranchAdoption, WorthQueryBranchAdoptionPreparationDenial> {
        self.application()
            .with_application_advancement(request, |phase| {
                self.prepare_branch_adoption_in_advancement(
                    &phase,
                    target,
                    expected_requirements,
                    None,
                    None,
                    maximum_selection_work,
                    principal,
                    request,
                )
            })
            .map_err(WorthQueryBranchAdoptionPreparationDenial::ExecutionDenied)?
    }

    pub fn prepare_branch_adoption_with_migration(
        &self,
        target: &ApplicationProgramRevision,
        expected_requirements: &WorthQueryProgramAdoptionRequirements,
        migration: WorthQueryPreparedProgramMigration,
        maximum_selection_work: usize,
        principal: &worth_query_admission::facade::authenticated_principal::WorthQueryAuthenticatedExternalPrincipal<Schema>,
        request: &WorthQueryRequestScope,
    ) -> Result<WorthQueryPreparedBranchAdoption, WorthQueryBranchAdoptionPreparationDenial> {
        self.application()
            .with_application_advancement(request, |phase| {
                self.prepare_branch_adoption_in_advancement(
                    &phase,
                    target,
                    expected_requirements,
                    Some(migration),
                    None,
                    maximum_selection_work,
                    principal,
                    request,
                )
            })
            .map_err(WorthQueryBranchAdoptionPreparationDenial::ExecutionDenied)?
    }

    /// Prepares one branch-local move with explicit workflow dispositions,
    /// resolved against private fresh owner truth, and an optional
    /// admitted migration.
    /// ```
    /// use worth_query_execution::facade::primary_graph::{WorthQuerySelectedProductOperation, WorthQueryWorkflowDispositions};
    /// use worth_query_installation::facade::{ApplicationSchema, WorthQueryProgramAdoptionRequirements};
    /// use worth_query_declaration::facade::application_program::ApplicationProgramRevision;
    /// use worth_query_admission::facade::authenticated_principal::{WorthQueryAuthenticatedExternalPrincipal, WorthQueryRequestScope};
    /// fn explicit<S:ApplicationSchema>(selected:&WorthQuerySelectedProductOperation<'_, S>, target:&ApplicationProgramRevision,
    ///     requirements:&WorthQueryProgramAdoptionRequirements, principal:&WorthQueryAuthenticatedExternalPrincipal<S>, scope:&WorthQueryRequestScope) {
    ///     let _ = selected.prepare_branch_adoption_with_choices(target,requirements,None,Some(WorthQueryWorkflowDispositions::carry_compatible()),64,principal,scope);
    /// }
    /// ```
    pub fn prepare_branch_adoption_with_choices(
        &self,
        target: &ApplicationProgramRevision,
        expected_requirements: &WorthQueryProgramAdoptionRequirements,
        migration: Option<WorthQueryPreparedProgramMigration>,
        workflow: Option<WorthQueryWorkflowDispositions>,
        maximum_selection_work: usize,
        principal: &worth_query_admission::facade::authenticated_principal::WorthQueryAuthenticatedExternalPrincipal<Schema>,
        request: &WorthQueryRequestScope,
    ) -> Result<WorthQueryPreparedBranchAdoption, WorthQueryBranchAdoptionPreparationDenial> {
        self.application()
            .with_application_advancement(request, |phase| {
                self.prepare_branch_adoption_in_advancement(
                    &phase,
                    target,
                    expected_requirements,
                    migration,
                    workflow,
                    maximum_selection_work,
                    principal,
                    request,
                )
            })
            .map_err(WorthQueryBranchAdoptionPreparationDenial::ExecutionDenied)?
    }

    #[doc(hidden)]
    pub fn prepare_branch_adoption_in_advancement(
        &self,

        phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
        target: &ApplicationProgramRevision,
        expected_requirements: &WorthQueryProgramAdoptionRequirements,
        migration: Option<WorthQueryPreparedProgramMigration>,
        workflow: Option<WorthQueryWorkflowDispositions>,
        maximum_selection_work: usize,
        principal: &worth_query_admission::facade::authenticated_principal::WorthQueryAuthenticatedExternalPrincipal<Schema>,
        request: &WorthQueryRequestScope,
    ) -> Result<WorthQueryPreparedBranchAdoption, WorthQueryBranchAdoptionPreparationDenial> {
        let application = self.application();
        let _phase_request = phase
            .execution_request_for(&application.product_runtime)
            .map_err(|cause| {
                WorthQueryBranchAdoptionPreparationDenial::ExecutionDenied(cause.into())
            })?;
        use super::super::authorization::{AdoptionEffect, AdoptionEffectOutcome};
        match self.with_program_adoption(
            phase,
            principal,
            request,
            target,
            AdoptionEffect::Prepare {
                requirements: expected_requirements,
                migration,
                workflow,
                maximum_selection_work,
            },
        ) {
            Ok(AdoptionEffectOutcome::Prepared(outcome)) => outcome,
            Err(refusal) => Err(WorthQueryBranchAdoptionPreparationDenial::Authorization(
                refusal.cause,
            )),
            Ok(_) => unreachable!("preparation dispatch preserves the requested effect"),
        }
    }
}
