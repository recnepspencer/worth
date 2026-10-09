use crate::facade::{
    temporal_certification_record, Aspect, AspectVersion, ClockAdvanceRequest, ClockDomain,
    ClockTick, EvaluationRequestMode, IntervalCondition, MissedTickPolicy, NodeEvaluationResult,
    OutputIdentity, PreviousValueRevision, SignalError, SignalGraph, SignalRuntime,
    TemporalCertificationFamily, TemporalCondition, TEMPORAL_CERTIFICATION_BUNDLE_SCHEMA_VERSION,
};
use std::sync::atomic::{AtomicU32, Ordering};

#[test]
fn temporal_wake_boundedness_certification_family_covers_large_interval_jumps() {
    let mut runtime = SignalRuntime::builder(SignalGraph::new())
        .with_kernel_defaults()
        .build();
    let collapse = IntervalCondition::try_new(5)
        .unwrap()
        .with_missed_tick_policy(MissedTickPolicy::CollapseToOne);
    let skip = IntervalCondition::try_new(5)
        .unwrap()
        .with_missed_tick_policy(MissedTickPolicy::SkipToLatest);
    let catch_up = IntervalCondition::try_new(5)
        .unwrap()
        .with_missed_tick_policy(MissedTickPolicy::CatchUpAll);
    runtime
        .schedule_temporal_wake(TemporalCondition::interval(collapse), ClockTick::new(5))
        .unwrap();
    runtime
        .schedule_temporal_wake(TemporalCondition::interval(skip), ClockTick::new(5))
        .unwrap();
    runtime
        .schedule_temporal_wake(TemporalCondition::interval(catch_up), ClockTick::new(5))
        .unwrap();
    runtime
        .advance_clock(ClockAdvanceRequest::new(
            ClockDomain::MonotonicExecution,
            ClockTick::new(1_005),
        ))
        .unwrap();

    let promoted = runtime.promote_due_temporal_wakes_ready().unwrap();
    assert_eq!(promoted.len(), 3);
    let mut regenerations = Vec::new();
    for ready in promoted {
        regenerations.push(runtime.regenerate_interval_wake(ready.id()).unwrap());
    }
    let due_ticks = regenerations
        .iter()
        .map(|regeneration| regeneration.scheduled().due_tick())
        .collect::<Vec<_>>();
    assert!(due_ticks.contains(&ClockTick::new(1_010)));
    assert!(due_ticks.contains(&ClockTick::new(1_005)));
    assert!(due_ticks.contains(&ClockTick::new(10)));
    assert_eq!(
        runtime
            .telemetry()
            .temporal
            .interval_wake_regeneration_count,
        3
    );
    assert_eq!(
        runtime.telemetry().temporal.missed_interval_count,
        399,
        "large interval jumps should be charged to missed-tick policy outcomes, not hidden loops"
    );
    assert_eq!(
        runtime
            .telemetry()
            .temporal
            .temporal_broad_scan_denial_count,
        1
    );

    let snapshot = runtime
        .capture_snapshot()
        .expect("snapshot capture should succeed without managed queue bindings");
    let record = temporal_certification_record(
        TemporalCertificationFamily::TemporalWakeBoundedness,
        snapshot.reconstructability.unwrap().temporal,
        None,
    );
    assert!(record.passed);
    assert_eq!(
        record.family,
        TemporalCertificationFamily::TemporalWakeBoundedness
    );
}

#[test]
fn stale_after_expires_without_upstream_writes_under_runtime_owned_time() {
    let mut runtime = SignalRuntime::builder(SignalGraph::new())
        .with_kernel_defaults()
        .build();
    let node = runtime.graph_mut().node().stale_after(4).unwrap().build();
    let calls = AtomicU32::new(0);

    let deferred = runtime
        .evaluate_with_plan(
            node,
            &(),
            &|_ctx| {
                calls.fetch_add(1, Ordering::Relaxed);
                Ok::<NodeEvaluationResult, SignalError>(NodeEvaluationResult::from_version(
                    AspectVersion::from_updates([(Aspect::new(4), 1)]),
                ))
            },
            EvaluationRequestMode::Default,
        )
        .unwrap();
    assert_eq!(deferred.tasks_deferred_by_condition, 1);
    assert_eq!(calls.load(Ordering::Relaxed), 0);

    runtime
        .advance_clock(ClockAdvanceRequest::new(
            ClockDomain::MonotonicExecution,
            ClockTick::new(4),
        ))
        .unwrap();
    let admitted = runtime
        .evaluate_with_plan(
            node,
            &(),
            &|_ctx| {
                calls.fetch_add(1, Ordering::Relaxed);
                Ok::<NodeEvaluationResult, SignalError>(NodeEvaluationResult::from_version(
                    AspectVersion::from_updates([(Aspect::new(4), 2)]),
                ))
            },
            EvaluationRequestMode::Default,
        )
        .unwrap();
    assert_eq!(admitted.tasks_executed, 1);
    assert_eq!(calls.load(Ordering::Relaxed), 1);
    assert_eq!(admitted.temporal_summary.resolver_fallback_count(), 0);
}

#[test]
fn previous_value_time_gated_equivalence_certification_family_captures_committed_lineage() {
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
    let source = runtime.graph_mut().node().build();
    let value_aspect = Aspect::new(5);
    runtime
        .transaction(request_execution, &mut (), |tx| {
            tx.read(source, &|view| {
                Ok(view.finish(
                    NodeEvaluationResult::from_version(AspectVersion::from_updates([(
                        value_aspect,
                        7,
                    )]))
                    .with_output_identity("previous-value-equivalence"),
                ))
            })?;
            Ok(())
        })
        .unwrap();
    let wake = runtime
        .schedule_temporal_wake(TemporalCondition::after(2).unwrap(), ClockTick::new(2))
        .unwrap();
    runtime
        .advance_clock(ClockAdvanceRequest::new(
            ClockDomain::MonotonicExecution,
            ClockTick::new(2),
        ))
        .unwrap();
    runtime.promote_due_temporal_wakes_ready().unwrap();
    let access = runtime
        .grant_temporal_previous_value_access(wake.id())
        .unwrap();
    let reference = runtime.previous_temporal_value(&access, source).unwrap();

    assert_eq!(reference.revision(), PreviousValueRevision::new(1));
    assert_eq!(
        reference.output_identity().map(OutputIdentity::as_str),
        Some("previous-value-equivalence")
    );
    let snapshot = runtime
        .capture_snapshot()
        .expect("snapshot capture should succeed without managed queue bindings");
    let record = temporal_certification_record(
        TemporalCertificationFamily::PreviousValueTimeGatedEquivalence,
        snapshot.reconstructability.unwrap().temporal,
        None,
    );
    assert!(record.passed);
    assert_eq!(
        runtime.telemetry().temporal.previous_value_reference_count,
        1
    );
}

#[test]
fn temporal_certification_bundle_accepts_complete_required_family_set() {
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
    let node = runtime.graph_mut().node().after(2).unwrap().build();
    let outcome = runtime
        .transaction(request_execution, &mut (), |tx| {
            tx.evaluate_with_plan(
                node,
                &|_ctx| {
                    Ok::<NodeEvaluationResult, SignalError>(NodeEvaluationResult::from_version(
                        AspectVersion::from_updates([(Aspect::new(6), 1)]),
                    ))
                },
                EvaluationRequestMode::Default,
            )?;
            Ok(())
        })
        .unwrap();
    let parity = runtime
        .temporal_replay_parity_report(&outcome.reconstructability, &outcome.reconstructability);
    let artifact = outcome.reconstructability.temporal.clone();

    let bundle = runtime.temporal_certification_bundle([
        temporal_certification_record(
            TemporalCertificationFamily::TemporalEligibilityReplayParity,
            artifact.clone(),
            Some(parity),
        ),
        temporal_certification_record(
            TemporalCertificationFamily::TemporalBranchRestoreEquivalence,
            artifact.clone(),
            None,
        ),
        temporal_certification_record(
            TemporalCertificationFamily::TemporalWakeBoundedness,
            artifact.clone(),
            None,
        ),
        temporal_certification_record(
            TemporalCertificationFamily::PreviousValueTimeGatedEquivalence,
            artifact,
            None,
        ),
    ]);

    assert!(bundle.passed, "{:?}", bundle.failures);
    bundle.ensure_passed().unwrap();
    assert_eq!(
        bundle.schema_version,
        TEMPORAL_CERTIFICATION_BUNDLE_SCHEMA_VERSION
    );
    assert_eq!(bundle.records.len(), 4);
    assert_eq!(bundle.summary.required_family_count, 4);
    assert_eq!(bundle.summary.provided_record_count, 4);
    assert_eq!(bundle.summary.failed_family_count, 0);
    assert_eq!(bundle.summary.missing_family_count, 0);
    assert_eq!(bundle.summary.duplicate_family_count, 0);
    assert!(!bundle.bundle_digest.is_empty());
}
