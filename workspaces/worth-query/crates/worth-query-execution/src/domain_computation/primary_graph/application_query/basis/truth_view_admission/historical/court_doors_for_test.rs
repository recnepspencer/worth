//! Sealed test doors over this module's private members.
use super::*;

impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema>
where
    Schema: ApplicationSchema,
{
    #[doc(hidden)]
    pub fn admit_application_historical_basis_for_test(
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
