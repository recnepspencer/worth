use super::super::failure_subscribers::FailingSubscriber;
use super::super::runtime_world::{build_runtime, Ev};
use super::world::{CommittedObservationRecord, Phase3RecordingObservationListener};
use crate::data::checkpoint::CheckpointBarrier;
use crate::facade::{
    AuthorityPolicy, EvaluationRequestMode, NodeEvaluationResult, ObservationBoundaryOutcome,
    ObservationPolicy,
};
use crate::tests::support::{version_ab, GraphDependencyBatchExt, ASPECT_A};
use std::sync::{Arc, Mutex};

#[test]
fn observation_phase3_commit_dispatches_once_per_observer_per_transaction() {
    // This standalone caller declares the operational serial memory policy.
    let serial_request = worth_execution::SerialRequest::from_memory(
        worth_execution::SerialMemoryBudget::new(
            crate::runtime_policy::SignalRuntimePolicy::operational().serial_memory_bytes,
        ),
        worth_execution::CancellationToken::new(),
        None,
    );
    let request_execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let mut graph = crate::data::graph::SignalGraph::new();
    let source = graph
        .node()
        .authority_policy(AuthorityPolicy::AuthoritativeOnly)
        .build();
    let derived = graph.node().build();
    graph.append_dependency(derived, source, ASPECT_A).unwrap();
    let mut runtime = build_runtime(graph);
    let calls = Arc::new(Mutex::new(Vec::<CommittedObservationRecord>::new()));

    runtime.observe_nodes(
        ObservationPolicy::visited(),
        [source, derived],
        Box::new(Phase3RecordingObservationListener {
            calls: Arc::clone(&calls),
        }),
    );

    let mut ctx = ();
    let mut tx = runtime.begin(request_execution, &mut ctx);
    tx.mark_dirty(source, ASPECT_A).unwrap();
    tx.evaluate_dirty(&|view| {
        if view.node() == source {
            Ok(view.finish(NodeEvaluationResult::from_version(version_ab(1, 0))))
        } else {
            Ok(view.finish(NodeEvaluationResult::from_version(version_ab(10, 0))))
        }
    })
    .unwrap();

    let result = tx.commit().unwrap();
    let calls = calls.lock().expect("phase3 observation mutex poisoned");
    assert_eq!(
        calls.len(),
        1,
        "one delivery per observer per committed transaction"
    );
    assert_eq!(calls[0].matched_node_count, 2);
    assert!(calls[0].visited);
    assert!(calls[0].recomputed);
    assert!(calls[0].meaningful_change);
    assert!(calls[0].trigger_matched);
    assert_eq!(result.observation.classified_event_count, 1);
    assert_eq!(result.observation.trigger_matched_event_count, 1);
    assert_eq!(result.observation.delivered_event_count, 1);
    assert_eq!(result.observation.rollback_suppressed_event_count, 0);
    assert_eq!(result.observation.boundary_events.len(), 1);
    assert_eq!(result.observation.boundary_events[0].matched_nodes.len(), 2);
    assert_eq!(
        runtime.telemetry().transaction.delivered_observation_count,
        1
    );
}

#[test]
fn observation_phase3_touched_observer_fires_for_commit_without_execution_report() {
    // This standalone caller declares the operational serial memory policy.
    let serial_request = worth_execution::SerialRequest::from_memory(
        worth_execution::SerialMemoryBudget::new(
            crate::runtime_policy::SignalRuntimePolicy::operational().serial_memory_bytes,
        ),
        worth_execution::CancellationToken::new(),
        None,
    );
    let request_execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let mut graph = crate::data::graph::SignalGraph::new();
    let source = graph.node().build();
    let mut runtime = build_runtime(graph);
    let calls = Arc::new(Mutex::new(Vec::<CommittedObservationRecord>::new()));
    let handle = runtime.observe_nodes(
        ObservationPolicy::visited(),
        [source],
        Box::new(Phase3RecordingObservationListener {
            calls: Arc::clone(&calls),
        }),
    );

    let mut ctx = ();
    let result = runtime
        .transaction(request_execution, &mut ctx, |tx| {
            tx.mark_dirty(source, ASPECT_A)?;
            Ok(())
        })
        .expect("touch-only transaction should commit");

    let recorded = calls
        .lock()
        .expect("phase3 touch-only observation mutex poisoned")
        .clone();
    assert_eq!(recorded.len(), 1);
    assert_eq!(
        recorded[0],
        CommittedObservationRecord {
            observer_id: handle.observer_id().get(),
            handle_id: handle.handle_id().get(),
            matched_node_count: 1,
            visited: true,
            recomputed: false,
            meaningful_change: false,
            trigger_matched: true,
        }
    );
    assert_eq!(result.observation.classified_event_count, 1);
    assert_eq!(result.observation.trigger_matched_event_count, 1);
    assert_eq!(result.observation.delivered_event_count, 1);
    assert_eq!(result.observation.rollback_suppressed_event_count, 0);
}

#[test]
fn observation_phase3_rollback_suppresses_normal_delivery_and_records_boundary_summary() {
    // This standalone caller declares the operational serial memory policy.
    let serial_request = worth_execution::SerialRequest::from_memory(
        worth_execution::SerialMemoryBudget::new(
            crate::runtime_policy::SignalRuntimePolicy::operational().serial_memory_bytes,
        ),
        worth_execution::CancellationToken::new(),
        None,
    );
    let request_execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let mut graph = crate::data::graph::SignalGraph::new();
    let source = graph
        .node()
        .authority_policy(AuthorityPolicy::AuthoritativeOnly)
        .build();
    let mut runtime = build_runtime(graph);
    let calls = Arc::new(Mutex::new(Vec::<CommittedObservationRecord>::new()));

    runtime.observe_nodes(
        ObservationPolicy::meaningful_change(),
        [source],
        Box::new(Phase3RecordingObservationListener {
            calls: Arc::clone(&calls),
        }),
    );

    let mut ctx = ();
    let mut tx = runtime.begin(request_execution, &mut ctx);
    tx.evaluate_with_plan(
        source,
        &|view| Ok(view.finish(NodeEvaluationResult::from_version(version_ab(1, 0)))),
        EvaluationRequestMode::Default,
    )
    .unwrap();

    let result = tx.rollback().unwrap();
    assert!(
        calls
            .lock()
            .expect("phase3 observation mutex poisoned")
            .is_empty(),
        "rollback must not emit normal observation delivery"
    );
    assert_eq!(result.observation.classified_event_count, 1);
    assert_eq!(result.observation.trigger_matched_event_count, 1);
    assert_eq!(result.observation.delivered_event_count, 0);
    assert_eq!(result.observation.rollback_suppressed_event_count, 1);
    assert_eq!(result.observation.boundary_events.len(), 1);
    assert!(matches!(
        result.observation.boundary_events[0].outcome,
        ObservationBoundaryOutcome::RollbackSuppressed
    ));
    assert_eq!(
        runtime
            .telemetry()
            .transaction
            .rollback_suppressed_observation_count,
        1
    );
}

#[test]
fn observation_phase3_failed_commit_suppresses_delivery_during_fail_and_rollback() {
    // This standalone caller declares the operational serial memory policy.
    let serial_request = worth_execution::SerialRequest::from_memory(
        worth_execution::SerialMemoryBudget::new(
            crate::runtime_policy::SignalRuntimePolicy::operational().serial_memory_bytes,
        ),
        worth_execution::CancellationToken::new(),
        None,
    );
    let request_execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let mut graph = crate::data::graph::SignalGraph::new();
    let source = graph
        .node()
        .authority_policy(AuthorityPolicy::AuthoritativeOnly)
        .build();
    let mut runtime = build_runtime(graph);
    runtime
        .event_bus_mut()
        .subscribe(Box::new(FailingSubscriber))
        .unwrap();
    let calls = Arc::new(Mutex::new(Vec::<CommittedObservationRecord>::new()));

    runtime.observe_nodes(
        ObservationPolicy::meaningful_change(),
        [source],
        Box::new(Phase3RecordingObservationListener {
            calls: Arc::clone(&calls),
        }),
    );

    let mut ctx = ();
    let mut tx = runtime.begin(request_execution, &mut ctx);
    tx.evaluate_with_plan(
        source,
        &|view| Ok(view.finish(NodeEvaluationResult::from_version(version_ab(1, 0)))),
        EvaluationRequestMode::Default,
    )
    .unwrap();
    tx.emit_event(Ev::Tick);
    tx.flush_events(CheckpointBarrier::PerOperation).unwrap();

    let err = tx.commit().unwrap_err();
    assert!(format!("{err}").contains("event bus flush failed"));
    assert!(
        calls
            .lock()
            .expect("phase3 observation mutex poisoned")
            .is_empty(),
        "failed commit rollback must suppress normal observation delivery"
    );
    assert_eq!(
        runtime
            .telemetry()
            .transaction
            .rollback_suppressed_observation_count,
        1
    );
    assert_eq!(
        runtime.telemetry().transaction.delivered_observation_count,
        0
    );
}
