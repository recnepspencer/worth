//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryApplicationBasisRegistry {
    #[doc(hidden)]
    pub fn register_for_test(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        basis: AdmittedRelationalBranchBasis,
        graph: WorthQueryPrimaryGraphIntegrationHandle,
    ) -> Result<WorthQueryApplicationBasisLease, WorthQueryApplicationBasisRegistrationDenial> {
        Self::register(self, execution, basis, graph)
    }
}
