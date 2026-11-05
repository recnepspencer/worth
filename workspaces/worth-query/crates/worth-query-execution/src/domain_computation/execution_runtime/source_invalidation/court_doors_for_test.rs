//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryInvalidationResources {
    #[doc(hidden)]
    pub fn attach_runtime_observations_for_test(&self) {
        Self::attach_runtime_observations(self)
    }
}

impl WorthQueryInvalidationResources {
    #[doc(hidden)]
    pub fn invalidation_equivalence_is_armed_for_test(&self) -> bool {
        Self::invalidation_equivalence_is_armed(self)
    }
}
