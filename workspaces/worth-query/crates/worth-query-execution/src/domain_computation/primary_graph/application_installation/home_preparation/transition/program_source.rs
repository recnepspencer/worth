//! Observe the source branch's admitted programme once, before reserving its open.
use super::*;
impl<S: ApplicationSchemaComposition, P> PreparedHomeProgramTransitionTarget<S, P> {
    pub fn admit_home_transition_program_source<A: ApplicationSchemaComposition, AP>(
        &self,
        predecessor: &PreparedExistingHomeSignalProgram<A, AP>,
        native: &RelationalRuntime,
        source: &worth_runtime_world::facade::ProductBranchObservation,
        issuer: &worth_proof::InstalledBranchProgramSourceOwner,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<worth_proof::AdmittedBranchProgramSource, WorthQueryPrimaryGraphInstallationDenial>
    {
        let (layout, receipt) = predecessor.predecessor_native_meaning();
        if receipt.runtime_instance_id() != native.runtime_instance_id() {
            return Err(author_source::denied(
                "transition source belongs to another Native owner",
            ));
        }
        let view = native
            .read_truth()
            .project_observation(&source.basis().relational_basis().observation())
            .map_err(|cause| {
                author_source::denied(format!("transition branch source refused: {cause:?}"))
            })?;
        let mut selected = None;
        let (activation, _) = super::super::super::super::bootstrap::read_activation(
            execution,
            &view,
            layout.program_activation(),
            usize::MAX,
            "transition branch does not activate either admitted programme",
            |rendering| {
                selected = predecessor.predecessor_program_for_rendering(rendering).or_else(|| self._pending._state.support.roster.entries().iter().find(|entry| super::super::super::super::program_occurrence::program_revision_rendering(entry.revision()) == *rendering).map(|entry| *entry.revision()));
                selected.is_some()
            },
        )?;
        if predecessor.predecessor_activation().1.published() != Some(activation) {
            return Err(author_source::denied(
                "transition source names another activation owner",
            ));
        }
        let revision = *selected
            .expect("the actual activation was admitted against its roster")
            .as_bytes();
        let commitment =
            worth_runtime_world::facade::world_branch_program_source_commitment(source, revision)
                .map_err(|cause| {
                author_source::denied(format!("transition World source refused: {cause:?}"))
            })?;
        Ok(issuer.admit_source(revision, commitment))
    }
}
