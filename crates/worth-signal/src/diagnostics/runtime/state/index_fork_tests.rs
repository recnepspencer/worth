use super::super::DiagnosticsState;
use crate::diagnostics::facts::{ExplanationFact, ProvenanceFact};
use crate::diagnostics::lineage::{
    LineageArtifactId, LineageRecord, LineageRecordKind, SnapshotRestoreKind,
};
use crate::diagnostics::policy::ArtifactRetentionPolicy;
use crate::diagnostics::replay::{ReplayCursor, ReplayEvent, ReplayEventDetail, ReplayEventKind};
use crate::facade::{NodeId, SignalGraph};
use crate::state::{
    SignalBranchId, SignalSnapshotDiagnostics, SignalSnapshotId, SnapshotArtifactRetentionPolicy,
};

fn event(sequence: u64, node: NodeId) -> ReplayEvent {
    ReplayEvent {
        cursor: ReplayCursor(sequence),
        kind: ReplayEventKind::SnapshotCaptured,
        branch_id: SignalBranchId(0),
        snapshot_id: Some(SignalSnapshotId(sequence)),
        node: Some(node),
        execution_record_id: None,
        semantic_segment_id: None,
        lineage_artifact_id: Some(LineageArtifactId(sequence)),
        reuse_origin: None,
        persistent_correspondence_kind: None,
        composition_region_count: None,
        detail: Some(ReplayEventDetail::Message("retained evidence".repeat(128))),
    }
}

fn populated_state() -> (DiagnosticsState, [NodeId; 2]) {
    let mut graph = SignalGraph::new();
    let nodes = [graph.node().build(), graph.node().build()];
    let mut state = DiagnosticsState::default();
    for node in nodes {
        let explanation = graph.observe().explain(node).unwrap();
        state
            .explanation_facts
            .insert(node, ExplanationFact::from_explanation(&explanation));
        state
            .provenance_facts
            .insert(node, ProvenanceFact::from_explanation(&explanation));
    }
    for sequence in 0..24 {
        let node = nodes[sequence as usize % nodes.len()];
        state.record_replay_event(event(sequence, node));
        state.record_lineage_record(LineageRecord {
            sequence,
            emitted_on_branch_id: SignalBranchId(0),
            kind: LineageRecordKind::SnapshotRestore {
                snapshot_id: SignalSnapshotId(sequence),
                node: Some(node),
                artifact_id: Some(LineageArtifactId(sequence)),
                restore_kind: SnapshotRestoreKind::PerNodeArtifact,
            },
        });
    }
    (state, nodes)
}

fn assert_shared_indexes(left: &DiagnosticsState, right: &DiagnosticsState) {
    assert!(left
        .replay_events_by_branch
        .ptr_eq(&right.replay_events_by_branch));
    assert!(left
        .replay_events_by_node
        .ptr_eq(&right.replay_events_by_node));
    assert!(left
        .replay_events_by_artifact
        .ptr_eq(&right.replay_events_by_artifact));
    assert!(left
        .replay_cursor_offsets
        .ptr_eq(&right.replay_cursor_offsets));
    assert!(left
        .snapshot_replay_cursors
        .ptr_eq(&right.snapshot_replay_cursors));
    assert!(left
        .lineage_records_by_artifact
        .ptr_eq(&right.lineage_records_by_artifact));
    assert!(left
        .lineage_records_by_node
        .ptr_eq(&right.lineage_records_by_node));
    assert!(left.explanation_facts.ptr_eq(&right.explanation_facts));
    assert!(left.provenance_facts.ptr_eq(&right.provenance_facts));
}

#[test]
fn prepared_diagnostic_indexes_share_roots_and_isolate_changed_entries() {
    let (mut source, nodes) = populated_state();
    let original = snapshot(&source);
    let mut draft = source.fork_persistent();
    assert_shared_indexes(&source, &draft);
    assert_eq!(snapshot(&draft), original);
    let retained_frame = source
        .replay_events_for_node(nodes[0])
        .unwrap()
        .front()
        .unwrap();
    let untouched_fact = source.explanation_facts.get(&nodes[1]).unwrap();

    draft.record_replay_event(event(100, nodes[0]));
    draft.explanation_facts.get_mut(&nodes[0]).unwrap().state = "draft annotation".into();
    draft.provenance_facts.remove(&nodes[0]);
    assert_eq!(snapshot(&source), original);
    assert_eq!(draft.replay_events_for_node(nodes[0]).unwrap().len(), 13);
    assert!(std::ptr::eq(
        retained_frame,
        draft
            .replay_events_for_node(nodes[0])
            .unwrap()
            .front()
            .unwrap()
    ));
    assert!(std::ptr::eq(
        untouched_fact,
        draft.explanation_facts.get(&nodes[1]).unwrap()
    ));
    assert!(source.provenance_facts.contains_key(&nodes[0]));
    assert!(!draft.provenance_facts.contains_key(&nodes[0]));
}

#[test]
fn restored_and_rebuilt_indexes_can_be_shared_again_without_changing_snapshot_wire() {
    let (source, _) = populated_state();
    let payload = snapshot(&source);
    let wire = serde_json::to_value(&payload).unwrap();
    let mut restored = DiagnosticsState::default();
    restored.restore_snapshot_payload(serde_json::from_value(wire.clone()).unwrap());
    let sibling = restored.fork_persistent();
    assert_shared_indexes(&restored, &sibling);
    assert_eq!(serde_json::to_value(snapshot(&restored)).unwrap(), wire);
    restored.rebuild_indexes();
    let rebuilt_fork = restored.fork_persistent();
    assert_shared_indexes(&restored, &rebuilt_fork);
    assert_eq!(snapshot(&restored), snapshot(&sibling));
    assert_eq!(restored.replay_cursor_offset(ReplayCursor(23)), Some(23));
    assert_eq!(
        restored.snapshot_replay_cursor(SignalSnapshotId(23)),
        Some(ReplayCursor(23))
    );
}

#[test]
fn eviction_preserves_a_newer_snapshot_mapping_and_the_shared_sibling() {
    let (mut source, nodes) = populated_state();
    let mut draft = source.fork_persistent();
    let older = event(0, nodes[0]);
    let mut newer = event(100, nodes[0]);
    newer.snapshot_id = older.snapshot_id;
    draft.record_replay_event(newer);
    draft.remove_replay_event_from_index(&older);
    assert_eq!(
        draft.snapshot_replay_cursor(SignalSnapshotId(0)),
        Some(ReplayCursor(100))
    );
    assert_eq!(
        source.snapshot_replay_cursor(SignalSnapshotId(0)),
        Some(ReplayCursor(0))
    );
    assert_eq!(source.replay_cursor_offset(ReplayCursor(0)), Some(0));
    assert_eq!(draft.replay_cursor_offset(ReplayCursor(0)), None);
    assert_eq!(draft.replay_events_for_node(nodes[0]).unwrap().len(), 12);
}

fn snapshot(state: &DiagnosticsState) -> SignalSnapshotDiagnostics {
    state.snapshot_payload_with_retention(SnapshotArtifactRetentionPolicy {
        explanation_retention: ArtifactRetentionPolicy::Retain,
        provenance_retention: ArtifactRetentionPolicy::Retain,
    })
}
