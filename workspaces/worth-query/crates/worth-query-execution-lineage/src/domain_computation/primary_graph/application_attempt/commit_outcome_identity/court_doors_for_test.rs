//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryApplicationCommitOutcomeIdentity {
    #[doc(hidden)]
    pub fn mint_for_test() -> Option<Self> {
        Self::mint()
    }
}

impl WorthQueryApplicationCommitOutcomeIdentity {
    #[doc(hidden)]
    pub fn restore_for_test(value: u64) -> Option<Self> {
        Self::restore(value)
    }
}
