use super::runtime_world::{build_runtime, Ev};
use crate::data::checkpoint::CheckpointBarrier;
use crate::facade::{AuthorityPolicy, DecisionDetail, DecisionRecord, EvaluationRequestMode};
use crate::logic::transaction::TransactionOutcome;
use crate::tests::support::{version_ab, ASPECT_B};

#[test]
fn begin_commit_applies_staged_state_once() {
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
    tx.emit_event(Ev::Tick);
    tx.flush_events(CheckpointBarrier::PerOperation).unwrap();
    assert_eq!(tx.commit().unwrap().outcome, TransactionOutcome::Committed);
    assert_eq!(runtime.telemetry().transaction.transaction_commit_count, 1);
}

#[test]
fn commit_result_without_evaluation_reports_empty_execution_summary() {
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
    tx.emit_event(Ev::Tick);
    tx.flush_events(CheckpointBarrier::PerOperation).unwrap();

    let result = tx.commit().unwrap();
    assert_eq!(result.outcome, TransactionOutcome::Committed);
    assert!(result.execution_report.is_none());
    assert_eq!(result.evaluation_summary.nodes_evaluated, 0);
    assert_eq!(result.evaluation_summary.nodes_recomputed, 0);
    assert_eq!(result.evaluation_summary.nodes_suppressed, 0);
    assert_eq!(result.evaluation_summary.plans_built, 0);
    assert_eq!(result.evaluation_summary.stages_executed, 0);
    assert_eq!(result.rollback, None);
    assert!(result.decision_summary.committed);
    assert!(!result.decision_summary.rollback_recorded);
    assert!(result.integrity_markers.event_epochs_attached);
    assert_eq!(result.event_epochs.len(), 1);
    assert!(result.timing.total_nanos >= result.timing.commit_nanos);
}

#[test]
fn commit_result_with_evaluation_carries_execution_summary() {
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

    let mut ctx = ();
    let mut tx = runtime.begin(request_execution, &mut ctx);
    tx.evaluate_with_plan(
        source,
        &|view| Ok(view.finish(version_ab(1, 0))),
        EvaluationRequestMode::Default,
    )
    .unwrap();

    let result = tx.commit().unwrap();
    assert_eq!(result.outcome, TransactionOutcome::Committed);
    assert!(result.execution_report.is_some());
    assert!(result.evaluation_summary.nodes_evaluated >= 1);
    assert!(result.evaluation_summary.nodes_recomputed >= 1);
    assert!(result.evaluation_summary.plans_built >= 1);
    assert!(result.evaluation_summary.stages_executed >= 1);
    assert!(result.timing.evaluation_nanos > 0);
    assert!(result.decision_summary.stage_authority_decisions >= 1);
    assert!(result.integrity_markers.execution_report_attached);
    assert_eq!(result.decision_log.records.len(), 3);
    assert!(matches!(
        result.decision_log.records[0].detail,
        DecisionDetail::TransactionOutcome {
            outcome: TransactionOutcome::Committed
        }
    ));
    assert!(matches!(
        result.decision_log.records[1],
        DecisionRecord {
            stage_index: Some(0),
            detail: DecisionDetail::StageAuthorityPolicy {
                authority_policy: AuthorityPolicy::AuthoritativeOnly
            }
        }
    ));
    assert!(matches!(
        result.decision_log.records[2],
        DecisionRecord {
            stage_index: Some(0),
            detail: DecisionDetail::StageParallelAdmission { .. }
        }
    ));
}
