use super::DiagnosticHistory;
use crate::data::persistent_ord_map::PersistentOrdMap;
use crate::diagnostics::replay::ReplayEvent;

use super::DiagnosticsState;

impl DiagnosticsState {
    pub(super) fn rebuild_indexes(&mut self) {
        self.replay_events_by_branch.clear();
        self.replay_events_by_node.clear();
        self.replay_events_by_artifact.clear();
        self.replay_cursor_offsets.clear();
        self.snapshot_replay_cursors.clear();
        self.replay_cursor_offset_base = 0;
        for event in self.replay_events.iter() {
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
        }
        self.rebuild_replay_cursor_offsets();
        self.lineage_records_by_artifact.clear();
        self.lineage_records_by_node.clear();
        for record in self.lineage_records.iter() {
            if let Some(node) = record.node() {
                self.lineage_records_by_node
                    .entry(node)
                    .or_default()
                    .push_back(record.clone())
                    .expect("diagnostic history exhausted its private position space");
            }
            if let Some(artifact_id) = record.subject_artifact_id() {
                self.lineage_records_by_artifact
                    .entry(artifact_id)
                    .or_default()
                    .push_back(record.clone())
                    .expect("diagnostic history exhausted its private position space");
            }
        }
    }

    pub(super) fn remove_replay_event_from_index(&mut self, event: &ReplayEvent) {
        remove_event_from_index(&mut self.replay_events_by_branch, &event.branch_id, event);
        if let Some(node) = event.node {
            remove_event_from_index(&mut self.replay_events_by_node, &node, event);
        }
        if let Some(artifact_id) = event.lineage_artifact_id {
            remove_event_from_index(&mut self.replay_events_by_artifact, &artifact_id, event);
        }
        self.replay_cursor_offsets.remove(&event.cursor);
        if let Some(snapshot_id) = event.snapshot_id {
            if self.snapshot_replay_cursors.get(&snapshot_id) == Some(&event.cursor) {
                self.snapshot_replay_cursors.remove(&snapshot_id);
            }
        }
    }

    pub(super) fn remove_lineage_record_from_index(
        &mut self,
        record: &crate::diagnostics::lineage::LineageRecord,
    ) {
        if let Some(node) = record.node() {
            remove_lineage_from_index(&mut self.lineage_records_by_node, &node, record);
        }
        if let Some(artifact_id) = record.subject_artifact_id() {
            remove_lineage_from_index(&mut self.lineage_records_by_artifact, &artifact_id, record);
        }
    }

    fn rebuild_replay_cursor_offsets(&mut self) {
        self.replay_cursor_offsets.clear();
        self.replay_cursor_offset_base = 0;
        for (index, event) in self.replay_events.iter().enumerate() {
            self.replay_cursor_offsets.insert(event.cursor, index);
        }
    }
}

#[cfg(test)]
#[path = "index_fork_tests.rs"]
mod tests;

fn remove_event_from_index<K: Clone + Ord>(
    index: &mut PersistentOrdMap<K, DiagnosticHistory<ReplayEvent>>,
    key: &K,
    event: &ReplayEvent,
) {
    let remove_key = if let Some(events) = index.get_mut(key) {
        if let Some(front) = events.front() {
            if front == event {
                events.pop_front();
            } else {
                let position = events.iter().position(|candidate| candidate == event);
                if let Some(position) = position {
                    events.remove(position);
                }
            }
        }
        events.is_empty()
    } else {
        false
    };
    if remove_key {
        index.remove(key);
    }
}

fn remove_lineage_from_index<K: Clone + Ord>(
    index: &mut PersistentOrdMap<K, DiagnosticHistory<crate::diagnostics::lineage::LineageRecord>>,
    key: &K,
    record: &crate::diagnostics::lineage::LineageRecord,
) {
    let remove_key = if let Some(records) = index.get_mut(key) {
        if let Some(front) = records.front() {
            if front == record {
                records.pop_front();
            } else {
                let position = records.iter().position(|candidate| candidate == record);
                if let Some(position) = position {
                    records.remove(position);
                }
            }
        }
        records.is_empty()
    } else {
        false
    };
    if remove_key {
        index.remove(key);
    }
}
