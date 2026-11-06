//! Sealed test doors over this module's private members.
use super::*;

impl Denial {
    #[doc(hidden)]
    pub fn retry_denial_for_test(self) -> Retry {
        Self::retry_denial(self)
    }
}
