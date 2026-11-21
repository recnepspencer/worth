//! Sealed test doors over this module's private members.
use super::*;

impl<'runtime, Schema> WorthQuerySelectedProductOperation<'runtime, Schema> {
    #[doc(hidden)]
    pub fn prepare_shared_query_basis_for_test(
        self,
    ) -> Result<SharedSelectedProductOperation<'runtime, Schema>, (Self, CompanionPreflightStop)>
    {
        Self::prepare_shared_query_basis(self)
    }
}

impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    #[doc(hidden)]
    pub fn retain_selected_query_basis_admitted_for_test(
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
