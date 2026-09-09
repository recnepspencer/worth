use super::*;
use crate::diagnostics::epochs::{
    EventEpochOutcome, EventEpochSummary, EventSubscriberOutcome, EventSubscriberOutcomeKind,
};
use crate::diagnostics::failure::{ExecutionFailurePhase, FailureSummary, RollbackDiagnostic};
use crate::diagnostics::profile::DiagnosticsTier;
use crate::facade::{NodeEvaluationResult, SignalGraph, SignalRuntimePolicy};
use crate::tests::support::{evaluate, version_ab, ASPECT_A};

fn recorded_state() -> DiagnosticsState {
    let mut graph = SignalGraph::new();
    graph.set_runtime_policy(SignalRuntimePolicy::forensic());
    let node = graph.node().output_identity().build();
    evaluate(&mut graph, node, &mut |_id, _graph| {
        Ok(NodeEvaluationResult::from_version(version_ab(1, 0))
            .with_output_identity("retained state")
            .with_label("diagnostic detail"))
    })
    .unwrap();
    let explanation = graph.observe().explain(node).unwrap();
    let mut state = graph.diagnostics_state().clone();
    state.record_explanation_fact(
        crate::diagnostics::facts::ExplanationFact::from_explanation(&explanation),
    );
    state.record_provenance_fact(crate::diagnostics::facts::ProvenanceFact::from_explanation(
        &explanation,
    ));
    assert!(state.latest_flow.is_some());
    assert!(!state.recent_history.is_empty());
    assert!(!state.replay_events.is_empty());
    assert!(!state.lineage_records.is_empty());
    // Pending and failed output below are explicit retained representation fixtures.
    let epoch = EventEpochSummary {
        ordinal: 0,
        barrier: crate::data::checkpoint::CheckpointBarrier::PerOperation,
        emitted_event_count: 1,
        subscriber_count: 1,
        committed_subscriber_count: 0,
        failed_subscriber_position: Some(0),
        outcome: EventEpochOutcome::Failed,
        failure_subscriber: Some("subscriber".into()),
        message: Some("failure detail".repeat(32)),
        subscriber_outcomes: vec![EventSubscriberOutcome {
            subscriber_name: "subscriber".into(),
            outcome: EventSubscriberOutcomeKind::Failed,
            requires_data_ids: vec!["required".repeat(16)],
            provides_data_ids: vec!["provided".repeat(16)],
            staged_data_ids: vec!["staged".repeat(16)],
        }],
    };
    let mut failure =
        FailureSummary::suppressed(DiagnosticsTier::Forensic, ExecutionFailurePhase::Apply);
    failure.message = "retained failure".repeat(64);
    failure.event_epochs = vec![epoch.clone()];
    state.record_failure(failure);
    state.record_rollback(RollbackDiagnostic {
        rolled_back: true,
        staged_node_patch_count: 1,
        max_touched_nodes_in_txn: 1,
        reason: Some("retained rollback".repeat(64)),
        event_epochs: vec![epoch.clone()],
    });
    state.attach_event_epochs_to_latest_flow(vec![epoch]);
    state.record_observation(Default::default());
    state.note_change_input(node, ASPECT_A, &[], Some("pending causality".repeat(64)));
    state
}

#[test]
fn empty_aggregate_matches_the_immutable_carrier_heap_contract() {
    let mut state = DiagnosticsState::default();
    let carrier = state
        .retained_branch_carrier_charge(&mut Work::new(10000))
        .unwrap();
    assert_eq!(
        state
            .prepare_retained_heap_charge(&mut Work::new(10000))
            .unwrap(),
        carrier
    );
}

#[test]
fn cold_diagnostic_preparation_denial_preserves_semantics_and_allows_retry() {
    let mut state = recorded_state();
    let mut short = state.clone();
    let wire = serde_json::to_value(&state).unwrap();
    let mut work = Work::new(100000);
    let charge = state.prepare_retained_heap_charge(&mut work).unwrap();
    assert!(charge.bytes() > 0);
    assert_eq!(serde_json::to_value(&state).unwrap(), wire);
    let limit = work.visits() - 1;
    assert_eq!(
        short.prepare_retained_heap_charge(&mut Work::new(limit)),
        Err(Denial::WorkExhausted {
            maximum_visits: limit
        })
    );
    assert_eq!(serde_json::to_value(&short).unwrap(), wire);
    short
        .prepare_retained_heap_charge(&mut Work::new(100000))
        .unwrap();
    assert_eq!(serde_json::to_value(&short).unwrap(), wire);
    let mut restored: DiagnosticsState =
        serde_json::from_str(&serde_json::to_string(&state).unwrap()).unwrap();
    restored.rebuild_indexes();
    restored
        .prepare_retained_heap_charge(&mut Work::new(100000))
        .unwrap();
    assert_eq!(serde_json::to_value(&restored).unwrap(), wire);
}

#[test]
fn prepared_diagnostic_children_keep_custody_and_accounting_across_full_fork() {
    let mut state = recorded_state();
    state
        .prepare_retained_heap_charge(&mut Work::new(100000))
        .unwrap();
    let wire = serde_json::to_value(&state).unwrap();
    let mut fork = state.fork_persistent();
    assert!(state.explanation_facts.ptr_eq(&fork.explanation_facts));
    assert!(state
        .replay_events_by_node
        .ptr_eq(&fork.replay_events_by_node));
    assert_eq!(
        state.recent_history.prepared_retained_charge().unwrap(),
        fork.recent_history.prepared_retained_charge().unwrap()
    );
    let before = fork
        .prepare_retained_heap_charge(&mut Work::new(100000))
        .unwrap();
    let observation = fork.latest_observation.as_ref().unwrap();
    let observation_charge = observation
        .retained_heap_charge(&mut Work::new(10000))
        .unwrap();
    fork.latest_observation = None;
    let after = fork
        .prepare_retained_heap_charge(&mut Work::new(100000))
        .unwrap();
    // The flow still owns the same observation Arc; only this root's charge is removed.
    assert_eq!(before.bytes() - after.bytes(), observation_charge.bytes());
    assert_eq!(serde_json::to_value(&state).unwrap(), wire);
    assert!(fork
        .latest_flow
        .as_ref()
        .unwrap()
        .view()
        .observation
        .is_some());
}

fn assert_prepared_index<K: Clone + Ord, T>(
    source: &PersistentOrdMap<K, DiagnosticHistory<T>>,
    sibling: &PersistentOrdMap<K, DiagnosticHistory<T>>,
) {
    assert!(!source.is_empty());
    assert!(source.ptr_eq(sibling));
    for (key, history) in source.iter() {
        assert!(history.prepared_retained_charge().is_ok());
        assert!(std::ptr::eq(history, sibling.get(key).unwrap()));
    }
}

#[test]
fn aggregate_cold_preparation_prepares_all_effective_native_history_indexes_in_place() {
    let mut state = recorded_state();
    let sibling = state.fork_persistent();
    let wire = serde_json::to_value(&state).unwrap();
    state
        .prepare_retained_heap_charge(&mut Work::new(100_000))
        .unwrap();
    assert_prepared_index(
        &state.replay_events_by_branch,
        &sibling.replay_events_by_branch,
    );
    assert_prepared_index(&state.replay_events_by_node, &sibling.replay_events_by_node);
    assert_prepared_index(
        &state.replay_events_by_artifact,
        &sibling.replay_events_by_artifact,
    );
    assert_prepared_index(
        &state.lineage_records_by_artifact,
        &sibling.lineage_records_by_artifact,
    );
    assert_prepared_index(
        &state.lineage_records_by_node,
        &sibling.lineage_records_by_node,
    );
    assert_eq!(serde_json::to_value(&state).unwrap(), wire);
    assert_eq!(serde_json::to_value(&sibling).unwrap(), wire);
}
