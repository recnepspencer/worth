use crate::facade::{
    Aspect, AspectVersion, ClockAdvanceRequest, ClockDomain, ClockTick, EvaluationRequestMode,
    NodeEvaluationResult, SignalError, SignalGraph, SignalRuntime, TemporalCondition,
    TemporalReconstructabilityArtifact, TemporalWakeOwner, TemporalWakeRetirementReason,
};
use std::sync::atomic::{AtomicU32, Ordering};

#[test]
fn stale_ready_owned_wake_is_superseded_before_temporal_lowering() {
    let mut runtime = SignalRuntime::builder(SignalGraph::new())
        .with_kernel_defaults()
        .build();
    let node = runtime.graph_mut().node().throttle(5).unwrap().build();
    let stale = runtime
        .schedule_owned_temporal_wake(
            TemporalWakeOwner::Node(node),
            TemporalCondition::after(1).unwrap(),
            ClockTick::new(1),
        )
        .unwrap();
    runtime
        .advance_clock(ClockAdvanceRequest::new(
            ClockDomain::MonotonicExecution,
            ClockTick::new(1),
        ))
        .unwrap();
    runtime.promote_temporal_wake_ready(stale.id()).unwrap();
    let calls = AtomicU32::new(0);

    let report = runtime
        .evaluate_with_plan(
            node,
            &(),
            &|_ctx| {
                calls.fetch_add(1, Ordering::Relaxed);
                Ok::<NodeEvaluationResult, SignalError>(NodeEvaluationResult::from_version(
                    AspectVersion::from_updates([(Aspect::new(1), 1)]),
                ))
            },
            EvaluationRequestMode::Default,
        )
        .unwrap();

    assert_eq!(calls.load(Ordering::Relaxed), 0);
    assert_eq!(report.tasks_executed, 0);
    assert_eq!(report.tasks_deferred_by_condition, 1);
    assert_eq!(report.temporal_summary.resolver_fallback_count(), 0);
    assert_eq!(runtime.temporal_wake_summary().ready_count(), 0);
    assert_eq!(runtime.temporal_wake_summary().retired_count(), 1);
    assert_eq!(runtime.temporal_wake_summary().scheduled_count(), 1);
    assert_eq!(
        runtime.temporal_frontier_snapshot().next_due_tick(),
        Some(ClockTick::new(6))
    );
    assert_eq!(runtime.telemetry().temporal.rescheduled_wake_count, 1);
}

#[test]
fn transaction_stale_ready_policy_drift_records_supersession_evidence() {
    // This standalone caller declares the operational serial memory policy.
    let serial_request = worth_execution::SerialRequest::from_memory(
        worth_execution::SerialMemoryBudget::new(
            crate::runtime_policy::SignalRuntimePolicy::operational().serial_memory_bytes,
        ),
        worth_execution::CancellationToken::new(),
        None,
    );
    let request_execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let mut runtime = SignalRuntime::builder(SignalGraph::new())
        .with_kernel_defaults()
        .build();
    let node = runtime.graph_mut().node().debounce(5).unwrap().build();
    let stale = runtime
        .schedule_owned_temporal_wake(
            TemporalWakeOwner::Node(node),
            TemporalCondition::after(1).unwrap(),
            ClockTick::new(1),
        )
        .unwrap();
    runtime
        .advance_clock(ClockAdvanceRequest::new(
            ClockDomain::MonotonicExecution,
            ClockTick::new(1),
        ))
        .unwrap();
    runtime.promote_temporal_wake_ready(stale.id()).unwrap();
    let calls = AtomicU32::new(0);

    let outcome = runtime
        .transaction(request_execution, &mut (), |tx| {
            tx.evaluate_with_plan(
                node,
                &|_ctx| {
                    calls.fetch_add(1, Ordering::Relaxed);
                    Ok::<NodeEvaluationResult, SignalError>(NodeEvaluationResult::from_version(
                        AspectVersion::from_updates([(Aspect::new(6), 1)]),
                    ))
                },
                EvaluationRequestMode::Default,
            )?;
            Ok(())
        })
        .unwrap();

    assert_eq!(calls.load(Ordering::Relaxed), 0);
    assert_eq!(outcome.temporal_evidence.rescheduled_wakes.len(), 1);
    assert_eq!(outcome.temporal_evidence.retired_wakes.len(), 1);
    assert_eq!(outcome.temporal_evidence.scheduled_wakes.len(), 1);
    let supersession = &outcome.temporal_evidence.rescheduled_wakes[0];
    assert_eq!(supersession.retired().id(), stale.id());
    assert_eq!(
        supersession.retired().reason(),
        TemporalWakeRetirementReason::Superseded
    );
    assert!(matches!(
        supersession.scheduled().condition(),
        TemporalCondition::Debounce(_)
    ));
    assert_eq!(supersession.scheduled().due_tick(), ClockTick::new(6));
    assert_eq!(
        outcome.reconstructability.temporal.rescheduled_wake_count,
        1
    );
    assert_ne!(
        outcome.reconstructability.temporal.rescheduled_wake_digest,
        TemporalReconstructabilityArtifact::default().rescheduled_wake_digest
    );
}

#[test]
fn graph_only_sealed_temporal_execution_cannot_use_host_resolver_truth() {
    let mut graph = SignalGraph::new();
    let node = graph.node().after(5).unwrap().build();
    let plan = graph
        .build_evaluation_plan(&[node], EvaluationRequestMode::Default)
        .unwrap();

    let err = graph
        .execute_prepared_plan(&plan, &(), &|_ctx| {
            Ok::<NodeEvaluationResult, SignalError>(NodeEvaluationResult::from_version(
                AspectVersion::from_updates([(Aspect::new(4), 1)]),
            ))
        })
        .unwrap_err();

    assert!(
        format!("{err}").contains("runtime-owned temporal lowering"),
        "sealed temporal policies must not be admitted by graph-only host resolver truth"
    );
}
