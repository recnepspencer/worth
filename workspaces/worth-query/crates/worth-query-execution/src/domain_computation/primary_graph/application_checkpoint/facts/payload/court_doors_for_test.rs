//! Sealed test doors over this module's private members.
use super::*;

impl FactPayload {
    #[doc(hidden)]
    pub fn iter_for_test(&self) -> FactIter<'_> {
        Self::iter(self)
    }
}
