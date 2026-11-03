use super::{WorthQueryPrimaryGraphCommittedApplication, WorthQueryPrimaryGraphProvider};

impl WorthQueryPrimaryGraphProvider {
    #[cfg(test)]
    pub(in crate::domain_computation::primary_graph) fn discard_receipt_basis_window_for_test(
        &self,
    ) {
        *self
            .receipt_basis_retention
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = Default::default();
    }

    #[cfg(test)]
    pub(in crate::domain_computation::primary_graph) fn discard_completed_evidence_for_test(&self) {
        *self
            .completed_commit_evidence
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = Default::default();
    }

    pub(in crate::domain_computation::primary_graph) fn capture_correction_witness(
        &self,
        execution: worth_execution::ExecutionRequest<'_, '_>,
        commit: &worth_relational::facade::history::RelationalCommitReceipt,
    ) -> Result<
        Option<std::sync::Arc<super::CorrectionPerformedWitness>>,
        super::CorrectionWitnessDenial,
    > {
        self.completed_commit_evidence
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .capture_correction_witness(execution, commit)
    }

    pub(in crate::domain_computation::primary_graph) fn observe_completed_application(
        &self,
        commit: &worth_relational::facade::history::RelationalCommitReceipt,
    ) -> Option<WorthQueryPrimaryGraphCommittedApplication> {
        self.completed_commit_evidence
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .observe(commit)
    }

    #[cfg(test)]
    pub(in crate::domain_computation::primary_graph) fn retained_application_commit_basis(
        &self,
        commit: &worth_relational::facade::history::RelationalCommitReceipt,
    ) -> Option<super::WorthQueryRetainedApplicationCommitBasis> {
        self.completed_commit_evidence
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .observe(commit)?;
        self.receipt_basis_retention
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .acquire(commit.commit_id)
    }

    #[cfg(test)]
    pub(in crate::domain_computation::primary_graph) fn retained_receipt_basis_count(
        &self,
    ) -> usize {
        self.receipt_basis_retention
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .retained_count()
    }
}
