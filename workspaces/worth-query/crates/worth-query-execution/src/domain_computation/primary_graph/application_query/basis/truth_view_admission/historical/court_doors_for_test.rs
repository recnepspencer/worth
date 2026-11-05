//! Sealed test doors over this module's private members.
use super::*;

pub trait HistoricalDoorsOps<Schema>:
    crate::domain_computation::primary_graph::application_query::host::ApplicationQueryHost<Schema>
where
    Schema: ApplicationSchema,
{
    #[doc(hidden)]
    fn admit_application_historical_basis_for_test(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        read: WorthQueryApplicationHistoricalRead,
        request: &WorthQueryRequestScope,
    ) -> Result<
        WorthQueryApplicationHistoricalBasis<Schema>,
        WorthQueryApplicationQueryAdmissionDenial,
    > {
        Self::admit_application_historical_basis(self, execution, read, request)
    }
}

impl<
        Schema,
        HostRuntime: crate::domain_computation::primary_graph::application_query::host::ApplicationQueryHost<
                Schema,
            > + ?Sized,
    > HistoricalDoorsOps<Schema> for HostRuntime
where
    Schema: ApplicationSchema,
{
}
