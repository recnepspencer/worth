//! Completed evidence and exact World history handoff.

use super::{WorthQueryPrimaryGraphCommittedApplication, WorthQueryPrimaryGraphProvider};

impl WorthQueryPrimaryGraphProvider {
    /// Hands the World history hold of `receipt`'s commit to its own caller.
    pub(in crate::domain_computation::primary_graph) fn claim_fresh_history(
        &self,
        receipt: &mut WorthQueryPrimaryGraphCommittedApplication,
    ) {
        self.completed_commit_evidence
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .claim_fresh_history(receipt);
    }

    /// Exact completed evidence entries retained for idempotency.
    #[cfg(feature = "test-query-execution-observer")]
    pub(in crate::domain_computation::primary_graph) fn completed_evidence_retained(
        &self,
    ) -> usize {
        self.completed_commit_evidence
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .entry_count()
    }
}
