//! Sealed test doors over this module's private members.
use super::*;

impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    #[doc(hidden)]
    pub fn select_product_branch_for_test(
        &self, execution: &worth_execution::ExecutionRequest<'_, '_>, identity: &ProductBranchIdentity) -> Result<
        WorthQuerySelectedProductOperation<'_, Schema>,
        WorthQueryProductBranchAdmissionDenial,
    > {
        Self::select_product_branch(self, execution, identity)
    }
}
