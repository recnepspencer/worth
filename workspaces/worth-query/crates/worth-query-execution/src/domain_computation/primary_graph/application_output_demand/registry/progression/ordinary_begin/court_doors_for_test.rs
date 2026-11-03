//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryOutputDemandRegistry {
    #[doc(hidden)]
    pub fn begin_for_test(
        &self, interest: &WorthQueryOutputDemandInterest) -> WorthQueryOutputDemandAdvanceAdmission {
        Self::begin(self, interest)
    }
}
