//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryApplicationCommittedChanges {
    #[doc(hidden)]
    pub fn shares_original_for_test(
        &self, other: &Self) -> bool {
        Self::shares_original(self, other)
    }
}
