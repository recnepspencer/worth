//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryPrimaryGraphIntegrationHandle {
    #[doc(hidden)]
    pub fn carry_primary_index_basis_for_basis_for_test(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        runtime: &mut RelationalRuntime,
        basis: &AdmittedRelationalBranchBasis,
    ) -> Result<(), WorthQueryPrimaryIndexCurrencyDenial> {
        Self::carry_primary_index_basis_for_basis(self, execution, runtime, basis)
    }
}
