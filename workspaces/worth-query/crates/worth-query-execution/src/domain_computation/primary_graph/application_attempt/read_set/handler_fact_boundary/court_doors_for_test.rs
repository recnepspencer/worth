//! Sealed test doors over this module's private members.
use super::*;

impl CompletedHandlerFactBoundary {
    #[doc(hidden)]
    pub fn handler_fact_count_for_test(&self) -> usize {
        Self::handler_fact_count(self)
    }
}
