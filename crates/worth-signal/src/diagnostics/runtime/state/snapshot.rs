use std::collections::BTreeMap;
use std::sync::Arc;

use crate::runtime_policy::SignalRuntimePolicy;
use crate::state::{
    SignalSnapshotDiagnostics, SignalSnapshotId, SignalSnapshotMeta,
    SnapshotArtifactRetentionPolicy,
};

use super::DiagnosticsState;

impl DiagnosticsState {
    pub fn allocate_snapshot_meta(
        &mut self,
        policy: SignalRuntimePolicy,
        artifact_retention: SnapshotArtifactRetentionPolicy,
    ) -> SignalSnapshotMeta {
        self.bootstrap_defaults();
        let snapshot_id = SignalSnapshotId(self.next_snapshot_id);
        self.next_snapshot_id += 1;
        self.snapshot_meta(snapshot_id, policy, artifact_retention)
    }

    pub(crate) fn allocate_snapshot_meta_with_reserved_id(
        &mut self,
        snapshot_id: SignalSnapshotId,
        policy: SignalRuntimePolicy,
        artifact_retention: SnapshotArtifactRetentionPolicy,
    ) -> SignalSnapshotMeta {
        self.bootstrap_defaults();
        debug_assert!(
            snapshot_id.0 < u64::MAX,
            "owner snapshot identity exhaustion is denied during reservation"
        );
        self.next_snapshot_id = self.next_snapshot_id.max(snapshot_id.0.saturating_add(1));
        self.snapshot_meta(snapshot_id, policy, artifact_retention)
    }

    fn snapshot_meta(
        &self,
        snapshot_id: SignalSnapshotId,
        policy: SignalRuntimePolicy,
        artifact_retention: SnapshotArtifactRetentionPolicy,
    ) -> SignalSnapshotMeta {
        let branch = self.active_branch();
        let replay_head = self.replay_events.back().map(|frame| frame.cursor);
        SignalSnapshotMeta::new(
            snapshot_id,
            &branch,
            replay_head,
            policy,
            artifact_retention,
        )
    }

    pub fn snapshot_payload_with_retention(
        &self,
        artifact_retention: SnapshotArtifactRetentionPolicy,
    ) -> SignalSnapshotDiagnostics {
        SignalSnapshotDiagnostics {
            latest_flow: self.latest_flow().map(|flow| flow.to_owned_summary()),
            latest_failure: self.latest_failure.as_deref().cloned(),
            latest_rollback: self.latest_rollback.as_deref().cloned(),
            latest_observation: self.latest_observation.as_deref().cloned(),
            recent_history: self.recent_history.iter().cloned().collect(),
            replay_frames: self.replay_events.iter().cloned().collect(),
            explanation_facts: if artifact_retention.retains_explanation_facts() {
                self.explanation_facts
                    .iter()
                    .map(|(node, fact)| (*node, fact.clone()))
                    .collect()
            } else {
                BTreeMap::new()
            },
            provenance_facts: if artifact_retention.retains_provenance_facts() {
                self.provenance_facts
                    .iter()
                    .map(|(node, fact)| (*node, fact.clone()))
                    .collect()
            } else {
                BTreeMap::new()
            },
            lineage_records: self.lineage_records.iter().cloned().collect(),
            branch_catalog: self
                .branch_catalog
                .iter()
                .map(|(id, handle)| (*id, handle.clone()))
                .collect(),
            active_branch: self.active_branch,
            next_replay_cursor: self.next_replay_cursor,
            next_snapshot_id: self.next_snapshot_id,
            next_branch_id: self.next_branch_id,
            next_lineage_artifact_id: self.next_lineage_artifact_id,
            next_lineage_sequence: self.next_lineage_sequence,
            observation_activation_mask: self.observation_activation_mask,
        }
    }

    pub fn restore_snapshot_payload(&mut self, payload: SignalSnapshotDiagnostics) {
        self.latest_flow = payload.latest_flow.map(Into::into);
        self.latest_failure = payload.latest_failure.map(Arc::new);
        self.latest_rollback = payload.latest_rollback.map(Arc::new);
        self.latest_observation = payload.latest_observation.map(Arc::new);
        self.recent_history = payload.recent_history.into_iter().collect();
        self.replay_events = payload.replay_frames.into_iter().collect();
        self.explanation_facts = payload.explanation_facts.into_iter().collect();
        self.provenance_facts = payload.provenance_facts.into_iter().collect();
        self.lineage_records = payload.lineage_records.into_iter().collect();
        self.branch_catalog = payload.branch_catalog.into_iter().collect();
        self.active_branch = payload.active_branch;
        self.next_replay_cursor = payload.next_replay_cursor;
        self.next_snapshot_id = payload.next_snapshot_id;
        self.next_branch_id = payload.next_branch_id;
        self.next_lineage_artifact_id = payload.next_lineage_artifact_id;
        self.next_lineage_sequence = payload.next_lineage_sequence;
        self.observation_activation_mask = payload.observation_activation_mask;
        self.pending_input = None;
        self.pending_graph_summary = None;
        self.rebuild_indexes();
    }
}
