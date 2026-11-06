//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryProducerOutputConnectionDenial {
    #[doc(hidden)]
    pub fn include_for_test(&mut self, candidate: Self) {
        Self::include(self, candidate)
    }
}
