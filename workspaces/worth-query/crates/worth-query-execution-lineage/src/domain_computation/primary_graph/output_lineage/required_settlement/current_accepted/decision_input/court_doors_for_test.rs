//! Sealed test doors over this module's private members.
use super::*;

impl AcceptedCurrentCandidate {
    #[doc(hidden)]
    pub fn decision_input_changed_at_for_test(
        &self,
        request_execution: &worth_execution::ExecutionRequest<'_, '_>,
        owner: &SourceInvalidationOwner,
        runtime: &RelationalRuntime,
        basis: CertificationBasis<'_>,
        selected: &PositionedRelationalSnapshot,
    ) -> Result<bool, CurrentAcceptedStop> {
        Self::decision_input_changed_at(self, request_execution, owner, runtime, basis, selected)

    }
}
