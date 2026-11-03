//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryPrimaryGraphProvider {
    #[doc(hidden)]
    pub fn graph_for_test(&self) -> &WorthQueryPrimaryGraphIntegrationHandle {
        &self.graph
    }
}

impl WorthQueryPrimaryGraphProvider {
    #[doc(hidden)]
    pub fn receipt_basis_retention_for_test(&self) -> &Arc<Mutex<session_commit::WorthQueryReceiptBasisRetentionStore>> {
        &self.receipt_basis_retention
    }
}
