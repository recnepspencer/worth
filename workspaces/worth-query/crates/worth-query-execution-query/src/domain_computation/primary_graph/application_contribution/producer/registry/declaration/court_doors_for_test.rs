//! Sealed test doors over this module's private members.
use super::*;

impl DeclaredProducerBinding {
    #[doc(hidden)]
    pub fn has_same_meaning_as_for_test(&self, expected: &Self) -> bool {
        Self::has_same_meaning_as(self, expected)
    }
}
