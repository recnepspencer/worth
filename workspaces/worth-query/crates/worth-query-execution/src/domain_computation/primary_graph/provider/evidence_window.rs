//! The completed-evidence capacity as the declared idempotency window.

use worth_relational::facade::history::CommitId;

use super::completed_evidence_capacity::CompletedEvidenceTicket;
use super::{WorthQueryPrimaryGraphCommittedApplication, WorthQueryPrimaryGraphProvider};

impl WorthQueryPrimaryGraphProvider {
    /// Admits `bytes` of new completed evidence, evicting the oldest evidence
    /// the window no longer has room for. Only evidence whose eviction frees
    /// its bytes is evicted; admission is refused once every retained entry
    /// holds a mandatory dispatch basis or is still held by a caller.
    pub(super) fn reserve_completed_evidence(
        &self,
        bytes: usize,
    ) -> Option<CompletedEvidenceTicket> {
        if !self.completed_evidence_capacity.admits(bytes) {
            return None;
        }
        loop {
            if let Some(ticket) = self.completed_evidence_capacity.reserve(bytes) {
                return Some(ticket);
            }
            // Read outside the evidence lock: a mandatory basis is retained
            // before its commit's evidence is recorded, so no entry can turn
            // mandatory between this read and the eviction below.
            let mandatory = self
                .receipt_basis_retention
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .mandatory_commits();
            let (commit, evicted) = self
                .completed_commit_evidence
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .evict_oldest(&mandatory)?;
            self.receipt_basis_retention
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .release_ordinary(commit);
            // The window held the last copy, so dropping it refunds its ticket.
            drop(evicted);
        }
    }

    /// Whether `commit`'s evidence left the idempotency window.
    pub(in crate::domain_computation::primary_graph) fn completed_evidence_expired(
        &self,
        commit: CommitId,
    ) -> bool {
        self.completed_commit_evidence
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .expired(commit)
    }

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

    /// Completed evidence entries and the bytes their tickets hold.
    #[cfg(feature = "test-query-execution-observer")]
    pub(in crate::domain_computation::primary_graph) fn completed_evidence_retained(
        &self,
    ) -> (usize, usize) {
        (
            self.completed_commit_evidence
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .entry_count(),
            self.completed_evidence_capacity.retained_bytes(),
        )
    }
}
