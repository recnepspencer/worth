use super::*;
use crate::data::aspect::Aspect;
use crate::diagnostics::failure::ExecutionFailurePhase;
use crate::diagnostics::profile::DiagnosticsTier;
use crate::facade::SignalGraph;

fn retained_state() -> DiagnosticsState {
    let mut graph = SignalGraph::new();
    graph.set_runtime_policy(SignalRuntimePolicy::forensic());
    let node = graph.node().build();
    let summary = graph
        .observe()
        .diagnostics_summary(DiagnosticsTier::Forensic);
    let mut state = graph.diagnostics_state().clone();
    // These payloads exercise retention mechanics, not execution certification.
    let mut failure =
        FailureSummary::suppressed(DiagnosticsTier::Forensic, ExecutionFailurePhase::Apply);
    failure.message = "failure detail".repeat(512);
    state.record_failure(failure);
    state.record_rollback(RollbackDiagnostic {
        rolled_back: true,
        staged_node_patch_count: 1,
        max_touched_nodes_in_txn: 1,
        reason: Some("rollback detail".repeat(512)),
        event_epochs: Vec::new(),
    });
    state.record_observation(ObservationBoundarySummary::default());
    state.latest_graph_summary = Some(Arc::new(summary.clone()));
    state.set_pending_graph_summary(summary);
    state.record_frontier_execution(
        InvalidationPlanningEstimate::default(),
        FrontierDiagnosticsSidecar::new(
            1,
            Vec::new(),
            Vec::new(),
            Default::default(),
            Default::default(),
        ),
        (0..512)
            .map(|wave| {
                InvalidationTraceRecord::new(
                    node,
                    Aspect::new(0),
                    wave,
                    Default::default(),
                    Default::default(),
                )
            })
            .collect(),
    );
    state
}

fn assert_shared<T>(left: &Option<Arc<T>>, right: &Option<Arc<T>>) {
    assert!(Arc::ptr_eq(left.as_ref().unwrap(), right.as_ref().unwrap()));
}

#[test]
fn summary_roots_are_shared_and_replacement_and_clear_preserve_siblings() {
    let mut source = retained_state();
    let mut draft = source.fork_persistent();
    assert_shared(&source.latest_failure, &draft.latest_failure);
    assert_shared(&source.latest_rollback, &draft.latest_rollback);
    assert_shared(&source.latest_observation, &draft.latest_observation);
    assert_shared(&source.latest_graph_summary, &draft.latest_graph_summary);
    assert_shared(&source.pending_graph_summary, &draft.pending_graph_summary);
    assert_shared(
        &source.latest_frontier_execution,
        &draft.latest_frontier_execution,
    );
    assert_eq!(
        source.latest_invalidation_planning_estimate(),
        draft.latest_invalidation_planning_estimate()
    );
    assert!(Arc::ptr_eq(
        &source.latest_invalidation_trace_records,
        &draft.latest_invalidation_trace_records
    ));
    let before = serde_json::to_value(&source).unwrap();
    draft.record_observation(ObservationBoundarySummary {
        classified_event_count: 2,
        ..Default::default()
    });
    draft.clear_pending_input();
    assert_eq!(serde_json::to_value(&source).unwrap(), before);
    assert_eq!(source.latest_invalidation_trace_records().len(), 512);
    assert!(draft.latest_invalidation_trace_records().is_empty());
    assert!(!Arc::ptr_eq(
        &source.latest_invalidation_trace_records,
        &draft.latest_invalidation_trace_records
    ));
    assert_shared(&source.latest_failure, &draft.latest_failure);
    assert_eq!(
        source.latest_observation().unwrap().classified_event_count,
        0
    );
    assert_eq!(
        draft.latest_observation().unwrap().classified_event_count,
        2
    );
}

#[test]
fn shared_summaries_export_owned_wire_and_restore_without_pointer_aliasing() {
    let source = retained_state();
    let payload = source.snapshot_payload_with_retention(Default::default());
    let wire = serde_json::to_value(&payload).unwrap();
    assert_eq!(
        wire["latest_failure"],
        serde_json::to_value(source.latest_failure()).unwrap()
    );
    assert_eq!(
        wire["latest_rollback"],
        serde_json::to_value(source.latest_rollback()).unwrap()
    );
    assert_eq!(
        wire["latest_observation"],
        serde_json::to_value(source.latest_observation()).unwrap()
    );
    let mut restored = DiagnosticsState::default();
    restored.restore_snapshot_payload(serde_json::from_value(wire).unwrap());
    assert_eq!(restored.latest_failure(), source.latest_failure());
    assert_eq!(restored.latest_rollback(), source.latest_rollback());
    assert_eq!(restored.latest_observation(), source.latest_observation());
    assert!(!Arc::ptr_eq(
        source.latest_failure.as_ref().unwrap(),
        restored.latest_failure.as_ref().unwrap()
    ));
    let restored: DiagnosticsState =
        serde_json::from_value(serde_json::to_value(&source).unwrap()).unwrap();
    assert_eq!(
        restored.latest_graph_summary(),
        source.latest_graph_summary()
    );
    assert_eq!(
        restored.latest_invalidation_trace_records(),
        source.latest_invalidation_trace_records()
    );
}
