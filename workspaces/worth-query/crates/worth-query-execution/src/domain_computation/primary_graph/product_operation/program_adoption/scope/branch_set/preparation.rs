//! Freshly prepare the ordered remaining branch targets.
use super::*;
impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    pub fn prepare_branch_set_adoption(
        &self,

        coverage: WorthQueryOrderedProgramAdoptionCoverage,
        target: &ApplicationProgramRevision,
        maximum_selection_work_per_branch: usize,
        principal: &worth_query_admission::facade::authenticated_principal::WorthQueryAuthenticatedExternalPrincipal<Schema>,
        request: &WorthQueryRequestScope,
    ) -> Result<WorthQueryPreparedBranchSetAdoption, WorthQueryBranchSetAdoptionPreparationDenial>
    {
        self.with_application_advancement(request, |phase| {
            let phase_request = phase
                .execution_request_for(&self.runtime.product_runtime)
                .map_err(|cause| {
                    WorthQueryBranchSetAdoptionPreparationDenial::ExecutionDenied(cause.into())
                })?;
            let execution = &phase_request;
            let (pending, total_selection_work_units) = self.prepare_branch_set_targets(
                execution,
                &phase,
                &coverage.branches,
                target,
                maximum_selection_work_per_branch,
                principal,
                request,
            )?;
            let mut progress = Vec::new();
            progress
                .try_reserve_exact(coverage.branches.len())
                .map_err(|_| {
                    WorthQueryBranchSetAdoptionPreparationDenial::RetentionAllocationRejected
                })?;
            Ok(WorthQueryPreparedBranchSetAdoption {
                pending,
                progress,
                resolution_required: None,
                target: *target,
                total_selection_work_units,
            })
        })
        .map_err(WorthQueryBranchSetAdoptionPreparationDenial::ExecutionDenied)?
    }

    pub(super) fn prepare_branch_set_targets(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
        branches: &[WorthQueryProductBranch],
        target: &ApplicationProgramRevision,
        maximum_selection_work_per_branch: usize,
        principal: &worth_query_admission::facade::authenticated_principal::WorthQueryAuthenticatedExternalPrincipal<Schema>,
        request: &WorthQueryRequestScope,
    ) -> Result<
        (VecDeque<PendingBranchAdoption>, usize),
        WorthQueryBranchSetAdoptionPreparationDenial,
    > {
        self.runtime.product_runtime
            .admit_directory_capability(crate::domain_computation::primary_graph::WorthQueryOperationDurableCapability::ProgramAdoption)
            .map_err(WorthQueryBranchSetAdoptionPreparationDenial::DurableStateAdmission)?;
        let mut pending = VecDeque::new();
        pending.try_reserve(branches.len()).map_err(|_| {
            WorthQueryBranchSetAdoptionPreparationDenial::RetentionAllocationRejected
        })?;
        let mut total_selection_work_units = 0usize;
        for &branch in branches {
            let selected = self
                .on_branch(branch)
                .select_with_request(execution)
                .map_err(|denial| {
                    WorthQueryBranchSetAdoptionPreparationDenial::ProductSelection {
                        branch,
                        denial,
                    }
                })?;
            let requirements = selected
                .branch_adoption_requirements(target)
                .map_err(
                    |denial| WorthQueryBranchSetAdoptionPreparationDenial::Adoption {
                        branch,
                        denial,
                    },
                )?;
            let adoption = selected
                .prepare_branch_adoption_in_advancement(
                    phase,
                    target,
                    &requirements,
                    None,
                    None,
                    maximum_selection_work_per_branch,
                    principal,
                    request,
                )
                .map_err(
                    |denial| WorthQueryBranchSetAdoptionPreparationDenial::Adoption {
                        branch,
                        denial,
                    },
                )?;
            total_selection_work_units = total_selection_work_units
                .checked_add(adoption.selection_work_units())
                .ok_or(WorthQueryBranchSetAdoptionPreparationDenial::WorkAccountingOverflow)?;
            pending.push_back(PendingBranchAdoption { branch, adoption });
        }
        Ok((pending, total_selection_work_units))
    }
}
