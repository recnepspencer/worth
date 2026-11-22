//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryOutputDemandRegistry {
    #[doc(hidden)]
    pub fn complete_prepared_source_for_test(
        &self,
        receipt: &crate::domain_computation::primary_graph::WorthQueryApplicationCommitReceipt,
    ) {
        Self::complete_prepared_source(self, &receipt.into())
    }
}

impl WorthQueryOutputDemandRegistry {
    #[doc(hidden)]
    pub fn begin_source_preparation_for_test(
        &self,
        occurrence: worth_runtime_world::facade::ProductBranchIncarnation,
    ) -> WorthQueryRequiredOutputSourcePreparation {
        Self::begin_source_preparation(self, occurrence)
    }
}

impl WorthQueryOutputDemandRegistry {
    #[doc(hidden)]
    pub fn discard_prepared_source_for_test(
        &self,
        source_commit: &worth_runtime_world::facade::CompositeCommitIdentity,
    ) {
        Self::discard_prepared_source(self, source_commit)
    }
}
