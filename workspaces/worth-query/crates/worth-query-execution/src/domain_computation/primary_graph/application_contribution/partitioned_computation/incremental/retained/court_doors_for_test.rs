//! Sealed test doors over this module's private members.
use super::*;

impl SealedComputationRun {
    #[doc(hidden)]
    pub fn state_for_test(&self) -> &RetainedComputation {
        &self.state
    }
}

impl RetainedComputation {
    #[doc(hidden)]
    pub fn facts_for_test(&self) -> &SealedComputationFacts {
        &self.facts
    }
}

impl RetainedComputation {
    #[doc(hidden)]
    pub fn facts_mut_for_test(&mut self) -> &mut SealedComputationFacts {
        &mut self.facts
    }
}

impl SealedComputationRun {
    #[doc(hidden)]
    pub fn state_into_for_test(self) -> RetainedComputation {
        self.state
    }
}
