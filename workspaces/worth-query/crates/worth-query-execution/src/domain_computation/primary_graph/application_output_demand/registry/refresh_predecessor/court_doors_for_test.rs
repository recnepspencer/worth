//! Sealed test doors over this module's private members.
use super::*;

impl<'a> OutputRefreshPredecessor<'a> {
    #[doc(hidden)]
    pub fn of_for_test(
        authority: &'a WorthQueryAcceptedOutputAuthority,
        interest: &'a WorthQueryOutputDemandInterest,
    ) -> Self {
        Self::of(authority, interest)
    }
}
