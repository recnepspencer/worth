//! Named reconstruction-lane entry point and detached COW fork for
//! `LineageState`. Naming and group tables are shared copy-on-write via
//! [`LineageState::fork_detached`].

use super::super::recovery_token::{next_state_id, FullLineageRecoveryToken};
use super::LineageState;

impl LineageState {
    /// An independent state for a forked runtime (`O(resident)` COW fork).
    pub fn fork_detached(&self) -> Self {
        let state_id = next_state_id();
        Self {
            state_id,
            nodes: self.nodes.clone(),
            relation_lineages: self.relation_lineages.clone(),
            relation_lineage_holders: self.relation_lineage_holders.clone(),
            branch_forks: self.branch_forks.clone(),
            fork_children: self.fork_children.clone(),
            name_indexes: self.name_indexes.fork_all(),
            naming_rebuilt_event_frontier: self.naming_rebuilt_event_frontier,
            published_events: self.published_events.clone(),
            published_event_positions: self.published_event_positions.clone(),
            event_commits_by_branch: self.event_commits_by_branch.clone(),
            published_group_lineage: self.published_group_lineage.clone(),
            group_states: self.group_states.fork_all(),
            cold_naming: self.cold_naming.clone(),
            branch_generations: self.branch_generations.clone(),
            next_generation: self.next_generation,
            full_history: self
                .full_history
                .as_ref()
                .map(|_| FullLineageRecoveryToken::fresh(state_id)),
            resolution_indexes: self.resolution_indexes.clone(),
            identity_allocator: self.identity_allocator.detached(),
        }
    }

    /// A reconstruction lane's working copy of the lineage state; the lane
    /// token keeps it off every ordinary open, commit and read path, and
    /// naming/group tables share copy-on-write (`O(resident)`).
    pub fn duplicate_for_reconstruction(
        &self,
        _lane: &crate::contracts::storage::substrate::maintenance::reconstruction::ReconstructionLane<'_>,
    ) -> Self {
        self.fork_detached()
    }
}
