//! Crate-boundary internal port for `WorthQueryProductEntry` internal selection
//! paths. Never exported on `worth-query-execution::facade` or Host.

use worth_query_installation::facade::ApplicationSchema;

use super::context::{WorthQueryProductEntry, WorthQuerySelectedProductOperation};
use crate::basis::WorthQueryProductBranchAdmissionDenial;

pub trait ProductEntryInternalPort<'runtime, Schema: ApplicationSchema>: Sized {
    fn select_for_managed_index<Stop: From<WorthQueryProductBranchAdmissionDenial>>(
        self,
        request: &worth_execution::ExecutionRequest<'_, '_>,
        before_basis: impl FnOnce(
            &worth_runtime_world::facade::ProductBranchObservation,
        ) -> Result<(), Stop>,
    ) -> Result<WorthQuerySelectedProductOperation<'runtime, Schema>, Stop>;

    fn select_with_request(
        self,
        request: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<
        WorthQuerySelectedProductOperation<'runtime, Schema>,
        WorthQueryProductBranchAdmissionDenial,
    >;
}

impl<'runtime, Schema: ApplicationSchema> ProductEntryInternalPort<'runtime, Schema>
    for WorthQueryProductEntry<'runtime, Schema>
{
    fn select_for_managed_index<Stop: From<WorthQueryProductBranchAdmissionDenial>>(
        self,
        request: &worth_execution::ExecutionRequest<'_, '_>,
        before_basis: impl FnOnce(
            &worth_runtime_world::facade::ProductBranchObservation,
        ) -> Result<(), Stop>,
    ) -> Result<WorthQuerySelectedProductOperation<'runtime, Schema>, Stop> {
        Self::select_for_managed_index(self, request, before_basis)
    }

    fn select_with_request(
        self,
        request: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<
        WorthQuerySelectedProductOperation<'runtime, Schema>,
        WorthQueryProductBranchAdmissionDenial,
    > {
        Self::select_with_request(self, request)
    }
}
