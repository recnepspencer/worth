//! Sealed test doors over this module's private members.
use super::*;

impl FactPayload {
    #[doc(hidden)]
    pub fn iter_for_test(&self) -> FactIter<'_> {
        Self::iter(self)
    }
}

impl FactPayload {
    #[doc(hidden)]
    pub fn is_empty_for_test(&self) -> bool {
        Self::is_empty(self)
    }
}

impl FactPayload {
    #[doc(hidden)]
    pub fn same_backing_for_test(&self, other: &Self) -> bool {
        Self::same_backing(self, other)
    }
}

impl FactPayload {
    #[doc(hidden)]
    pub fn len_for_test(&self) -> usize {
        Self::len(self)
    }
}
