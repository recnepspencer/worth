//! Sealed test doors over this module's private members.
use super::*;

impl SealedComputationRun {
    #[doc(hidden)]
    pub fn state_for_test(&self) -> &RetainedComputation {
        &self.state
    }
}
