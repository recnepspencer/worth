//! Physical candidate capture reserves no canonical history position.
use super::*;
impl RelationalPreparationHistory {
    pub(crate) fn capture_applied_candidate_root(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        previous: &Arc<RelationalBranchRoot>,
        state: &crate::storage::overlay::WorkingState,
        symbols: &crate::runtime::RuntimeSymbolTable,
    ) -> Result<Arc<RelationalBranchRoot>, RelationalBranchRootCaptureDenial> {
        let committed = state.clone().into_partition_commits().1;
        let delta = crate::storage::RelationalPublishedPartitionDelta::from_committed_partitions(
            &committed,
        );
        RelationalBranchRoot::capture_applied_candidate(
            execution,
            &self.root_ids,
            previous,
            &delta,
            symbols,
            state.content.directory()?,
        )
    }
}
