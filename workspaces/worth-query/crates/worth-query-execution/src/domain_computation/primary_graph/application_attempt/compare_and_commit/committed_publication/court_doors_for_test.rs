//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryCommittedProductPublication {
    #[doc(hidden)]
    pub fn is_selected_at_for_test(
        &self, observation: &ProductBranchObservation) -> bool {
        Self::is_selected_at(self, observation)
    }
}

impl WorthQueryCommittedProductPublication {
    #[doc(hidden)]
    pub fn take_output_demand_observation_for_test(
        &self) -> Option<worth_runtime_world::facade::ProductBranchObservation> {
        Self::take_output_demand_observation(self)
    }
}
