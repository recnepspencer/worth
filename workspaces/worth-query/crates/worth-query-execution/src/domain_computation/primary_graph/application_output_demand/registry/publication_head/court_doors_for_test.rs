//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryOutputDemandRegistry {
    #[doc(hidden)]
    pub fn publication_head_selection_for_test(
        &self,
        occurrence: ProductBranchIncarnation,
    ) -> Result<PublicationHeadSelection, WorthQueryOutputDemandDenial> {
        Self::publication_head_selection(self, occurrence)
    }
}
