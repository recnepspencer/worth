//! Sealed test doors over this module's private members.
use super::*;
use crate::domain_computation::primary_graph::vocabulary::product_operation::shared_query_basis::SelectedQueryBasisRetentionStop;

impl<'runtime, Schema> WorthQuerySelectedProductOperation<'runtime, Schema> {
    #[doc(hidden)]
    pub fn prepare_shared_query_basis_for_test(
        self,
    ) -> Result<SharedSelectedProductOperation<'runtime, Schema>, (Self, CompanionPreflightStop)>
    {
        Self::prepare_shared_query_basis(self)
    }
}

pub trait SharedQueryBasisDoorsOps<Schema>:
    crate::domain_computation::primary_graph::product_operation::host::ProductOperationHost<Schema>
{
    #[doc(hidden)]
    fn retain_selected_query_basis_admitted_for_test(
        &self,
        shared: &SharedSelectedProductOperation<'_, Schema>,
    ) -> Result<
        (
            WorthQueryProductObservationLease,
            WorthQueryApplicationBasisLease,
        ),
        SelectedQueryBasisRetentionStop,
    > {
        Self::retain_selected_query_basis_admitted(self, shared)
    }
}

impl<
        Schema,
        HostRuntime: crate::domain_computation::primary_graph::product_operation::host::ProductOperationHost<
                Schema,
            > + ?Sized,
    > SharedQueryBasisDoorsOps<Schema> for HostRuntime
{
}
