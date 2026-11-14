//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryOutputDemandInterest {
    #[doc(hidden)]
    pub fn required_context_for_test(
        &self,
        family: &'static str,
        prerequisite_admission: Result<(), crate::domain_computation::execution_runtime::product_world::WorthQueryDurableStateAdmissionDenial>,
        source_owner: &SourceInvalidationOwner,
    ) -> Result<RequiredOutputDemandContext, WorthQueryOutputDemandDenial> {
        Self::required_context(self, family, prerequisite_admission, source_owner)
    }
}
