//! Detached definition inventories and native generation inspection.
use super::*;
use crate::history::data::BranchId;
use crate::indexes::data::{DerivedIndexDefinition, DerivedIndexGeneration, DerivedIndexId};
impl<'runtime> IndexAccess<'runtime> {
    pub fn generation_selection_counters(
        &self,
    ) -> crate::indexes::data::DerivedIndexSelectionCounters {
        self.runtime.indexes.generation_selection_counters()
    }

    pub fn latest_generation(
        &self,
        index_id: DerivedIndexId,
        branch_id: &BranchId,
    ) -> Option<std::sync::Arc<DerivedIndexGeneration>> {
        let definition = self.runtime.indexes.definition(index_id)?;
        self.runtime
            .indexes
            .latest_generation(index_id, definition.branch_scoped.then_some(branch_id))
    }

    /// Finds the installed definition with the same semantic identity.
    ///
    /// The caller-supplied numeric identity is deliberately ignored: a
    /// recovered owner must bind the identity restored with the checkpoint,
    /// not allocate a look-alike definition after recovery.
    pub fn matching_definition(
        &self,
        expected: &DerivedIndexDefinition,
    ) -> Option<DerivedIndexDefinition> {
        self.runtime
            .indexes
            .definitions()
            .into_iter()
            .find(|candidate| {
                candidate.name == expected.name
                    && candidate.kind == expected.kind
                    && candidate.branch_scoped == expected.branch_scoped
            })
            .map(|definition| definition.as_ref().clone())
    }

    /// Detaches one exact-definition inventory for callers binding several
    /// restored indexes. Each lookup preserves the owner's numeric identity.
    pub fn definition_lookup_snapshot(&self) -> DerivedIndexDefinitionLookup {
        DerivedIndexDefinitionLookup::new(self.runtime.indexes.definitions())
    }

    /// The generation published for one index by the branch that authored a
    /// commit.
    ///
    /// This names the receipt's branch. It is not a currency check for a
    /// fresh fork that selects the same commit: that fork needs its own
    /// branch-scoped generation, so judge observed reads with
    /// [`Self::published_generation_for_observation`].
    pub fn published_generation_for_commit(
        &self,
        index_id: DerivedIndexId,
        commit: &crate::history::data::RelationalCommitReceipt,
    ) -> Option<std::sync::Arc<DerivedIndexGeneration>> {
        let definition = self.runtime.indexes.definition(index_id)?;
        self.runtime.indexes.published_generation_for_commit(
            index_id,
            definition.branch_scoped.then_some(&commit.branch_id),
            commit.commit_id,
            commit.version_id,
        )
    }

    /// The generation published for one index at an exact branch observation.
    ///
    /// A fork that has not committed yet selects its parent's commit, so the
    /// commit receipt names the parent branch. A branch-scoped index is still
    /// judged on the observing branch, which has to publish its own
    /// generation before its reads can use the index.
    pub fn published_generation_for_observation(
        &self,
        index_id: DerivedIndexId,
        observation: &crate::mvcc::RelationalBranchObservation,
    ) -> Option<std::sync::Arc<DerivedIndexGeneration>> {
        let definition = self.runtime.indexes.definition(index_id)?;
        self.runtime.indexes.published_generation_for_commit(
            index_id,
            definition
                .branch_scoped
                .then_some(observation.identity().branch_id()),
            observation.commit_id()?,
            observation.version_id(),
        )
    }

    pub fn generations_for_version(
        &self,
        version_id: crate::identity::data::VersionId,
    ) -> Vec<DerivedIndexGeneration> {
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
        generations
    }

    pub(crate) fn definitions_snapshot(&self) -> Vec<DerivedIndexDefinition> {
        self.runtime
            .indexes
            .definitions()
            .into_iter()
            .map(|definition| definition.as_ref().clone())
            .collect()
    }

    #[cfg(test)]
    pub(crate) fn generations_snapshot(&self) -> Vec<DerivedIndexGeneration> {
        self.runtime
            .indexes
            .all_generations()
            .into_iter()
            .map(|generation| generation.as_ref().clone())
            .collect()
    }

    #[cfg(test)]
    pub(crate) fn entity_unique_field_entries(
        &self,
        field_locator: &worth_foundational::facade::AspectFieldLocator,
    ) -> Option<
        std::collections::BTreeMap<
            crate::storage::data::AuthoritativeFieldComparisonKey,
            std::collections::BTreeSet<crate::identity::data::EntityId>,
        >,
    > {
        self.runtime
            .indexes
            .with_unique_index(|index| index.get(field_locator).cloned())
    }
}
