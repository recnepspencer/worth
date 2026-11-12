//! Completed evidence and exact World history handoff.

use super::WorthQueryPrimaryGraphProvider;

impl WorthQueryPrimaryGraphProvider {
    /// Exact completed evidence entries retained for idempotency.
    pub(in crate::domain_computation::primary_graph) fn completed_evidence_retained(
        &self,
    ) -> usize {
        self.completed_commit_evidence
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .entry_count()
    }
}
