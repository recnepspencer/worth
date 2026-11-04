//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryOutputDemandRegistry {
    #[doc(hidden)]
    pub fn rejoin_refreshed_for_test(
        &self,
        stale: &WorthQueryOutputDemandInterest,
    ) -> Result<
        Option<(
            WorthQueryOutputDemandInterest,
            Arc<RequiredOutputReadmission>,
        )>,
        WorthQueryOutputDemandDenial,
    > {
        Self::rejoin_refreshed(self, stale)
    }
}
