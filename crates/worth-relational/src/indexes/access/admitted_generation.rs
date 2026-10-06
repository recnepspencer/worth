use crate::indexes::data::{DerivedIndexId, SelectedIndexGenerationAdmissionStop};
use crate::mvcc::RelationalBranchObservation;

use super::IndexAccess;

impl IndexAccess<'_> {
    /// Whether exactly one selected index is already published for this
    /// observation. Missing currency is reported without rebuilding indexes.
    pub fn has_published_generation_for_observation_admitted<Stop>(
        &self,
        index: DerivedIndexId,
        observation: &RelationalBranchObservation,
        prepare: impl FnMut(u64, u64) -> Result<(), Stop>,
    ) -> Result<bool, SelectedIndexGenerationAdmissionStop<Stop>> {
        self.runtime
            .indexes
            .has_published_generation_for_observation_admitted(index, observation, prepare)
    }
}
