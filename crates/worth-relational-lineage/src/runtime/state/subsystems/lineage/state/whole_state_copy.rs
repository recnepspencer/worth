//! The lineage state's one whole-state copy, made only through named entry
//! points: a runtime fork and a reconstruction lane.

use super::LineageState;

impl LineageState {
    /// The one whole-state copy: a runtime fork's or a reconstruction
    /// lane's working copy, O(lineage state). `LineageState` is not `Clone`,
    /// so no ordinary commit, read or resolution path can copy it.
    fn duplicate_whole_state(&self) -> Self {
        let naming_states = self.branch_naming_states.iter();
        Self {
            nodes: self.nodes.clone(),
            relation_lineages: self.relation_lineages.clone(),
            relation_lineage_holders: self.relation_lineage_holders.clone(),
            branch_forks: self.branch_forks.clone(),
            branch_naming_states: naming_states
                .map(|(branch, state)| (branch.clone(), state.duplicate_for_branch_fork()))
                .collect(),
            name_indexes: self.name_indexes.duplicate_whole(),
            naming_rebuilt_event_frontier: self.naming_rebuilt_event_frontier,
            published_events: self.published_events.clone(),
            published_event_positions: self.published_event_positions.clone(),
            published_group_lineage: self.published_group_lineage.clone(),
            group_states: self.group_states.duplicate_whole(),
            resolution_indexes: self.resolution_indexes.clone(),
            identity_allocator: self.identity_allocator.detached(),
        }
    }

    /// A runtime fork's copy of the whole lineage state.
    pub(crate) fn duplicate_for_runtime_fork(&self) -> Self {
        self.duplicate_whole_state()
    }

    /// A reconstruction lane's working copy of the whole lineage state; the
    /// lane token keeps it off every ordinary open, commit and read path.
    pub fn duplicate_for_reconstruction(
        &self,
        _lane: &crate::contracts::storage::substrate::maintenance::reconstruction::ReconstructionLane<'_>,
    ) -> Self {
        self.duplicate_whole_state()
    }
}
