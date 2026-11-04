//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryOutputDemandRegistry {
    #[doc(hidden)]
    pub fn interest_ready_readmission_for_test(
        &self,
        interest: &WorthQueryOutputDemandInterest,
    ) -> Result<Option<SelectedReadyReadmission>, WorthQueryOutputDemandDenial> {
        Self::interest_ready_readmission(self, interest)
    }
}
