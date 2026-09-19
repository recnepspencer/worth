use crate::data::graph::signal_graph::SignalGraph;
#[cfg(any(test, doctest))]
use crate::diagnostics::replay::ReplayEvent;
use crate::diagnostics::replay::ReplaySlice;
use crate::state::SignalBranchHandle;

impl SignalGraph {
    #[cfg(any(test, doctest))]
    pub(crate) fn replay_events(
        &self,
    ) -> &crate::diagnostics::state::DiagnosticHistory<ReplayEvent> {
        self.observation.diagnostics.replay_events()
    }

    pub(crate) fn replay_for_branch(&self, branch_id: crate::state::SignalBranchId) -> ReplaySlice {
        ReplaySlice {
            start: None,
            end: None,
            frames: self
                .diagnostics_state()
                .replay_events_for_branch(branch_id)
                .map(|frames| frames.iter().cloned().collect())
                .unwrap_or_default(),
        }
    }

    pub(crate) fn current_branch(&self) -> SignalBranchHandle {
        self.observation.diagnostics.active_branch()
    }

    #[cfg(test)]
    pub(crate) fn known_branches(&self) -> Vec<SignalBranchHandle> {
        self.observation
            .diagnostics
            .branch_catalog()
            .values()
            .cloned()
            .collect()
    }

    pub(crate) fn branch_handle(
        &self,
        branch_id: crate::state::SignalBranchId,
    ) -> Option<SignalBranchHandle> {
        self.observation
            .diagnostics
            .branch_catalog()
            .get(&branch_id)
            .cloned()
    }

    pub(crate) fn branch_head_snapshot_id(
        &self,
        branch_id: crate::state::SignalBranchId,
    ) -> Option<crate::state::SignalSnapshotId> {
        self.branch_handle(branch_id)
            .and_then(|branch| branch.head_snapshot_id)
    }
}
