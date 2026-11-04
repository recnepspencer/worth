//! Sealed test doors over this module's private members.
use super::*;

impl<'a> ValueDecodeSession<'a> {
    #[doc(hidden)]
    pub fn denial_for_test(&self) -> Option<Stop> {
        Self::denial(self)
    }
}
