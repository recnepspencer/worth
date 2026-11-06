//! Sealed test doors over this module's private members.
use super::*;

impl SourceInvalidationOwner {
    #[doc(hidden)]
    pub fn retire_settlements_for_test(&self, candidates: &[Identity]) -> Vec<Identity> {
        Self::retire_settlements(self, candidates)
    }
}
