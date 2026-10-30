//! Fallible generation payload reads admit the existing checkpoint image first.
use super::*;
use crate::history::data::BranchId;
use crate::indexes::data::{DerivedIndexGeneration, DerivedIndexId};
impl IndexAccess<'_> {
    pub fn read_latest_generation(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        index_id: DerivedIndexId,
        branch_id: &BranchId,
    ) -> Result<
        Option<std::sync::Arc<DerivedIndexGeneration>>,
        crate::execution::RelationalExecutionDenialCause,
    > {
        crate::execution::check_request(execution)?;
        let Some(definition) = self.runtime.indexes.definition(index_id) else {
            return Ok(None);
        };
        self.runtime
            .indexes
            .definition_read_binding()
            .demand_checkpoint_image(execution)?;
        Ok(self
            .runtime
            .indexes
            .latest_generation(index_id, definition.branch_scoped.then_some(branch_id)))
    }

    /// The generation published for one index by the branch that authored a
    /// commit.
    ///
    /// This names the receipt's branch. It is not a currency check for a
    /// fresh fork that selects the same commit: that fork needs its own
    /// branch-scoped generation, so judge observed reads with
    /// [`Self::read_published_generation_for_observation`].
    pub fn read_published_generation_for_commit(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        index_id: DerivedIndexId,
        commit: &crate::history::data::RelationalCommitReceipt,
    ) -> Result<
        Option<std::sync::Arc<DerivedIndexGeneration>>,
        crate::execution::RelationalExecutionDenialCause,
    > {
        crate::execution::check_request(execution)?;
        let Some(definition) = self.runtime.indexes.definition(index_id) else {
            return Ok(None);
        };
        self.runtime
            .indexes
            .definition_read_binding()
            .demand_checkpoint_image(execution)?;
        Ok(self.runtime.indexes.published_generation_for_commit(
            index_id,
            definition.branch_scoped.then_some(&commit.branch_id),
            commit.commit_id,
            commit.version_id,
        ))
    }

    /// The generation published for one index at an exact branch observation.
    ///
    /// A fork that has not committed yet selects its parent's commit, so the
    /// commit receipt names the parent branch. A branch-scoped index is still
    /// judged on the observing branch, which has to publish its own
    /// generation before its reads can use the index.
    pub fn read_published_generation_for_observation(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        index_id: DerivedIndexId,
        observation: &crate::mvcc::RelationalBranchObservation,
    ) -> Result<
        Option<std::sync::Arc<DerivedIndexGeneration>>,
        crate::execution::RelationalExecutionDenialCause,
    > {
        crate::execution::check_request(execution)?;
        let Some(commit) = observation.commit_id() else {
            return Ok(None);
        };
        let Some(definition) = self.runtime.indexes.definition(index_id) else {
            return Ok(None);
        };
        self.runtime
            .indexes
            .definition_read_binding()
            .demand_checkpoint_image(execution)?;
        Ok(self.runtime.indexes.published_generation_for_commit(
            index_id,
            definition
                .branch_scoped
                .then_some(observation.identity().branch_id()),
            commit,
            observation.version_id(),
        ))
    }

    #[expect(
        clippy::disallowed_methods,
        reason = "Comparison uses declared semantic keys; type names label diagnostics only"
    )]
    pub fn read_generations_for_version(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        version_id: crate::identity::data::VersionId,
    ) -> Result<Vec<DerivedIndexGeneration>, crate::execution::RelationalExecutionDenialCause> {
        crate::execution::check_request(execution)?;
        self.runtime
            .indexes
            .definition_read_binding()
            .demand_checkpoint_image(execution)?;
        let mut generations = self
            .runtime
            .indexes
            .all_generations()
            .into_iter()
            .filter(|generation| generation.applicability.version_id <= version_id)
            .map(|generation| generation.as_ref().clone())
            .collect::<Vec<_>>();
        generations.sort_by(|left, right| {
            left.applicability
                .branch_id
                .cmp(&right.applicability.branch_id)
                .then_with(|| left.source_commit_id.cmp(&right.source_commit_id))
                .then_with(|| left.generation_id.cmp(&right.generation_id))
        });
        Ok(generations)
    }
}
