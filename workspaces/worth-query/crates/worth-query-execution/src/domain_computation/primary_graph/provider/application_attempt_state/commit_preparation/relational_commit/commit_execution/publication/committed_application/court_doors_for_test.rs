//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryPrimaryGraphCommittedApplication {
    #[doc(hidden)]
    pub fn commit_reference_for_test(&self) -> &RelationalCommitReceipt {
        Self::commit_reference(self)
    }
}

impl WorthQueryPrimaryGraphCommittedApplication {
    #[doc(hidden)]
    pub fn committed_product_publication_for_test(
        &self,
    ) -> &crate::domain_computation::primary_graph::vocabulary::application_attempt::committed_publication::WorthQueryCommittedProductPublication {
        Self::committed_product_publication(self)
    }
}
