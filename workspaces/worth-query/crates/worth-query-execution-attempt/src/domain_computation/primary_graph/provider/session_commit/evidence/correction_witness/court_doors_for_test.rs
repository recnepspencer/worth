//! Sealed test doors over the session store's completed commits.
use super::*;

impl WorthQueryCompletedCommitEvidenceStore {
    /// Every receipt this session completed, in commit order.
    #[doc(hidden)]
    pub fn completed_receipts_for_test(&self) -> Vec<RelationalCommitReceipt> {
        self.by_commit
            .values()
            .map(|completed| completed.commit_reference().clone())
            .collect()
    }
}
