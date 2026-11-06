//! Sealed test doors over this module's private members.
use super::*;

impl AcceptedCurrentCandidate {
    #[doc(hidden)]
    pub fn certify_current_for_test<'basis>(
        &'basis self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        owner: &SourceInvalidationOwner,
        runtime: &RelationalRuntime,
        product: &'basis WorthQueryProductObservationLease,
        snapshot: &'basis SnapshotHandle,
        selected: &'basis PositionedRelationalSnapshot,
    ) -> Result<CurrentAcceptedResult<'basis>, CurrentAcceptedStop> {
        Self::certify_current(self, execution, owner, runtime, product, snapshot, selected)
    }
}
