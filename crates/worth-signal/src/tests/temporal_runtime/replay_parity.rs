use crate::facade::{
    temporal_certification_record, Aspect, AspectVersion, ClockAdvanceRequest, ClockDomain,
    ClockTick, EvaluationRequestMode, NodeEvaluationResult, SignalError, SignalGraph,
    SignalRuntime, TemporalCertificationFamily, TemporalCondition, TemporalReplayMismatchClass,
    TemporalWakeRetirementReason,
};

#[test]
fn temporal_replay_parity_report_compares_canonical_temporal_digests() {
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
    let node = runtime.graph_mut().node().after(4).unwrap().build();
    let aspect = Aspect::new(6);

    let expected = runtime
        .transaction(request_execution, &mut (), |tx| {
            tx.evaluate_with_plan(
                node,
                &|_ctx| {
                    Ok::<NodeEvaluationResult, SignalError>(NodeEvaluationResult::from_version(
                        AspectVersion::from_updates([(aspect, 1)]),
                    ))
                },
                EvaluationRequestMode::Default,
            )?;
            Ok(())
        })
        .unwrap()
        .reconstructability;
    let replayed = expected.clone();

    let parity = runtime.temporal_replay_parity_report(&expected, &replayed);
    assert!(parity.parity);
    assert!(parity.mismatch_classes.is_empty());
    assert_eq!(
        runtime
            .telemetry()
            .temporal
            .temporal_replay_parity_check_count,
        1
    );

    let mut drifted = expected.clone();
    drifted.temporal.ready_wake_digest = "drifted-ready-wake-digest".to_owned();
    drifted.temporal.certification_digest = "drifted-certification-digest".to_owned();
    let mismatch = runtime.temporal_replay_parity_report(&expected, &drifted);

    assert!(!mismatch.parity);
    assert!(mismatch
        .mismatch_classes
        .contains(&TemporalReplayMismatchClass::ReadyWakeDigestMismatch));
    assert!(mismatch
        .mismatch_classes
        .contains(&TemporalReplayMismatchClass::CertificationDigestMismatch));
    assert_eq!(
        runtime
            .telemetry()
            .temporal
            .temporal_replay_parity_check_count,
        2
    );
}

#[test]
fn temporal_replay_parity_survives_snapshot_restore_of_ready_frontier() {
    let mut runtime = SignalRuntime::builder(SignalGraph::new())
        .with_kernel_defaults()
        .build();
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
    let snapshot = runtime
        .capture_snapshot()
        .expect("snapshot capture should succeed without managed queue bindings");
    let expected = snapshot
        .reconstructability
        .clone()
        .expect("snapshot should carry temporal reconstructability");

    runtime
        .retire_temporal_wake(wake.id(), TemporalWakeRetirementReason::Consumed)
        .unwrap();
    runtime.restore_snapshot(&snapshot).unwrap();
    let restored = runtime
        .capture_snapshot()
        .expect("snapshot capture should succeed without managed queue bindings");
    let replayed = restored
        .reconstructability
        .clone()
        .expect("restored snapshot should carry temporal reconstructability");

    let parity = runtime.temporal_replay_parity_report(&expected, &replayed);
    assert!(parity.parity, "{:?}", parity.mismatch_classes);
    let record = temporal_certification_record(
        TemporalCertificationFamily::TemporalBranchRestoreEquivalence,
        replayed.temporal.clone(),
        Some(parity),
    );
    assert!(record.passed);
    assert_eq!(
        record.family,
        TemporalCertificationFamily::TemporalBranchRestoreEquivalence
    );
    assert_eq!(runtime.temporal_wake_summary().ready_count(), 1);
    assert_eq!(
        runtime.temporal_frontier_snapshot().next_ready_wake_id(),
        Some(wake.id())
    );
}
