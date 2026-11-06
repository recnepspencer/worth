//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryPrimaryGraphProvider {
    #[doc(hidden)]
    pub fn observe_completed_application_for_test(
        &self,
        commit: &worth_relational::facade::history::RelationalCommitReceipt,
    ) -> Option<WorthQueryPrimaryGraphCommittedApplication> {
        Self::observe_completed_application(self, commit)
    }
}

impl WorthQueryPrimaryGraphProvider {
    #[doc(hidden)]
    pub fn capture_correction_witness_for_test(
        &self,
        execution: worth_execution::ExecutionRequest<'_, '_>,
        commit: &worth_relational::facade::history::RelationalCommitReceipt,
    ) -> Result<
        Option<std::sync::Arc<super::super::CorrectionPerformedWitness>>,
        super::super::CorrectionWitnessDenial,
    > {
        Self::capture_correction_witness(self, execution, commit)
    }
}

impl WorthQueryPrimaryGraphProvider {
    #[doc(hidden)]
    pub fn retained_application_commit_basis_for_test(
        &self,
        commit: &worth_relational::facade::history::RelationalCommitReceipt,
    ) -> Option<super::super::session_commit::WorthQueryRetainedApplicationCommitBasis> {
        Self::retained_application_commit_basis(self, commit)
    }
}
