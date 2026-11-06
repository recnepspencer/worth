//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryOutputDemandInterest {
    #[doc(hidden)]
    pub fn required_context_for_test(
        &self,
        family: &'static str,
        source_owner: &SourceInvalidationOwner,
    ) -> Result<RequiredOutputDemandContext, WorthQueryOutputDemandDenial> {
        Self::required_context(self, family, source_owner)
    }
}
