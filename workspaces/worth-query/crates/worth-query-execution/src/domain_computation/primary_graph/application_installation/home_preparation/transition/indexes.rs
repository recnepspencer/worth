//! Explicit migration reconstruction seals the same candidate's complete indexes.
use super::*;
use worth_foundational::facade::HomeProgramTransitionStepRole;
use worth_relational::facade::{
    indexes::{
        DerivedIndexMaintenanceBudget, DerivedIndexMaintenanceDenial, DerivedIndexMaintenanceWork,
    },
    mvcc::PreparedRelationalCommitCandidate,
};
impl<Schema: ApplicationSchemaComposition, Program>
    PreparedHomeProgramTransitionTarget<Schema, Program>
{
    /// This is the explicit program-migration lane, not ordinary incremental
    /// business publication. The caller supplies finite migration resources;
    /// the original request still controls all reconstruction and allocation.
    pub fn prepare_home_program_transition_indexes(
        &self,
        request: &worth_execution::ExecutionRequest<'_, '_>,
        native: &RelationalRuntime,
        candidate: &mut PreparedRelationalCommitCandidate,
        role: HomeProgramTransitionStepRole,
        budget: DerivedIndexMaintenanceBudget,
    ) -> Result<DerivedIndexMaintenanceWork, DerivedIndexMaintenanceDenial> {
        let indexes = match role {
            HomeProgramTransitionStepRole::BeforeTarget => &self.before_indexes,
            HomeProgramTransitionStepRole::Target | HomeProgramTransitionStepRole::AfterTarget => {
                &self.target_indexes
            }
        };
        native
            .index_authority()
            .prepare_for_candidate(candidate, indexes, None, budget, request)
    }
}
