//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryPrimaryGraphIntegrationHandle {
    #[doc(hidden)]
    pub fn ensure_primary_indexes_for_basis_for_test(
        &self, execution: &worth_execution::ExecutionRequest<'_, '_>, runtime: &mut RelationalRuntime, basis: &AdmittedRelationalBranchBasis) -> Result<(), WorthQueryPrimaryIndexCurrencyDenial> {
        Self::ensure_primary_indexes_for_basis(self, execution, runtime, basis)
    }
}
