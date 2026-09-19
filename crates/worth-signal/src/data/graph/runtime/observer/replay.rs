use super::GraphObserver;
use crate::data::handle::NodeId;
use crate::diagnostics::lineage::LineageArtifactId;
use crate::diagnostics::replay::{
    ReplayCursor, ReplayEvent, RetainedReplayView, SynthesizedReplaySlice,
};

impl<'a> GraphObserver<'a> {
    pub fn replay_events(&self) -> RetainedReplayView<'a> {
        let frames = self.graph.observation.diagnostics.replay_events();
        RetainedReplayView::new(None, None, frames, 0, frames.len())
    }

    pub fn replay_where(
        &self,
        mut predicate: impl FnMut(&ReplayEvent) -> bool,
    ) -> SynthesizedReplaySlice {
        SynthesizedReplaySlice {
            start: None,
            end: None,
            frames: self
                .replay_events()
                .iter()
                .filter(|frame| predicate(frame))
                .cloned()
                .collect(),
        }
    }

    pub fn replay_slice(
        &self,
        start: Option<ReplayCursor>,
        end: Option<ReplayCursor>,
    ) -> RetainedReplayView<'a> {
        let start_index =
            start.and_then(|cursor| self.graph.diagnostics_state().replay_cursor_offset(cursor));
        let end_index = end
            .and_then(|cursor| self.graph.diagnostics_state().replay_cursor_offset(cursor))
            .map(|index| index + 1);
        if start_index.is_some() || end_index.is_some() {
            let start_index = start_index.unwrap_or(0);
            let end_index = end_index.unwrap_or_else(|| self.replay_events().len());
            return RetainedReplayView::new(
                start,
                end,
                self.graph.observation.diagnostics.replay_events(),
                start_index,
                end_index.saturating_sub(start_index),
            );
        }
        RetainedReplayView::new(
            start,
            end,
            self.graph.observation.diagnostics.replay_events(),
            0,
            self.replay_events().len(),
        )
    }

    pub fn replay_for_branch(
        &self,
        branch_id: crate::state::SignalBranchId,
    ) -> RetainedReplayView<'a> {
        self.graph
            .diagnostics_state()
            .replay_events_for_branch(branch_id)
            .map(|frames| RetainedReplayView::new(None, None, frames, 0, frames.len()))
            .unwrap_or_else(RetainedReplayView::empty)
    }

    pub fn replay_for_node(&self, node: NodeId) -> RetainedReplayView<'a> {
        self.graph
            .diagnostics_state()
            .replay_events_for_node(node)
            .map(|frames| RetainedReplayView::new(None, None, frames, 0, frames.len()))
            .unwrap_or_else(RetainedReplayView::empty)
    }

    pub fn replay_for_artifact(&self, artifact_id: LineageArtifactId) -> RetainedReplayView<'a> {
        self.graph
            .diagnostics_state()
            .replay_events_for_artifact(artifact_id)
            .map(|frames| RetainedReplayView::new(None, None, frames, 0, frames.len()))
            .unwrap_or_else(RetainedReplayView::empty)
    }

    pub fn replay_from_cursor(&self, start: ReplayCursor) -> RetainedReplayView<'a> {
        self.replay_slice(Some(start), None)
    }

    pub fn replay_between(&self, start: ReplayCursor, end: ReplayCursor) -> RetainedReplayView<'a> {
        self.replay_slice(Some(start), Some(end))
    }

    pub fn replay_around_snapshot(
        &self,
        snapshot_id: crate::state::SignalSnapshotId,
    ) -> RetainedReplayView<'a> {
        let Some(cursor) = self
            .graph
            .diagnostics_state()
            .snapshot_replay_cursor(snapshot_id)
        else {
            return RetainedReplayView::empty();
        };
        let Some(index) = self.graph.diagnostics_state().replay_cursor_offset(cursor) else {
            return RetainedReplayView::empty();
        };
        let start = index.saturating_sub(4);
        let end = (index + 5).min(self.replay_events().len());
        RetainedReplayView::new(
            self.replay_events()
                .iter()
                .nth(start)
                .map(|event| event.cursor),
            self.replay_events()
                .iter()
                .nth(end.saturating_sub(1))
                .map(|event| event.cursor),
            self.graph.observation.diagnostics.replay_events(),
            start,
            end.saturating_sub(start),
        )
    }
}
