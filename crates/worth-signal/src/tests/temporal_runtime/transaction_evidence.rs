use crate::facade::{
    Aspect, AspectVersion, ClockAdvanceRequest, ClockDomain, ClockTick, EvaluationRequestMode,
    NodeEvaluationResult, RequiredDerivedRebuildSet, SignalError, SignalGraph, SignalRuntime,
    TemporalEligibilityAuthority, TemporalReconstructabilityArtifact, TemporalReplayMismatchClass,
    TemporalWakeRetirementReason,
};
use std::sync::atomic::{AtomicU32, Ordering};

#[test]
fn transaction_temporal_evidence_freezes_wake_and_reconstructability_artifacts() {
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
    let node = runtime.graph_mut().node().after(5).unwrap().build();
    let aspect = Aspect::new(5);
    let calls = AtomicU32::new(0);

    let deferred = runtime
        .transaction(request_execution, &mut (), |tx| {
            tx.evaluate_with_plan(
                node,
                &|_ctx| {
                    calls.fetch_add(1, Ordering::Relaxed);
                    Ok::<NodeEvaluationResult, SignalError>(NodeEvaluationResult::from_version(
                        AspectVersion::from_updates([(aspect, 1)]),
                    ))
                },
                EvaluationRequestMode::Default,
            )?;
            Ok(())
        })
        .unwrap();

    assert_eq!(calls.load(Ordering::Relaxed), 0);
    assert!(deferred.temporal_evidence.has_temporal_facts());
    assert_eq!(deferred.temporal_evidence.scheduled_wakes.len(), 1);
    assert_eq!(deferred.temporal_evidence.eligibility_facts.len(), 1);
    assert_eq!(deferred.reconstructability.temporal.scheduled_wake_count, 1);
    assert_eq!(
        deferred.reconstructability.temporal.eligibility_fact_count,
        1
    );
    assert_eq!(
        deferred
            .reconstructability
            .temporal
            .wake_summary
            .scheduled_count(),
        1
    );
    assert!(
        deferred
            .reconstructability
            .proof()
            .required_rebuild
            .iter()
            .any(|requirement| matches!(requirement, RequiredDerivedRebuildSet::TemporalState(_))),
        "temporal state must be an explicit reconstructability rebuild surface"
    );
    assert_ne!(
        deferred.reconstructability.temporal.certification_digest,
        TemporalReconstructabilityArtifact::default().certification_digest
    );

    runtime
        .advance_clock(ClockAdvanceRequest::new(
            ClockDomain::MonotonicExecution,
            ClockTick::new(5),
        ))
        .unwrap();
    let admitted = runtime
        .transaction(request_execution, &mut (), |tx| {
            tx.evaluate_with_plan(
                node,
                &|_ctx| {
                    calls.fetch_add(1, Ordering::Relaxed);
                    Ok::<NodeEvaluationResult, SignalError>(NodeEvaluationResult::from_version(
                        AspectVersion::from_updates([(aspect, 2)]),
                    ))
                },
                EvaluationRequestMode::Default,
            )?;
            Ok(())
        })
        .unwrap();

    assert_eq!(calls.load(Ordering::Relaxed), 1);
    assert_eq!(admitted.temporal_evidence.ready_wakes.len(), 1);
    assert_eq!(admitted.temporal_evidence.retired_wakes.len(), 1);
    assert_eq!(admitted.temporal_evidence.eligibility_facts.len(), 1);
    assert_eq!(admitted.reconstructability.temporal.ready_wake_count, 1);
    assert_eq!(admitted.reconstructability.temporal.retired_wake_count, 1);
    assert_eq!(
        admitted
            .temporal_evidence
            .eligibility_facts
            .first()
            .unwrap()
            .eligibility
            .authority(),
        TemporalEligibilityAuthority::RuntimeScheduledWake
    );
}

#[test]
fn transaction_debounce_burst_records_supersession_evidence_and_digest() {
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

    let first = runtime
        .transaction(request_execution, &mut (), |tx| {
            tx.evaluate_with_plan(
                node,
                &|_ctx| {
                    Ok::<NodeEvaluationResult, SignalError>(NodeEvaluationResult::from_version(
                        AspectVersion::from_updates([(Aspect::new(1), 1)]),
                    ))
                },
                EvaluationRequestMode::Default,
            )?;
            Ok(())
        })
        .unwrap();
    assert_eq!(first.temporal_evidence.scheduled_wakes.len(), 1);
    assert_eq!(first.temporal_evidence.rescheduled_wakes.len(), 0);

    runtime
        .advance_clock(ClockAdvanceRequest::new(
            ClockDomain::MonotonicExecution,
            ClockTick::new(2),
        ))
        .unwrap();
    let rescheduled = runtime
        .transaction(request_execution, &mut (), |tx| {
            tx.evaluate_with_plan(
                node,
                &|_ctx| {
                    Ok::<NodeEvaluationResult, SignalError>(NodeEvaluationResult::from_version(
                        AspectVersion::from_updates([(Aspect::new(1), 2)]),
                    ))
                },
                EvaluationRequestMode::Default,
            )?;
            Ok(())
        })
        .unwrap();

    assert_eq!(rescheduled.temporal_evidence.rescheduled_wakes.len(), 1);
    let supersession = &rescheduled.temporal_evidence.rescheduled_wakes[0];
    assert_eq!(
        supersession.retired().reason(),
        TemporalWakeRetirementReason::Superseded
    );
    assert_eq!(supersession.scheduled().due_tick(), ClockTick::new(7));
    assert_eq!(
        rescheduled
            .reconstructability
            .temporal
            .rescheduled_wake_count,
        1
    );
    assert_ne!(
        rescheduled
            .reconstructability
            .temporal
            .rescheduled_wake_digest,
        TemporalReconstructabilityArtifact::default().rescheduled_wake_digest
    );

    let mut drifted = rescheduled.reconstructability.clone();
    drifted.temporal.rescheduled_wake_digest = "reschedule-drift".to_owned();
    drifted.temporal.certification_digest = "certification-drift".to_owned();
    let parity = runtime.temporal_replay_parity_report(&rescheduled.reconstructability, &drifted);
    assert!(!parity.parity);
    assert!(parity
        .mismatch_classes
        .contains(&TemporalReplayMismatchClass::RescheduledWakeDigestMismatch));
}

#[test]
fn transaction_throttle_burst_records_reuse_evidence_and_digest() {
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
    let node = runtime.graph_mut().node().throttle(5).unwrap().build();

    let first = runtime
        .transaction(request_execution, &mut (), |tx| {
            tx.evaluate_with_plan(
                node,
                &|_ctx| {
                    Ok::<NodeEvaluationResult, SignalError>(NodeEvaluationResult::from_version(
                        AspectVersion::from_updates([(Aspect::new(2), 1)]),
                    ))
                },
                EvaluationRequestMode::Default,
            )?;
            Ok(())
        })
        .unwrap();
    assert_eq!(first.temporal_evidence.scheduled_wakes.len(), 1);
    assert_eq!(first.temporal_evidence.reused_wakes.len(), 0);

    runtime
        .advance_clock(ClockAdvanceRequest::new(
            ClockDomain::MonotonicExecution,
            ClockTick::new(2),
        ))
        .unwrap();
    let reused = runtime
        .transaction(request_execution, &mut (), |tx| {
            tx.evaluate_with_plan(
                node,
                &|_ctx| {
                    Ok::<NodeEvaluationResult, SignalError>(NodeEvaluationResult::from_version(
                        AspectVersion::from_updates([(Aspect::new(2), 2)]),
                    ))
                },
                EvaluationRequestMode::Default,
            )?;
            Ok(())
        })
        .unwrap();

    assert_eq!(reused.temporal_evidence.reused_wakes.len(), 1);
    let reuse = &reused.temporal_evidence.reused_wakes[0];
    assert_eq!(reuse.original_due_tick(), ClockTick::new(5));
    assert_eq!(reuse.attempted_due_tick(), ClockTick::new(7));
    assert_eq!(reuse.decision_tick(), ClockTick::new(2));
    assert_eq!(runtime.telemetry().temporal.wake_reuse_count, 1);
    assert_eq!(reused.reconstructability.temporal.reused_wake_count, 1);
    assert_ne!(
        reused.reconstructability.temporal.reused_wake_digest,
        TemporalReconstructabilityArtifact::default().reused_wake_digest
    );

    let mut drifted = reused.reconstructability.clone();
    drifted.temporal.reused_wake_digest = "reuse-drift".to_owned();
    drifted.temporal.certification_digest = "certification-drift".to_owned();
    let parity = runtime.temporal_replay_parity_report(&reused.reconstructability, &drifted);
    assert!(!parity.parity);
    assert!(parity
        .mismatch_classes
        .contains(&TemporalReplayMismatchClass::ReusedWakeDigestMismatch));
}
