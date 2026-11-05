//! Sealed test doors over this module's private members.
use super::*;

impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    #[doc(hidden)]
    pub fn select_product_branch_for_test(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        identity: &ProductBranchIdentity,
    ) -> Result<
        WorthQuerySelectedProductOperation<'_, Schema>,
        WorthQueryProductBranchAdmissionDenial,
    > {
        Self::select_product_branch(self, execution, identity)
    }
}

impl<'runtime, Schema> WorthQuerySelectedProductOperation<'runtime, Schema> {
    #[doc(hidden)]
    pub fn into_parts_for_test(
        self,
    ) -> (
        &'runtime WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        WorthQueryProductBranchLease,
        super::super::super::application_query::resource_lifecycle::WorthQueryApplicationBasisLease,
    ) {
        Self::into_parts(self)
    }
}

impl<'runtime, Schema> WorthQuerySelectedProductOperation<'runtime, Schema> {
    #[doc(hidden)]
    pub fn application_basis_for_test(
        &self,
    ) -> &super::super::super::application_query::resource_lifecycle::WorthQueryApplicationBasisLease
    {
        Self::application_basis(self)
    }
}

impl<'runtime, Schema> WorthQuerySelectedProductOperation<'runtime, Schema> {
    #[doc(hidden)]
    pub fn original_settled_producer_for_test(
        &self,
    ) -> Option<
        &std::sync::Arc<
            super::super::super::application_output_demand::OriginalSettledProducerObservation,
        >,
    > {
        Self::original_settled_producer(self)
    }
}

impl<'runtime, Schema: ApplicationSchema> WorthQuerySelectedProductOperation<'runtime, Schema> {
    #[doc(hidden)]
    pub fn retain_selection_for_test(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<Self, WorthQueryProductBranchAdmissionDenial> {
        Self::retain_selection(self, execution)
    }
}

impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    #[doc(hidden)]
    pub fn on_product_for_test(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        product: WorthQueryProductBranchLease,
    ) -> Result<
        WorthQuerySelectedProductOperation<'_, Schema>,
        WorthQueryProductBranchAdmissionDenial,
    > {
        Self::on_product(self, execution, product)
    }
}

impl<'runtime, Schema> WorthQueryProductEntry<'runtime, Schema>
where
    Schema: ApplicationSchema,
{
    #[doc(hidden)]
    pub fn select_with_request_for_test(
        self,
        request: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<
        WorthQuerySelectedProductOperation<'runtime, Schema>,
        WorthQueryProductBranchAdmissionDenial,
    > {
        Self::select_with_request(self, request)
    }
}
