//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryOutputDemandRegistry {
    #[doc(hidden)]
    pub fn finish_settlement_for_test(
        &self,
        interest: &WorthQueryOutputDemandInterest,
        accepted: &WorthQueryAcceptedOutputAuthority,
    ) -> Result<(), WorthQueryOutputDemandDenial> {
        Self::finish_settlement(self, interest, accepted)
    }
}

impl WorthQueryOutputDemandRegistry {
    #[doc(hidden)]
    pub fn peek_ready_for_test(
        &self,
        interest: &WorthQueryOutputDemandInterest,
    ) -> Result<Option<super::super::ReadyCompletion>, WorthQueryOutputDemandDenial> {
        Self::peek_ready(self, interest)
    }
}
