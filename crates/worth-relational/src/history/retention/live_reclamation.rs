use super::RelationalBranchRootReclamationOutcome;
use crate::storage::substrate::StorageAllocationOverflow;

impl crate::runtime::RelationalRuntime {
    /// Explicit cold index maintenance. An in-flight cutover or unsettled
    /// candidate leaves the catalog untouched for a later pass.
    pub fn run_index_generation_reclamation_pass(&self) -> Option<usize> {
        self.history
            .canonical_checkpoint_gate()
            .with_settled_publication_snapshot(|| {
                // The owner inventory, rather than Arc payload counts, decides
                // which historical versions still have live obligations.
                self.history
                    .reclaim_retired_branch_roots(self.config.storage.retention.reclaim_batch_size);
                let retained = self.history.retained_index_roots();
                self.indexes.reclaim_except_versions(&retained)
            })
    }

    /// Reclaim cold immutable branch roots through the history owner. This
    /// bounded maintenance pass never walks live references or commit history.
    pub fn run_branch_root_reclamation_pass(&self) -> RelationalBranchRootReclamationOutcome {
        self.history
            .reclaim_retired_branch_roots(self.config.storage.retention.reclaim_batch_size)
    }

    /// Reclaim cold immutable branch roots, refusing any unrepresentable
    /// reclamation counter or byte total with a typed overflow instead of saturating.
    pub fn try_run_branch_root_reclamation_pass(
        &self,
    ) -> Result<RelationalBranchRootReclamationOutcome, StorageAllocationOverflow> {
        self.history
            .try_reclaim_retired_branch_roots(self.config.storage.retention.reclaim_batch_size)
    }

    /// Retired branch roots awaiting reclamation now.
    pub fn retired_branch_root_count(&self) -> usize {
        self.history.retired_branch_root_count()
    }
}
