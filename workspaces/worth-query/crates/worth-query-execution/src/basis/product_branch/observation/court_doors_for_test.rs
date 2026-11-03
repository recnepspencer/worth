//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryProductBranchLease {
    #[doc(hidden)]
    pub fn observation_for_test(&self) -> &worth_runtime_world::facade::ProductBranchObservation {
        Self::observation(self)
    }
}
