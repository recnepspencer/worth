//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryOutputDemandRegistry {
    #[doc(hidden)]
    pub fn finish_replaced_interest_for_test(
        &self,
        replaced: &WorthQueryOutputDemandInterest,
        replacement: &WorthQueryOutputDemandInterest,
        subject: &str,
    ) -> Result<(), WorthQueryOutputDemandDenial> {
        Self::finish_replaced_interest(self, replaced, replacement, subject)
    }
}
