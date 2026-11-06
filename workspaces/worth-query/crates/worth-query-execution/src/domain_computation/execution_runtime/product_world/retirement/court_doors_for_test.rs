//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryProductRuntime {
    #[doc(hidden)]
    pub fn retire_product_branch_for_test(
        &self,
        observed: &WorthQueryProductBranchLease,
    ) -> Result<
        ProductBranchRetirementReport,
        RuntimeWorldServiceDenial<RuntimeWorldBranchRetirementDenial>,
    > {
        Self::retire_product_branch(self, observed)
    }
}
