use super::{
    WorthQueryPrimaryGraphCommittedApplication, WorthQueryPrimaryGraphProvider,
    WorthQueryProductIdempotencyAffinity, WorthQueryProviderIdempotencyResolution,
};

impl WorthQueryPrimaryGraphProvider {
    pub(in crate::domain_computation::primary_graph) fn observe_completed_application(
        &self,
        commit: &worth_relational::facade::history::RelationalCommitReceipt,
    ) -> Option<WorthQueryPrimaryGraphCommittedApplication> {
        self.completed_commit_evidence
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .observe(commit)
    }

    pub(in crate::domain_computation::primary_graph) fn observe_completed_application_for_session(
        &self,
        session: &crate::domain_computation::provider_session::WorthQueryProviderSessionTerminalBinding,
    ) -> Option<WorthQueryPrimaryGraphCommittedApplication> {
        self.completed_commit_evidence
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .observe_session(session)
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

    pub(in crate::domain_computation::primary_graph) fn resolve_completed_application_idempotency(
        &self,
        product: &WorthQueryProductIdempotencyAffinity,
        binding: crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationIdempotencyBinding,
    ) -> Option<WorthQueryProviderIdempotencyResolution> {
        let (committed_binding, committed) = self
            .completed_commit_evidence
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .observe_idempotency(product, binding)?;
        Some(if committed_binding == binding {
            WorthQueryProviderIdempotencyResolution::Equivalent(committed)
        } else {
            WorthQueryProviderIdempotencyResolution::Drift
        })
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
