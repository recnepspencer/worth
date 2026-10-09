use crate::facade::{
    temporal_certification_builder, temporal_certification_bundle, temporal_certification_record,
    temporal_replay_parity_report, Aspect, AspectVersion, ClockAdvanceRequest, ClockDomain,
    ClockTick, EvaluationRequestMode, IntervalCondition, MissedTickPolicy, NodeEvaluationResult,
    SignalError, SignalGraph, SignalRuntime, TemporalCertificationFailure,
    TemporalCertificationFamily, TemporalCertificationRecord, TemporalCondition,
    TemporalReconstructabilityArtifact, TemporalTransactionEvidence, TemporalWakeRetirementReason,
};

#[test]
fn temporal_certification_builder_requires_distinct_family_evidence_lanes() {
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

    let eligibility_node = runtime.graph_mut().node().after(2).unwrap().build();
    let eligibility_outcome = runtime
        .transaction(request_execution, &mut (), |tx| {
            tx.evaluate_with_plan(
                eligibility_node,
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
    let eligibility_parity = runtime.temporal_replay_parity_report(
        &eligibility_outcome.reconstructability,
        &eligibility_outcome.reconstructability,
    );
    let eligibility_artifact = eligibility_outcome.reconstructability.temporal.clone();

    let branch_wake = runtime
        .schedule_temporal_wake(TemporalCondition::after(3).unwrap(), ClockTick::new(3))
        .unwrap();
    runtime
        .advance_clock(ClockAdvanceRequest::new(
            ClockDomain::MonotonicExecution,
            ClockTick::new(3),
        ))
        .unwrap();
    runtime.promote_due_temporal_wakes_ready().unwrap();
    let snapshot = runtime
        .capture_snapshot()
        .expect("snapshot capture should succeed without managed queue bindings");
    let expected_restore = snapshot
        .reconstructability
        .clone()
        .expect("snapshot should carry temporal reconstructability");
    runtime
        .retire_temporal_wake(branch_wake.id(), TemporalWakeRetirementReason::Consumed)
        .unwrap();
    runtime.restore_snapshot(&snapshot).unwrap();
    let replayed_restore = runtime
        .capture_snapshot()
        .expect("snapshot capture should succeed without managed queue bindings")
        .reconstructability
        .expect("restored snapshot should carry temporal reconstructability");
    let restore_parity =
        runtime.temporal_replay_parity_report(&expected_restore, &replayed_restore);
    let restore_artifact = replayed_restore.temporal.clone();

    let interval = IntervalCondition::try_new(5)
        .unwrap()
        .with_missed_tick_policy(MissedTickPolicy::SkipToLatest);
    let interval_wake = runtime
        .schedule_temporal_wake(TemporalCondition::interval(interval), ClockTick::new(8))
        .unwrap();
    runtime
        .advance_clock(ClockAdvanceRequest::new(
            ClockDomain::MonotonicExecution,
            ClockTick::new(48),
        ))
        .unwrap();
    runtime.promote_due_temporal_wakes_ready().unwrap();
    let regeneration = runtime
        .regenerate_interval_wake(interval_wake.id())
        .unwrap();
    let mut wake_boundedness_evidence = TemporalTransactionEvidence::default();
    wake_boundedness_evidence.clock_basis = runtime.clock_basis();
    wake_boundedness_evidence
        .interval_regenerations
        .push(regeneration);
    let wake_boundedness_artifact = TemporalReconstructabilityArtifact::from_evidence(
        runtime.temporal_wake_summary(),
        &wake_boundedness_evidence,
    );

    let source = runtime.graph_mut().node().build();
    let value_aspect = Aspect::new(7);
    runtime
        .transaction(request_execution, &mut (), |tx| {
            tx.read(source, &|view| {
                Ok(view.finish(NodeEvaluationResult::from_version(
                    AspectVersion::from_updates([(value_aspect, 9)]),
                )))
            })?;
            Ok(())
        })
        .unwrap();
    let previous_wake = runtime
        .schedule_temporal_wake(TemporalCondition::after(1).unwrap(), ClockTick::new(49))
        .unwrap();
    runtime
        .advance_clock(ClockAdvanceRequest::new(
            ClockDomain::MonotonicExecution,
            ClockTick::new(49),
        ))
        .unwrap();
    runtime.promote_due_temporal_wakes_ready().unwrap();
    let previous_access = runtime
        .grant_temporal_previous_value_access(previous_wake.id())
        .unwrap();
    let previous_reference = runtime
        .previous_temporal_value(&previous_access, source)
        .unwrap();
    let mut previous_evidence = TemporalTransactionEvidence::default();
    previous_evidence.clock_basis = runtime.clock_basis();
    previous_evidence
        .previous_value_references
        .push(previous_reference);
    let previous_artifact = TemporalReconstructabilityArtifact::from_evidence(
        runtime.temporal_wake_summary(),
        &previous_evidence,
    );

    let bundle = runtime
        .temporal_certification_builder()
        .with_temporal_eligibility_replay_parity(eligibility_artifact, eligibility_parity)
        .unwrap()
        .with_temporal_branch_restore_equivalence(restore_artifact, restore_parity)
        .unwrap()
        .with_temporal_wake_boundedness(wake_boundedness_artifact)
        .unwrap()
        .with_previous_value_time_gated_equivalence(previous_artifact)
        .unwrap()
        .build()
        .unwrap();

    assert!(bundle.passed, "{:?}", bundle.failures);
    assert_eq!(bundle.records.len(), 4);
    assert_eq!(bundle.summary.passed_family_count, 4);
}

#[test]
fn temporal_certification_builder_rejects_missing_duplicate_and_synthetic_evidence() {
    let artifact = TemporalReconstructabilityArtifact::default();
    let parity = temporal_replay_parity_report(&artifact, &artifact);

    let missing_err = temporal_certification_builder().build().unwrap_err();
    assert!(format!("{missing_err}").contains("required certification family"));

    let synthetic_err = temporal_certification_builder()
        .with_temporal_eligibility_replay_parity(artifact.clone(), parity.clone())
        .unwrap_err();
    assert!(format!("{synthetic_err}").contains("default temporal artifact"));

    let mut eligibility_artifact = artifact.clone();
    eligibility_artifact.eligibility_fact_count = 1;
    eligibility_artifact.certification_digest = "non-default-eligibility".to_owned();
    let mut replayed = eligibility_artifact.clone();
    replayed.scheduled_wake_digest = "different-replayed-artifact".to_owned();
    let mismatched_parity = temporal_replay_parity_report(&eligibility_artifact, &replayed);
    let drift_err = temporal_certification_builder()
        .with_temporal_eligibility_replay_parity(replayed, mismatched_parity)
        .unwrap_err();
    assert!(format!("{drift_err}").contains("passing temporal replay parity"));

    let valid_parity = temporal_replay_parity_report(&eligibility_artifact, &eligibility_artifact);
    let duplicate_err = temporal_certification_builder()
        .with_temporal_eligibility_replay_parity(eligibility_artifact.clone(), valid_parity.clone())
        .unwrap()
        .with_temporal_eligibility_replay_parity(eligibility_artifact, valid_parity)
        .unwrap_err();
    assert!(format!("{duplicate_err}").contains("duplicate certification family"));
}

#[test]
fn temporal_certification_bundle_rejects_missing_duplicate_failed_empty_and_parity_drift() {
    let artifact = TemporalReconstructabilityArtifact::default();
    let mut drifted_artifact = artifact.clone();
    drifted_artifact.ready_wake_digest.push_str("-drift");
    let drift = temporal_replay_parity_report(&artifact, &drifted_artifact);
    let mut empty_digest_artifact = artifact.clone();
    empty_digest_artifact.certification_digest.clear();

    let bundle = temporal_certification_bundle([
        temporal_certification_record(
            TemporalCertificationFamily::TemporalEligibilityReplayParity,
            artifact.clone(),
            Some(drift),
        ),
        temporal_certification_record(
            TemporalCertificationFamily::TemporalEligibilityReplayParity,
            artifact.clone(),
            None,
        ),
        temporal_certification_record(
            TemporalCertificationFamily::TemporalBranchRestoreEquivalence,
            empty_digest_artifact,
            None,
        ),
        TemporalCertificationRecord {
            family: TemporalCertificationFamily::TemporalWakeBoundedness,
            artifact,
            parity: None,
            passed: false,
        },
    ]);

    assert!(!bundle.passed);
    assert!(format!("{}", bundle.ensure_passed().unwrap_err())
        .contains("temporal certification bundle failed"));
    assert!(bundle.failures.iter().any(|failure| matches!(
        failure,
        TemporalCertificationFailure::MissingRequiredFamily {
            family: TemporalCertificationFamily::PreviousValueTimeGatedEquivalence
        }
    )));
    assert!(bundle.failures.iter().any(|failure| matches!(
        failure,
        TemporalCertificationFailure::DuplicateFamily {
            family: TemporalCertificationFamily::TemporalEligibilityReplayParity,
            count: 2
        }
    )));
    assert!(bundle.failures.iter().any(|failure| matches!(
        failure,
        TemporalCertificationFailure::ParityMismatch {
            family: TemporalCertificationFamily::TemporalEligibilityReplayParity,
            ..
        }
    )));
    assert!(bundle.failures.iter().any(|failure| matches!(
        failure,
        TemporalCertificationFailure::FailedFamily {
            family: TemporalCertificationFamily::TemporalWakeBoundedness
        }
    )));
    assert!(bundle.failures.iter().any(|failure| matches!(
        failure,
        TemporalCertificationFailure::EmptyCertificationDigest {
            family: TemporalCertificationFamily::TemporalBranchRestoreEquivalence
        }
    )));
    assert_eq!(bundle.summary.provided_record_count, 4);
    assert_eq!(bundle.summary.missing_family_count, 1);
    assert_eq!(bundle.summary.duplicate_family_count, 1);
    assert_eq!(bundle.summary.failed_family_count, 4);
    assert!(!bundle.bundle_digest.is_empty());
}
