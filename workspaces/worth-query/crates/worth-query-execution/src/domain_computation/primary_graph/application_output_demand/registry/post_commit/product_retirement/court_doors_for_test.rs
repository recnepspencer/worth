//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryOutputDemandRegistry {
    #[doc(hidden)]
    pub fn release_product_occurrence_for_test(
        &self,
        occurrence: worth_runtime_world::facade::ProductBranchIncarnation,
    ) {
        Self::release_product_occurrence(self, occurrence)
    }
}
