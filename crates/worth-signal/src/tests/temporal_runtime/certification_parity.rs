use crate::facade::{
    temporal_certification_bundle, temporal_certification_bundle_parity_report,
    temporal_certification_record, temporal_replay_parity_report, Aspect, AspectVersion,
    ClockAdvanceRequest, ClockDomain, ClockTick, EvaluationRequestMode, NodeEvaluationResult,
    SignalError, SignalGraph, SignalRuntime, TemporalCertificationBundle,
    TemporalCertificationBundleMismatchClass, TemporalCertificationFamily, TemporalCondition,
    TemporalReconstructabilityArtifact, TemporalReplayParityReport, TemporalWakeRetirementReason,
    TEMPORAL_CERTIFICATION_BUNDLE_PARITY_SCHEMA_VERSION,
};

fn complete_temporal_certification_bundle_for_artifact(
    artifact: TemporalReconstructabilityArtifact,
    parity: TemporalReplayParityReport,
) -> TemporalCertificationBundle {
    temporal_certification_bundle([
        temporal_certification_record(
            TemporalCertificationFamily::TemporalEligibilityReplayParity,
            artifact.clone(),
            Some(parity.clone()),
        ),
        temporal_certification_record(
            TemporalCertificationFamily::TemporalBranchRestoreEquivalence,
            artifact.clone(),
            Some(parity),
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
    ])
}

#[test]
fn temporal_branch_restore_equivalence_certifies_full_bundle_parity() {
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
    let replayed = runtime
        .capture_snapshot()
        .expect("snapshot capture should succeed without managed queue bindings")
        .reconstructability
        .expect("restored snapshot should carry temporal reconstructability");

    let artifact_parity = runtime.temporal_replay_parity_report(&expected, &replayed);
    assert!(
        artifact_parity.parity,
        "{:?}",
        artifact_parity.mismatch_classes
    );

    let expected_bundle = complete_temporal_certification_bundle_for_artifact(
        expected.temporal.clone(),
        artifact_parity.clone(),
    );
    let replayed_bundle =
        complete_temporal_certification_bundle_for_artifact(replayed.temporal, artifact_parity);
    expected_bundle.ensure_passed().unwrap();
    replayed_bundle.ensure_passed().unwrap();

    let bundle_parity =
        runtime.temporal_certification_bundle_parity_report(&expected_bundle, &replayed_bundle);
    assert!(bundle_parity.parity, "{:?}", bundle_parity.mismatch_classes);
    assert_eq!(
        bundle_parity.proof_schema_version,
        TEMPORAL_CERTIFICATION_BUNDLE_PARITY_SCHEMA_VERSION
    );
    assert_eq!(
        bundle_parity.expected.bundle_digest,
        bundle_parity.replayed.bundle_digest
    );
    assert_eq!(
        runtime
            .telemetry()
            .temporal
            .temporal_replay_parity_check_count,
        2
    );
}

#[test]
fn temporal_certification_bundle_parity_detects_bundle_record_drift() {
    let artifact = TemporalReconstructabilityArtifact::default();
    let parity = temporal_replay_parity_report(&artifact, &artifact);
    let expected =
        complete_temporal_certification_bundle_for_artifact(artifact.clone(), parity.clone());
    let mut drifted = complete_temporal_certification_bundle_for_artifact(artifact, parity);
    drifted.bundle_digest = "drifted-temporal-certification-bundle".to_owned();
    drifted.records[0].artifact.certification_digest = "drifted-record".to_owned();

    let report = temporal_certification_bundle_parity_report(&expected, &drifted);
    assert!(!report.parity);
    assert!(report
        .mismatch_classes
        .contains(&TemporalCertificationBundleMismatchClass::BundleDigestMismatch));
    assert!(report
        .mismatch_classes
        .contains(&TemporalCertificationBundleMismatchClass::RecordSetMismatch));
}

#[test]
fn temporal_eligibility_replay_parity_certification_family_records_exact_digest_match() {
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
    let node = runtime.graph_mut().node().after(3).unwrap().build();
    let outcome = runtime
        .transaction(request_execution, &mut (), |tx| {
            tx.evaluate_with_plan(
                node,
                &|_ctx| {
                    Ok::<NodeEvaluationResult, SignalError>(NodeEvaluationResult::from_version(
                        AspectVersion::from_updates([(Aspect::new(3), 1)]),
                    ))
                },
                EvaluationRequestMode::Default,
            )?;
            Ok(())
        })
        .unwrap();

    let parity = runtime
        .temporal_replay_parity_report(&outcome.reconstructability, &outcome.reconstructability);
    let record = temporal_certification_record(
        TemporalCertificationFamily::TemporalEligibilityReplayParity,
        outcome.reconstructability.temporal.clone(),
        Some(parity),
    );

    assert!(record.passed);
    assert_eq!(
        record.family,
        TemporalCertificationFamily::TemporalEligibilityReplayParity
    );
    assert_eq!(record.artifact.eligibility_fact_count, 1);
    assert_eq!(
        runtime
            .telemetry()
            .temporal
            .temporal_replay_parity_check_count,
        1
    );
}
