//! Physical candidate capture reserves no canonical history position.
use super::*;
impl RelationalPreparationHistory {
    pub(crate) fn capture_applied_candidate_root(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        previous: &Arc<RelationalBranchRoot>,
        state: &worth_relational_storage::internal_port::storage::overlay::working_state::WorkingState,
        content: &worth_relational_branch::internal_port::content::publication::ContentPublication,
        symbols: &worth_relational_storage::internal_port::runtime::state::subsystems::services::symbol_table::RuntimeSymbolTable,
    ) -> Result<Arc<RelationalBranchRoot>, RelationalBranchRootCaptureDenial> {
        let committed = state.clone().into_partition_commits().1;
        let delta = worth_relational_storage::internal_port::storage::overlay::published_delta::RelationalPublishedPartitionDelta::from_committed_partitions(
            &committed,
        );
        RelationalBranchRoot::capture_applied_candidate(
            execution,
            &self.root_ids,
            previous,
            &delta,
            symbols,
            content.directory()?,
        )
    }
}
