//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryProductRuntime {
    #[doc(hidden)]
    pub fn begin_product_branch_close_for_test(
        &self,
        branch: WorthQueryProductBranch,
        scope: WorthQueryProductBranchCloseScope,
    ) -> Result<WorthQueryPendingProductBranchClose, WorthQueryProductBranchCloseDenial> {
        Self::begin_product_branch_close(self, branch, scope)
    }
}
