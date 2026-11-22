//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryOutputDemandRegistry {
    #[doc(hidden)]
    pub fn next_required_claim_for_selected_for_test(
        &self,
        product: &crate::basis::WorthQueryProductObservationLease,
        selected: &worth_relational::facade::runtime::PositionedRelationalSnapshot,
        source_owner: &SourceInvalidationOwner,
    ) -> Result<Option<RequiredWorkClaim>, WorthQueryOutputDemandDenial> {
        Self::next_required_claim_for_selected(self, product, selected, source_owner)
    }
}

impl WorthQueryOutputDemandRegistry {
    #[doc(hidden)]
    pub fn select_required_claim_for_test(
        &self,
        claim: &mut RequiredWorkClaim,
        selected: &worth_relational::facade::runtime::PositionedRelationalSnapshot,
    ) -> Result<Option<SelectedRequiredWork>, WorthQueryOutputDemandDenial> {
        Self::select_required_claim(self, claim, selected)
    }
}
