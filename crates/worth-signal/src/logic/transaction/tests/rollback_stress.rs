use super::failure_subscribers::FailingSubscriber;
use super::runtime_world::{build_runtime, Ev, Tier};
use crate::data::checkpoint::CheckpointBarrier;
use crate::facade::{EvaluationRequestMode, NodeEvaluationResult, NodeState};
use crate::logic::transaction::TransactionOutcome;
use crate::tests::support::{
    define_keyed_computation, evaluate, version_ab, GraphDependencyBatchExt, ASPECT_A, ASPECT_B,
};

#[test]
fn hostile_rollback_and_commit_cycles_do_not_leak_semantic_events() {
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
    let a = graph.node().build();
    let mut runtime = build_runtime(graph);
    let mut ctx = ();

    let baseline_events = runtime.graph().replay_events().len();
    for cycle in 0..12 {
        let before_state = runtime.graph().get_state(a).unwrap();
        let mut tx = runtime.begin(request_execution, &mut ctx);
        tx.mark_dirty(a, ASPECT_B).unwrap();
        tx.emit_event(Ev::Tick);
        if cycle % 2 == 0 {
            assert_eq!(
                tx.rollback().unwrap().outcome,
                TransactionOutcome::RolledBack
            );
            assert_eq!(runtime.graph().get_state(a).unwrap(), before_state);
        } else {
            tx.flush_events(CheckpointBarrier::PerOperation).unwrap();
            assert_eq!(tx.commit().unwrap().outcome, TransactionOutcome::Committed);
        }
    }

    let replay_events = runtime.graph().replay_events();
    assert!(replay_events.len() > baseline_events);
    let mut last_sequence = None;
    for event in replay_events {
        if let Some(previous) = last_sequence {
            assert!(
                previous < event.cursor,
                "replay sequences must be strictly increasing"
            );
        }
        last_sequence = Some(event.cursor);
    }
}

#[test]
fn hostile_commit_failure_does_not_leak_committed_semantic_outcome() {
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
    let a = graph.node().build();
    let mut runtime = build_runtime(graph);
    runtime
        .event_bus_mut()
        .subscribe(Box::new(FailingSubscriber))
        .unwrap();

    let replay_len_before = runtime.graph().replay_events().len();
    let mut ctx = ();
    let mut tx = runtime.begin(request_execution, &mut ctx);
    tx.mark_dirty(a, ASPECT_B).unwrap();
    tx.emit_event(Ev::Tick);
    tx.flush_events(CheckpointBarrier::PerOperation).unwrap();

    let err = tx.commit().unwrap_err();
    assert!(format!("{err}").contains("event bus flush failed"));
    let replay_events = runtime.graph().replay_events();
    assert!(replay_events.len() >= replay_len_before + 2);
    let last = replay_events
        .back()
        .expect("rollback/failure replay should be recorded");
    assert_ne!(
        last.detail.as_ref().and_then(|detail| detail.as_message()),
        Some("transaction committed"),
        "failed transaction must not leak committed replay outcome"
    );
    assert!(
        replay_events.iter().rev().take(2).any(|event| {
            event.detail.as_ref().and_then(|detail| detail.as_message())
                == Some("event bus flush failed")
        }),
        "failed transaction should surface flush failure in replay events"
    );
}

#[test]
fn transaction_created_keyed_nodes_are_removed_on_rollback() {
    // This standalone caller declares the operational serial memory policy.
    let serial_request = worth_execution::SerialRequest::from_memory(
        worth_execution::SerialMemoryBudget::new(
            crate::runtime_policy::SignalRuntimePolicy::operational().serial_memory_bytes,
        ),
        worth_execution::CancellationToken::new(),
        None,
    );
    let request_execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let graph = crate::data::graph::SignalGraph::new();
    let mut runtime = build_runtime(graph);
    let positions = define_keyed_computation(&mut runtime, "positions", Tier::A);
    let arena_before = runtime.graph().arena_capacity();
    let active_before = runtime.graph().active_node_count();
    let mut ctx = ();

    let mut tx = runtime.begin(request_execution, &mut ctx);
    let created = positions.keyed("wing-root").node_in_transaction(&mut tx);
    assert!(tx.staged_graph().is_alive(created));
    assert_eq!(
        tx.rollback().unwrap().outcome,
        TransactionOutcome::RolledBack
    );

    assert_eq!(runtime.graph().arena_capacity(), arena_before);
    assert_eq!(runtime.graph().active_node_count(), active_before);
    assert!(!runtime.graph().is_alive(created));
    assert_eq!(
        runtime
            .telemetry()
            .transaction
            .rollback_packet_created_node_count,
        1
    );
}

#[test]
fn repeated_created_node_rollbacks_do_not_accumulate_storage_debris() {
    // This standalone caller declares the operational serial memory policy.
    let serial_request = worth_execution::SerialRequest::from_memory(
        worth_execution::SerialMemoryBudget::new(
            crate::runtime_policy::SignalRuntimePolicy::operational().serial_memory_bytes,
        ),
        worth_execution::CancellationToken::new(),
        None,
    );
    let request_execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let graph = crate::data::graph::SignalGraph::new();
    let mut runtime = build_runtime(graph);
    let positions = define_keyed_computation(&mut runtime, "positions", Tier::A);
    let mut ctx = ();

    let (
        (dep_edges_before, dep_segments_before),
        (sub_edges_before, sub_segments_before),
        snapshot_before,
    ) = runtime.graph().storage_counts();

    for _ in 0..12 {
        let mut tx = runtime.begin(request_execution, &mut ctx);
        let created = positions.keyed("wing-root").node_in_transaction(&mut tx);
        tx.evaluate_with_plan(
            created,
            &|view| Ok(view.finish(version_ab(1, 0))),
            EvaluationRequestMode::ForceOnDemand,
        )
        .unwrap();
        assert_eq!(
            tx.rollback().unwrap().outcome,
            TransactionOutcome::RolledBack
        );
    }

    let (
        (dep_edges_after, dep_segments_after),
        (sub_edges_after, sub_segments_after),
        snapshot_after,
    ) = runtime.graph().storage_counts();

    assert_eq!(dep_edges_after, dep_edges_before);
    assert_eq!(dep_segments_after, dep_segments_before);
    assert_eq!(sub_edges_after, sub_edges_before);
    assert_eq!(sub_segments_after, sub_segments_before);
    assert_eq!(snapshot_after, snapshot_before);
}

#[test]
fn committed_source_delta_stages_downstream_cause_and_rollback_restores_baseline() {
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
    let downstream = graph.node().build();
    graph
        .append_dependency(downstream, source, ASPECT_A)
        .unwrap();
    let mut runtime = build_runtime(graph);
    let mut ctx = ();

    let mut seed = |_id: crate::data::handle::NodeId, _graph: &crate::data::graph::SignalGraph| {
        Ok(version_ab(1, 0))
    };
    evaluate(runtime.graph_mut(), source, &mut seed).unwrap();
    evaluate(runtime.graph_mut(), downstream, &mut seed).unwrap();

    let mut tx = runtime.begin(request_execution, &mut ctx);
    tx.mark_dirty(source, ASPECT_A).unwrap();
    tx.evaluate_with_plan(
        source,
        &|view| Ok(view.finish(version_ab(2, 0))),
        EvaluationRequestMode::Default,
    )
    .unwrap();

    assert_eq!(
        tx.staged_graph().get_state(downstream).unwrap(),
        NodeState::Dirty
    );
    let causes = tx.staged_graph().pending_causes(downstream).unwrap();
    assert_eq!(causes.len(), 1);
    assert_eq!(causes[0].key.producer, source);
    assert_eq!(
        tx.rollback().unwrap().outcome,
        TransactionOutcome::RolledBack
    );
    assert_eq!(
        runtime.graph().get_state(downstream).unwrap(),
        NodeState::Clean
    );
}

#[test]
fn evaluate_dirty_rollback_restores_preexisting_dirty_nodes() {
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
    graph
        .get_entry_mut(source)
        .unwrap()
        .set_state(NodeState::Dirty);
    let before_version = graph.get_entry(source).unwrap().get_aspect_version();
    let mut runtime = build_runtime(graph);
    let mut ctx = ();

    let mut tx = runtime.begin(request_execution, &mut ctx);
    tx.evaluate_dirty(&|view| {
        let current = view
            .graph()
            .get_entry(view.node())?
            .get_aspect_version()
            .get(ASPECT_A);
        Ok(crate::logic::evaluation::EvaluationOutput::from_result(
            NodeEvaluationResult::from_version(version_ab(current + 1, 0)),
        ))
    })
    .unwrap();
    assert_eq!(
        tx.rollback().unwrap().outcome,
        TransactionOutcome::RolledBack
    );

    assert_eq!(
        runtime
            .graph()
            .get_entry(source)
            .unwrap()
            .get_aspect_version(),
        before_version
    );
    assert_eq!(runtime.graph().get_state(source).unwrap(), NodeState::Dirty);
}
