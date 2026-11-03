//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryPreparedProductPublication {
    #[doc(hidden)]
    pub fn execute_for_test(
        self, execution: worth_execution::ExecutionRequest<'_, '_>) -> Result<Outcome, Denial<NoEffectCompositePublication>> {
        Self::execute(self, execution)
    }
}

impl WorthQueryProductPublicationBinding {
    #[doc(hidden)]
    pub fn prepare_relational_candidate_for_test(
        &self, candidate: worth_relational::facade::mvcc::PreparedRelationalCommitCandidate, request: &WorthQueryRequestScope, successor_observation_requested: bool) -> Result<WorthQueryPreparedProductPublication, Denial<NoEffectCompositePublication>> {
        Self::prepare_relational_candidate(self, candidate, request, successor_observation_requested)
    }
}
