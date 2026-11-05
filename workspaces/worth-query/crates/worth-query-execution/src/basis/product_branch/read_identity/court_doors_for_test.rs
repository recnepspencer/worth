//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryProductBranchReadIdentity {
    #[doc(hidden)]
    pub fn from_observation_for_test(observation: &ProductBranchObservation) -> Self {
        Self::from_observation(observation)
    }
}

impl WorthQueryProductBranchReadIdentity {
    #[doc(hidden)]
    pub fn matches_observation_for_test(&self, observation: &ProductBranchObservation) -> bool {
        Self::matches_observation(self, observation)
    }
}
