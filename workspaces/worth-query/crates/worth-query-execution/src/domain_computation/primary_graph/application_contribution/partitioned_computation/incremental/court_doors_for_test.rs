//! Sealed test doors over this module's private members.
use super::*;

impl RetainedBasisToken {
    #[doc(hidden)]
    pub fn new_for_test(field0: RetainedBasis) -> Self {
        Self(field0)
    }
}
