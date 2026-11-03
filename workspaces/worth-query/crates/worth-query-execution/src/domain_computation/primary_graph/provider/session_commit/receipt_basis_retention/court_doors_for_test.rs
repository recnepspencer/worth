//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryReceiptBasisRetentionStore {
    #[doc(hidden)]
    pub fn retain_for_test(
        &mut self, commit: CommitId, lease: RelationalBranchRetentionLease, mandatory: bool) {
        Self::retain(self, commit, lease, mandatory)
    }
}
