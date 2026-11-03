//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryExecutionRuntime {
    #[doc(hidden)]
    pub fn admit_direct_run_for_test(
        &self, operation: &WorthQueryExecutionBoundOperationAuthority, resource_attempt: WorthQueryDirectExecutionResourceAttempt, bridge_basis: BridgeBoundExecutionBasis, relational_basis: WorthQueryManagedRelationalObservation) -> Result<WorthQueryAdmittedDirectRun, WorthQueryManagedRunAdmissionRejection> {
        Self::admit_direct_run(self, operation, resource_attempt, bridge_basis, relational_basis)
    }
}
