use crate::indexes::data::SelectedIndexReadWork;
use crate::indexes::data::{DerivedIndexId, SelectedIndexGenerationAdmissionStop};
use crate::mvcc::RelationalBranchObservation;

use super::IndexAccess;

impl IndexAccess<'_> {
    /// Whether exactly one selected index is already published for this
    /// observation. Missing currency is reported without rebuilding indexes.
    /// The callback budgets Operation against logical work. OrderedNavigation
    /// is one logical operation plus a capacity-bounded single keyed descent;
    /// fixed validation, candidates, row reads and payload use Operation.
    pub fn has_published_generation_for_observation_admitted<Stop>(
        &self,
        index: DerivedIndexId,
        observation: &RelationalBranchObservation,
        prepare: impl FnMut(SelectedIndexReadWork, u64) -> Result<(), Stop>,
    ) -> Result<bool, SelectedIndexGenerationAdmissionStop<Stop>> {
        self.runtime
            .indexes
            .has_published_generation_for_observation_admitted(index, observation, prepare)
    }
}
