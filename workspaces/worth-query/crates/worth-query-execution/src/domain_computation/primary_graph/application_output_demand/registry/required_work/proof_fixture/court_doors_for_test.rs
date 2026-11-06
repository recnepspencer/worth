//! Sealed test doors over this module's private members.
use super::*;

impl RequiredWorkMembership {
    #[doc(hidden)]
    pub fn fixture_is_unresolved_for_test(&self) -> bool {
        Self::fixture_is_unresolved(self)
    }
}
