//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryProductRuntime {
    #[doc(hidden)]
    pub fn admit_product_branch_for_test(
        &self, identity: &worth_runtime_world::facade::ProductBranchIdentity) -> Result<WorthQueryProductBranchLease, WorthQueryProductBranchAdmissionDenial> {
        Self::admit_product_branch(self, identity)
    }
}
