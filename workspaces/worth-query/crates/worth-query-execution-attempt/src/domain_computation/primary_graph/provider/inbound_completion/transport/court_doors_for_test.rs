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
        let evidence =
            InstalledTransportCompletion::from_observed_dispatch(owner.clone(), dispatch)
                .map_err(|_| Denial::DispatchOwnerMismatch)?;
        let proposal = self.validate_installed_transport_completion_proposal(
            execution, basis, &evidence, binding,
        )?;
        self.prepare_completion_proposal(execution, proposal)
    }
}
