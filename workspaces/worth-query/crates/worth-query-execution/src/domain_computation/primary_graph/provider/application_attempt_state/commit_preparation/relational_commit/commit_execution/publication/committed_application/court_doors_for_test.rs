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
    ) -> &crate::domain_computation::primary_graph::WorthQueryCommittedProductPublication {
        Self::committed_product_publication(self)
    }
}

impl WorthQueryPrimaryGraphCommittedApplication {
    /// Actual performed group members, never a receipt inferred from a shared commit.
    #[doc(hidden)]
    pub fn coupled_members_for_test(&self) -> impl Iterator<Item = &Self> {
        self.publication_members().skip(1)
    }
}
