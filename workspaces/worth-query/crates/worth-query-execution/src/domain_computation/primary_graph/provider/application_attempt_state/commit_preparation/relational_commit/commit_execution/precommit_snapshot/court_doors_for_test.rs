//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryPrecommitSnapshot {
    #[doc(hidden)]
    pub fn acquire_for_test(
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        graph: WorthQueryPrimaryGraphIntegrationHandle,
        basis: &AdmittedRelationalBranchBasis,
    ) -> Result<Self, RelationalSnapshotAdmissionDenial> {
        Self::acquire(execution, graph, basis)
    }
}
