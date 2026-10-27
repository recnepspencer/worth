//! No-load serving-catalog residency and generation-selection telemetry.
use super::*;
use crate::indexes::data::DerivedIndexId;
impl IndexAccess<'_> {
    pub fn generation_selection_counters(
        &self,
    ) -> crate::indexes::data::DerivedIndexSelectionCounters {
        self.runtime.indexes.generation_selection_counters()
    }

    /// Observe serving-catalog residency without loading, reading entries or
    /// granting read authority. Deferred image state is distinct from absence.
    pub fn generation_residency_for_observation(
        &self,
        index_id: DerivedIndexId,
        observation: &crate::mvcc::RelationalBranchObservation,
    ) -> crate::indexes::data::DerivedIndexGenerationResidency {
        use crate::indexes::data::DerivedIndexGenerationResidency as State;
        let Some(commit) = observation.commit_id() else {
            return State::Absent;
        };
        let Some(definition) = self.runtime.indexes.definition(index_id) else {
            return State::Absent;
        };
        if self.runtime.indexes.carried_checkpoint_image().is_some() {
            return State::DeferredImage;
        }
        if self
            .runtime
            .indexes
            .published_generation_for_commit(
                index_id,
                definition
                    .branch_scoped
                    .then_some(observation.identity().branch_id()),
                commit,
                observation.version_id(),
            )
            .is_some()
        {
            State::Published
        } else {
            State::Absent
        }
    }
}
