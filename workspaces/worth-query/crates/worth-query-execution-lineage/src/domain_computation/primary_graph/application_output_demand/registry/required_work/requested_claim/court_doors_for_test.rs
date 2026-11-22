//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryOutputDemandRegistry {
    #[doc(hidden)]
    pub fn claim_cached_requested_ready_for_test(
        &self,
        key: &WorthQueryOutputDemandKey,
    ) -> Result<
        Option<(SelectedReadyReadmission, RequestedOutputReadClaim)>,
        WorthQueryOutputDemandDenial,
    > {
        Self::claim_cached_requested_ready(self, key)
    }
}
