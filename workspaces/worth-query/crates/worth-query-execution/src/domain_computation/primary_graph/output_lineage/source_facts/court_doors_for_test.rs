//! Sealed test doors over this module's private members.
use super::*;

impl RetainedSourceFacts {
    #[doc(hidden)]
    pub fn len_for_test(&self) -> usize {
        Self::len(self)
    }
}
