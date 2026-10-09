use super::failure_subscribers::FailingSubscriber;
use super::runtime_world::{build_runtime, Domain, Ev, Impact};
use crate::data::checkpoint::CheckpointBarrier;
use crate::logic::transaction::TransactionOutcome;
use crate::tests::support::ASPECT_B;

#[derive(Debug, Clone, Copy)]
enum TestEffect {
    CacheOne,
}

struct TestEffectMap;
impl crate::data::effect_mapping::EffectMapping for TestEffectMap {
    type Domain = Domain;
    type Impact = Impact;
    type Effect = TestEffect;

    fn route(
        effect: &Self::Effect,
        out: &mut crate::data::dirty_set::BatchedDirtySet<Self::Domain, Self::Impact>,
    ) {
        match effect {
            TestEffect::CacheOne => out.mark_domain_scoped(Domain::Cache, Impact::One),
        }
    }
}

#[test]
fn begin_rollback_preserves_committed_state() {
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
    let before = graph.get_state(a).unwrap();
    let mut runtime = build_runtime(graph);

    let mut ctx = ();
    let mut tx = runtime.begin(request_execution, &mut ctx);
    tx.mark_dirty(a, ASPECT_B).unwrap();
    assert_eq!(
        tx.rollback().unwrap().outcome,
        TransactionOutcome::RolledBack
    );
    assert_eq!(runtime.graph().get_state(a).unwrap(), before);
}

#[test]
fn rollback_result_carries_rollback_diagnostic() {
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
    let mut tx = runtime.begin(request_execution, &mut ctx);
    tx.mark_dirty(a, ASPECT_B).unwrap();

    let result = tx.rollback().unwrap();
    assert_eq!(result.outcome, TransactionOutcome::RolledBack);
    assert!(result.rollback.is_some());
    assert!(result.execution_report.is_none());
    assert!(result.decision_summary.rollback_recorded);
    assert!(result.decision_summary.rolled_back);
}

#[test]
fn read_only_rollback_emits_zero_rollback_packets() {
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
    let mut ctx = ();

    let result = runtime
        .begin(request_execution, &mut ctx)
        .rollback()
        .unwrap();

    assert_eq!(result.outcome, TransactionOutcome::RolledBack);
    assert_eq!(runtime.telemetry().transaction.rollback_packet_breadth, 0);
    assert_eq!(
        runtime.telemetry().transaction.rollback_packet_config_count,
        0
    );
    assert_eq!(
        runtime
            .telemetry()
            .transaction
            .rollback_packet_diagnostics_count,
        0
    );
    assert_eq!(
        runtime
            .telemetry()
            .transaction
            .rollback_packet_graph_patch_count,
        0
    );
    assert_eq!(
        runtime
            .telemetry()
            .transaction
            .rollback_packet_created_node_count,
        0
    );
    assert_eq!(
        runtime
            .telemetry()
            .transaction
            .rollback_packet_subscriber_repair_count,
        0
    );
}

#[test]
fn failed_event_flush_does_not_commit_graph_state() {
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
    let before = graph.get_state(a).unwrap();
    let mut runtime = build_runtime(graph);
    runtime
        .event_bus_mut()
        .subscribe(Box::new(FailingSubscriber))
        .unwrap();

    let mut ctx = ();
    let mut tx = runtime.begin(request_execution, &mut ctx);
    tx.mark_dirty(a, ASPECT_B).unwrap();
    tx.emit_event(Ev::Tick);
    tx.flush_events(CheckpointBarrier::PerOperation).unwrap();

    let err = tx.commit().unwrap_err();
    let msg = format!("{err}");
    assert!(msg.contains("event bus flush failed"));
    assert_eq!(runtime.graph().get_state(a).unwrap(), before);
    assert!(runtime.telemetry().transaction.transaction_poison_count >= 1);
}

#[test]
fn commit_failure_discards_checkpoint_state() {
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

    let mut ctx = ();
    let mut tx = runtime.begin(request_execution, &mut ctx);
    tx.mark_dirty(a, ASPECT_B).unwrap();
    tx.record_effect::<TestEffectMap>(&TestEffect::CacheOne);
    tx.emit_event(Ev::Tick);
    tx.flush_events(CheckpointBarrier::PerOperation).unwrap();

    let _ = tx.commit().unwrap_err();
    assert!(runtime.checkpoint().dirty().is_empty());
}
