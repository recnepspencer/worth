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

impl<'runtime, Schema> WorthQuerySelectedProductOperation<'runtime, Schema> {
    #[doc(hidden)]
    pub fn into_parts_for_test(
        self) -> (
        &'runtime WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        WorthQueryProductBranchLease,
        super::super::super::application_query::resource_lifecycle::WorthQueryApplicationBasisLease,
    ) {
        Self::into_parts(self)
    }
}
