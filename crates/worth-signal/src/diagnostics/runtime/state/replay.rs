use super::DiagnosticHistory;

use crate::data::handle::NodeId;
use crate::diagnostics::lineage::LineageArtifactId;
use crate::diagnostics::replay::{ReplayCursor, ReplayEvent};
use crate::state::{SignalBranchId, SignalSnapshotId};

use super::DiagnosticsState;

impl DiagnosticsState {
    pub fn replay_events(&self) -> &DiagnosticHistory<ReplayEvent> {
        &self.replay_events
    }

    pub fn replay_events_for_branch(
        &self,
        branch_id: SignalBranchId,
    ) -> Option<&DiagnosticHistory<ReplayEvent>> {
        self.replay_events_by_branch.get(&branch_id)
    }

    pub fn replay_events_for_node(&self, node: NodeId) -> Option<&DiagnosticHistory<ReplayEvent>> {
        self.replay_events_by_node.get(&node)
    }

    pub fn replay_events_for_artifact(
        &self,
        artifact_id: LineageArtifactId,
    ) -> Option<&DiagnosticHistory<ReplayEvent>> {
        self.replay_events_by_artifact.get(&artifact_id)
    }

    pub fn replay_cursor_offset(&self, cursor: ReplayCursor) -> Option<usize> {
        self.replay_cursor_offsets
            .get(&cursor)
            .copied()
            .map(|absolute| absolute.saturating_sub(self.replay_cursor_offset_base))
    }

    pub fn snapshot_replay_cursor(&self, snapshot_id: SignalSnapshotId) -> Option<ReplayCursor> {
        self.snapshot_replay_cursors.get(&snapshot_id).copied()
    }

    pub fn allocate_replay_cursor(&mut self) -> ReplayCursor {
        let cursor = ReplayCursor(self.next_replay_cursor);
        self.next_replay_cursor += 1;
        cursor
    }

    pub fn latest_replay_cursor(&self) -> Option<ReplayCursor> {
        self.replay_events.back().map(|event| event.cursor)
    }

    pub fn record_replay_event(&mut self, event: ReplayEvent) {
        self.replay_events_by_branch
            .entry(event.branch_id)
            .or_default()
            .push_back(event.clone())
            .expect("diagnostic history exhausted its private position space");
        if let Some(node) = event.node {
            self.replay_events_by_node
                .entry(node)
                .or_default()
                .push_back(event.clone())
                .expect("diagnostic history exhausted its private position space");
        }
        if let Some(artifact_id) = event.lineage_artifact_id {
            self.replay_events_by_artifact
                .entry(artifact_id)
                .or_default()
                .push_back(event.clone())
                .expect("diagnostic history exhausted its private position space");
        }
        if let Some(snapshot_id) = event.snapshot_id {
            self.snapshot_replay_cursors
                .insert(snapshot_id, event.cursor);
        }
        self.replay_events
            .push_back(event)
            .expect("diagnostic history exhausted its private position space");
        let absolute_index = self.replay_cursor_offset_base + self.replay_events.len() - 1;
        if let Some(cursor) = self.replay_events.back().map(|latest| latest.cursor) {
            self.replay_cursor_offsets.insert(cursor, absolute_index);
        }
        let limit = self.installed_retention_budget.history_limit.max(1) * 32;
        while self.replay_events.len() > limit {
            if let Some(event) = self.replay_events.pop_front() {
                self.replay_cursor_offset_base += 1;
                self.remove_replay_event_from_index(&event);
            }
        }
    }
}
