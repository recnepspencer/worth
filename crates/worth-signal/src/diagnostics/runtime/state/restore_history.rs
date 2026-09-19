use std::collections::{BTreeMap, VecDeque};
use std::sync::Arc;

use crate::diagnostics::failure::{FailureSummary, RollbackDiagnostic};
use crate::diagnostics::lineage::LineageRecord;
use crate::diagnostics::replay::{ReplayCursor, ReplayEvent};
use crate::diagnostics::summary::ExecutionHistorySummary;
use crate::logic::transaction::ObservationBoundarySummary;
use crate::state::{SignalBranchHandle, SignalBranchId, SignalSnapshotDiagnostics};

use super::DiagnosticsState;

impl DiagnosticsState {
    pub fn restore_snapshot_payload_preserving_history_from(
        &mut self,
        payload: SignalSnapshotDiagnostics,
        current: &DiagnosticsState,
    ) {
        let preservation = SnapshotHistoryPreservation::capture(current, &payload);

        self.restore_snapshot_payload(payload);
        preservation.merge_into(self);
        self.trim_restored_history();
    }

    fn trim_restored_history(&mut self) {
        self.trim_history();
        self.rebuild_indexes();
        let limit = self.installed_retention_budget.history_limit.max(1) * 32;
        while self.replay_events.len() > limit {
            if let Some(event) = self.replay_events.pop_front() {
                self.remove_replay_event_from_index(&event);
            }
        }
        while self.lineage_records.len() > limit {
            if let Some(record) = self.lineage_records.pop_front() {
                self.remove_lineage_record_from_index(&record);
            }
        }
    }
}

struct SnapshotHistoryPreservation {
    recent_history: VecDeque<ExecutionHistorySummary>,
    replay_events: VecDeque<ReplayEvent>,
    lineage_records: VecDeque<LineageRecord>,
    branch_catalog: BTreeMap<SignalBranchId, SignalBranchHandle>,
    latest_failure: Option<FailureSummary>,
    latest_rollback: Option<RollbackDiagnostic>,
    latest_observation: Option<ObservationBoundarySummary>,
    next_replay_cursor: u64,
    next_snapshot_id: u64,
    next_branch_id: u64,
    next_lineage_artifact_id: u64,
    next_lineage_sequence: u64,
    payload_latest_execution_record_id: Option<u64>,
    payload_last_replay_cursor: Option<ReplayCursor>,
    payload_last_lineage_sequence: Option<u64>,
    payload_captured_observation: bool,
}

impl SnapshotHistoryPreservation {
    fn capture(current: &DiagnosticsState, payload: &SignalSnapshotDiagnostics) -> Self {
        Self {
            recent_history: current.recent_history.iter().cloned().collect(),
            replay_events: current.replay_events.iter().cloned().collect(),
            lineage_records: current.lineage_records.iter().cloned().collect(),
            branch_catalog: current
                .branch_catalog
                .iter()
                .map(|(id, handle)| (*id, handle.clone()))
                .collect(),
            latest_failure: current.latest_failure.as_deref().cloned(),
            latest_rollback: current.latest_rollback.as_deref().cloned(),
            latest_observation: current.latest_observation.as_deref().cloned(),
            next_replay_cursor: current.next_replay_cursor,
            next_snapshot_id: current.next_snapshot_id,
            next_branch_id: current.next_branch_id,
            next_lineage_artifact_id: current.next_lineage_artifact_id,
            next_lineage_sequence: current.next_lineage_sequence,
            payload_latest_execution_record_id: payload
                .recent_history
                .iter()
                .filter_map(|summary| summary.latest_execution_record_id)
                .max(),
            payload_last_replay_cursor: payload.replay_frames.back().map(|event| event.cursor),
            payload_last_lineage_sequence: payload
                .lineage_records
                .back()
                .map(|record| record.sequence),
            payload_captured_observation: payload.latest_observation.is_some()
                || !payload.lineage_records.is_empty()
                || !payload.replay_frames.is_empty(),
        }
    }

    fn merge_into(self, state: &mut DiagnosticsState) {
        let Self {
            recent_history,
            replay_events,
            lineage_records,
            branch_catalog,
            latest_failure,
            latest_rollback,
            latest_observation,
            next_replay_cursor,
            next_snapshot_id,
            next_branch_id,
            next_lineage_artifact_id,
            next_lineage_sequence,
            payload_latest_execution_record_id,
            payload_last_replay_cursor,
            payload_last_lineage_sequence,
            payload_captured_observation,
        } = self;

        merge_recent_history_and_replay(
            state,
            recent_history,
            replay_events,
            payload_latest_execution_record_id,
            payload_last_replay_cursor,
        );
        merge_lineage_records(state, lineage_records, payload_last_lineage_sequence);
        restore_latest_diagnostics(
            state,
            latest_failure,
            latest_rollback,
            latest_observation,
            payload_captured_observation,
        );
        merge_branch_catalog(state, branch_catalog);

        state.next_replay_cursor = state.next_replay_cursor.max(next_replay_cursor);
        state.next_snapshot_id = state.next_snapshot_id.max(next_snapshot_id);
        state.next_branch_id = state.next_branch_id.max(next_branch_id);
        state.next_lineage_artifact_id =
            state.next_lineage_artifact_id.max(next_lineage_artifact_id);
        state.next_lineage_sequence = state.next_lineage_sequence.max(next_lineage_sequence);
    }
}

fn merge_recent_history_and_replay(
    state: &mut DiagnosticsState,
    recent_history: VecDeque<ExecutionHistorySummary>,
    replay_events: VecDeque<ReplayEvent>,
    payload_latest_execution_record_id: Option<u64>,
    payload_last_replay_cursor: Option<ReplayCursor>,
) {
    for summary in recent_history {
        let current_latest = summary.latest_execution_record_id;
        if payload_latest_execution_record_id
            .is_some_and(|latest| current_latest.is_some_and(|current| current > latest))
        {
            state
                .recent_history
                .push_back(summary)
                .expect("restored history fits its position space");
        }
    }
    for event in replay_events {
        if payload_last_replay_cursor.is_some_and(|latest| event.cursor > latest) {
            state
                .replay_events
                .push_back(event)
                .expect("restored history fits its position space");
        }
    }
}

fn merge_lineage_records(
    state: &mut DiagnosticsState,
    lineage_records: VecDeque<LineageRecord>,
    payload_last_lineage_sequence: Option<u64>,
) {
    for record in lineage_records {
        if payload_last_lineage_sequence.is_some_and(|latest| record.sequence > latest) {
            state
                .lineage_records
                .push_back(record)
                .expect("restored history fits its position space");
        }
    }
}

fn restore_latest_diagnostics(
    state: &mut DiagnosticsState,
    latest_failure: Option<FailureSummary>,
    latest_rollback: Option<RollbackDiagnostic>,
    latest_observation: Option<ObservationBoundarySummary>,
    payload_captured_observation: bool,
) {
    if !payload_captured_observation {
        return;
    }
    if latest_failure.is_some() {
        state.latest_failure = latest_failure.map(Arc::new);
    }
    if latest_rollback.is_some() {
        state.latest_rollback = latest_rollback.map(Arc::new);
    }
    if let Some(observation) = latest_observation {
        state.record_observation(observation);
    }
}

fn merge_branch_catalog(
    state: &mut DiagnosticsState,
    branch_catalog: BTreeMap<SignalBranchId, SignalBranchHandle>,
) {
    for (branch_id, branch_handle) in branch_catalog {
        match state.branch_catalog.get_mut(&branch_id) {
            Some(existing) if branch_id != state.active_branch => {
                if existing.head_snapshot_id.is_none() {
                    existing.head_snapshot_id = branch_handle.head_snapshot_id;
                }
            }
            None => {
                state.branch_catalog.insert(branch_id, branch_handle);
            }
            _ => {}
        }
    }
}
