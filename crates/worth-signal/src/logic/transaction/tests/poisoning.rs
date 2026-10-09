use super::failure_subscribers::{FailingSubscriber, NeedsMissingProviderSubscriber};
use super::runtime_world::{build_runtime, Ev};
use crate::data::checkpoint::CheckpointBarrier;
use crate::facade::{DecisionDetail, ExecutionFailurePhase};
use crate::logic::transaction::TransactionOutcome;
use crate::tests::support::ASPECT_B;

#[test]
fn poisoned_transaction_returns_poisoned_outcome() {
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
    let mut tx = runtime.begin(request_execution, &mut ctx);

    // Invalid handle poisons transaction.
    let invalid = crate::data::handle::NodeId::new(999_999, 0);
    assert!(tx.mark_dirty(invalid, ASPECT_B).is_err());
    let result = tx.commit().unwrap();
    assert_eq!(result.outcome, TransactionOutcome::Poisoned);
    assert!(result.decision_summary.poisoned);
    assert!(result.decision_summary.rollback_recorded);
    assert!(result.decision_summary.failure_recorded);
    assert!(result.integrity_markers.rollback_attached);
    assert!(result.integrity_markers.failure_attached);
    assert!(result
        .warnings
        .iter()
        .any(|warning| warning.code == "rollback"));
    assert!(result
        .warnings
        .iter()
        .any(|warning| warning.code == "failure"));
    assert!(result.decision_log.records.iter().any(|record| {
        matches!(
            record.detail,
            DecisionDetail::Rollback { ref reason }
                if reason == "poisoned transaction rollback"
        )
    }));
    assert!(result.decision_log.records.iter().any(|record| {
        matches!(
            record.detail,
            DecisionDetail::Failure {
                phase: ExecutionFailurePhase::Rollback,
                ..
            }
        )
    }));
}

#[test]
fn poisoned_rollback_rewinds_graph() {
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
    let invalid = crate::data::handle::NodeId::new(999_999, 0);
    assert!(tx.mark_dirty(invalid, ASPECT_B).is_err());
    assert_eq!(tx.rollback().unwrap().outcome, TransactionOutcome::Poisoned);
    assert_eq!(runtime.graph().get_state(a).unwrap(), before);
}

#[test]
fn poisoned_rollback_does_not_increment_explicit_rollback_metric() {
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
    let rollback_before = runtime.telemetry().transaction.transaction_rollback_count;
    let poison_before = runtime.telemetry().transaction.transaction_poison_count;
    let mut tx = runtime.begin(request_execution, &mut ctx);

    let invalid = crate::data::handle::NodeId::new(999_999, 0);
    assert!(tx.mark_dirty(invalid, ASPECT_B).is_err());
    assert_eq!(tx.rollback().unwrap().outcome, TransactionOutcome::Poisoned);

    assert_eq!(
        runtime.telemetry().transaction.transaction_rollback_count,
        rollback_before,
        "poisoned rollback should not inflate explicit rollback telemetry"
    );
    assert_eq!(
        runtime.telemetry().transaction.transaction_poison_count,
        poison_before + 1
    );
}

#[test]
fn failure_during_event_begin_rewinds_graph() {
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
        .subscribe(Box::new(NeedsMissingProviderSubscriber))
        .unwrap();

    let mut ctx = ();
    let mut tx = runtime.begin(request_execution, &mut ctx);
    tx.mark_dirty(a, ASPECT_B).unwrap();
    let err = tx.commit().unwrap_err();
    assert!(format!("{err}").contains("event bus begin failed"));
    assert_eq!(runtime.graph().get_state(a).unwrap(), before);
}

#[test]
fn commit_failure_reports_rolled_back_patch_count() {
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
    tx.emit_event(Ev::Tick);
    tx.flush_events(CheckpointBarrier::PerOperation).unwrap();

    let _ = tx.commit().unwrap_err();
    let rollback = runtime
        .observe()
        .diagnostics()
        .latest_rollback()
        .expect("rollback diagnostics should be recorded");
    assert!(
        rollback.staged_node_patch_count > 0,
        "rollback diagnostics should retain the staged patch count after rewinding"
    );
}
