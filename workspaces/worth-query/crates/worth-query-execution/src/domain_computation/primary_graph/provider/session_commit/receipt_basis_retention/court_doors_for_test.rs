//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryReceiptBasisRetentionStore {
    #[doc(hidden)]
    pub fn retain_group_for_test(
        &mut self,
        commit: CommitId,
        lease: RelationalBranchRetentionLease,
        dispatch_claims: usize,
    ) {
        Self::retain_group(self, commit, lease, dispatch_claims)
    }
}
