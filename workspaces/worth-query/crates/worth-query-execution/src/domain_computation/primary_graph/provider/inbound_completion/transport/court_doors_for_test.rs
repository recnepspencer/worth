//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryPrimaryGraphProvider {
    #[doc(hidden)]
    pub fn prepare_installed_transport_completion_candidate_for_test(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        basis: &AdmittedRelationalBranchBasis,
        owner: &WorthQueryCommittedDispatchOutboxObservation,
        dispatch: &WorthQueryExternalEffectDispatch,
        binding: &WorthQueryInstalledTransportCompletionBinding,
    ) -> Result<PreparedRelationalCommitCandidate, Denial> {
        Self::prepare_installed_transport_completion_candidate(
            self, execution, basis, owner, dispatch, binding,
        )
    }
}
