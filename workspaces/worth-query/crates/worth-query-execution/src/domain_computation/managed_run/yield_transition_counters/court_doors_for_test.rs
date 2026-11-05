//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryYieldTransitionCounters {
    #[doc(hidden)]
    pub fn attempted_bridge_finalization_for_test(&mut self) {
        Self::attempted_bridge_finalization(self)
    }
}

impl WorthQueryYieldTransitionCounters {
    #[doc(hidden)]
    pub fn attempted_checkpoint_suspension_for_test(&mut self) {
        Self::attempted_checkpoint_suspension(self)
    }
}

impl WorthQueryYieldTransitionCounters {
    #[doc(hidden)]
    pub fn classified_eligibility_for_test(&mut self) {
        Self::classified_eligibility(self)
    }
}

impl WorthQueryYieldTransitionCounters {
    #[doc(hidden)]
    pub fn minted_yielded_capability_for_test(&mut self) {
        Self::minted_yielded_capability(self)
    }
}

impl WorthQueryYieldTransitionCounters {
    #[doc(hidden)]
    pub fn observed_artifact_registry_for_test(&mut self) {
        Self::observed_artifact_registry(self)
    }
}

impl WorthQueryYieldTransitionCounters {
    #[doc(hidden)]
    pub fn observed_checkpoint_retained_bytes_for_test(&mut self, count: usize) {
        Self::observed_checkpoint_retained_bytes(self, count)
    }
}

impl WorthQueryYieldTransitionCounters {
    #[doc(hidden)]
    pub fn validated_retained_resources_for_test(&mut self) {
        Self::validated_retained_resources(self)
    }
}
