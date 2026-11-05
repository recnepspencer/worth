//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryApplicationOutputLineage {
    #[doc(hidden)]
    pub fn retain_computation_for_test(&self, sealed: SealedComputationRun) -> RecordedComputation {
        Self::retain_computation(self, sealed)
    }
}
