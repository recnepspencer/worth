//! Sealed test doors over this module's private members.
use super::*;

impl QueuedRequiredRoots {
    #[doc(hidden)]
    pub fn defer_caller_for_test(&mut self) {
        Self::defer_caller(self)
    }
}

impl QueuedRequiredRoots {
    #[doc(hidden)]
    pub fn draining_for_test(&self) -> bool {
        Self::draining(self)
    }
}

impl QueuedRequiredRoots {
    #[doc(hidden)]
    pub fn new_for_test() -> Self {
        Self::new()
    }
}

impl QueuedRequiredRoots {
    #[doc(hidden)]
    pub fn take_caller_for_test(&mut self) -> Option<WorthQueryOutputDemandAdvance> {
        Self::take_caller(self)
    }
}
