use crate::data::telemetry::TemporalTelemetry;
use crate::facade::{
    temporal_replay_parity_report, Aspect, AspectVersion, ClockAdvanceRequest, ClockDomain,
    ClockTick, DiagnosticsLevel, EvaluationRequestMode, IntervalCondition, MissedTickPolicy,
    NodeEvaluationResult, ReplaySlice, SignalBranchId, SignalError, SignalGraph, SignalRuntime,
    SignalSnapshotId, TemporalCertificationBundle, TemporalCondition, TemporalDiagnosticsSummary,
    TemporalReconstructabilityArtifact, TemporalReplayParityReport, TemporalTransactionEvidence,
    TemporalWakeRetirementReason,
};
type TestRuntime = SignalRuntime<(), (), (), (), ()>;

#[derive(Debug, Clone)]
pub(super) struct TemporalPhase9MixedBranchState {
    pub(super) branch_id: SignalBranchId,
    pub(super) restored_snapshot_id: SignalSnapshotId,
    pub(super) head_snapshot_before_restore: Option<SignalSnapshotId>,
    pub(super) head_snapshot_after_restore: Option<SignalSnapshotId>,
    pub(super) replay_before_restore: ReplaySlice,
    pub(super) replay_after_snapshot_drift: ReplaySlice,
    pub(super) replay_after_restore: ReplaySlice,
    pub(super) temporal_telemetry_after_restore: TemporalTelemetry,
    pub(super) reconstructability_before_restore: TemporalReconstructabilityArtifact,
    pub(super) reconstructability_after_snapshot_drift: TemporalReconstructabilityArtifact,
    pub(super) reconstructability_after_restore: TemporalReconstructabilityArtifact,
    pub(super) restore_parity: TemporalReplayParityReport,
}

#[derive(Debug, Clone)]
pub(super) struct TemporalPhase9MixedWorkloadOutcome {
    pub(super) bundle: TemporalCertificationBundle,
    pub(super) eligibility_artifact: TemporalReconstructabilityArtifact,
    pub(super) eligibility_parity: TemporalReplayParityReport,
    pub(super) boundedness_artifact: TemporalReconstructabilityArtifact,
    pub(super) previous_value_artifact: TemporalReconstructabilityArtifact,
    pub(super) feature: TemporalPhase9MixedBranchState,
    pub(super) sibling: TemporalPhase9MixedBranchState,
    pub(super) diagnostics_operational: TemporalDiagnosticsSummary,
    pub(super) diagnostics_forensic: TemporalDiagnosticsSummary,
    pub(super) temporal_telemetry: TemporalTelemetry,
}

fn exercise_temporal_phase9_hostile_suffix_on_active_branch(
    runtime: &mut TestRuntime,
) -> TemporalPhase9MixedBranchState {
    runtime
        .advance_clock(ClockAdvanceRequest::new(
            ClockDomain::MonotonicExecution,
            ClockTick::new(5),
        ))
        .unwrap();
    let ready = runtime.promote_due_temporal_wakes_ready().unwrap();
    assert!(
        ready.len() >= 3,
        "hostile branch suffix should admit after, throttle, and interval wakes at tick 5"
    );

    let snapshot = runtime
        .capture_snapshot()
        .expect("snapshot capture should succeed without managed queue bindings");
    let restored_snapshot_id = snapshot.meta().snapshot_id;
    let reconstructability_before_restore = snapshot
        .reconstructability
        .clone()
        .expect("temporal hostile branch snapshot should carry reconstructability");
    let head_snapshot_before_restore = runtime
        .observe()
        .branch_head_snapshot_id(runtime.observe().current_branch().id);
    let replay_before_restore = runtime
        .observe()
        .replay_for_branch(runtime.observe().current_branch().id);

    for wake in ready.iter().take(2) {
        runtime
            .retire_temporal_wake(wake.id(), TemporalWakeRetirementReason::Consumed)
            .unwrap();
    }
    runtime
        .advance_clock(ClockAdvanceRequest::new(
            ClockDomain::MonotonicExecution,
            ClockTick::new(9),
        ))
        .unwrap();
    let reconstructability_after_snapshot_drift = runtime
        .capture_snapshot()
        .expect("snapshot capture should succeed without managed queue bindings")
        .reconstructability
        .expect("post-drift temporal snapshot should carry reconstructability")
        .temporal;
    let replay_after_snapshot_drift = runtime
        .observe()
        .replay_for_branch(runtime.observe().current_branch().id);

    runtime.restore_snapshot(&snapshot).unwrap();
    let head_snapshot_after_restore = runtime
        .observe()
        .branch_head_snapshot_id(runtime.observe().current_branch().id);
    let replay_after_restore = runtime
        .observe()
        .replay_for_branch(runtime.observe().current_branch().id);
    let reconstructability_after_restore = runtime
        .capture_snapshot()
        .expect("snapshot capture should succeed without managed queue bindings")
        .reconstructability
        .expect("restored temporal snapshot should carry reconstructability")
        .temporal;
    let restore_parity = temporal_replay_parity_report(
        &reconstructability_before_restore.temporal,
        &reconstructability_after_restore,
    );

    TemporalPhase9MixedBranchState {
        branch_id: runtime.observe().current_branch().id,
        restored_snapshot_id,
        head_snapshot_before_restore,
        head_snapshot_after_restore,
        replay_before_restore,
        replay_after_snapshot_drift,
        replay_after_restore,
        temporal_telemetry_after_restore: runtime.telemetry().temporal,
        reconstructability_before_restore: reconstructability_before_restore.temporal,
        reconstructability_after_snapshot_drift,
        reconstructability_after_restore,
        restore_parity,
    }
}

pub(super) fn temporal_phase9_mixed_workload() -> TemporalPhase9MixedWorkloadOutcome {
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
    let output_aspect = Aspect::new(3);
    let source = runtime.graph_mut().node().build();
    runtime
        .transaction(request_execution, &mut (), |tx| {
            tx.read(source, &|view| {
                Ok(view.finish(
                    NodeEvaluationResult::from_version(AspectVersion::from_updates([(
                        output_aspect,
                        41,
                    )]))
                    .with_output_identity("phase9-temporal-previous"),
                ))
            })?;
            Ok(())
        })
        .unwrap();

    let after = runtime.graph_mut().node().after(3).unwrap().build();
    let debounce = runtime.graph_mut().node().debounce(5).unwrap().build();
    let throttle = runtime.graph_mut().node().throttle(5).unwrap().build();
    let stale_after = runtime.graph_mut().node().stale_after(7).unwrap().build();
    let interval = runtime.graph_mut().node().interval(5).unwrap().build();
    let temporal_nodes = [after, debounce, throttle, stale_after, interval];

    let initial = runtime
        .transaction(request_execution, &mut (), |tx| {
            for node in temporal_nodes {
                tx.evaluate_with_plan(
                    node,
                    &|_ctx| {
                        Ok::<NodeEvaluationResult, SignalError>(NodeEvaluationResult::from_version(
                            AspectVersion::from_updates([(output_aspect, 1)]),
                        ))
                    },
                    EvaluationRequestMode::Default,
                )?;
            }
            Ok(())
        })
        .unwrap();
    assert_eq!(initial.temporal_summary.deferred_count(), 5);
    assert_eq!(initial.temporal_summary.resolver_fallback_count(), 0);
    assert_eq!(initial.temporal_evidence.scheduled_wakes.len(), 5);
    assert_eq!(initial.temporal_evidence.eligibility_facts.len(), 5);
    let eligibility_artifact = initial.reconstructability.temporal.clone();
    let eligibility_parity = runtime
        .temporal_replay_parity_report(&initial.reconstructability, &initial.reconstructability);

    runtime
        .advance_clock(ClockAdvanceRequest::new(
            ClockDomain::MonotonicExecution,
            ClockTick::new(2),
        ))
        .unwrap();
    let burst = runtime
        .transaction(request_execution, &mut (), |tx| {
            tx.evaluate_with_plan(
                debounce,
                &|_ctx| {
                    Ok::<NodeEvaluationResult, SignalError>(NodeEvaluationResult::from_version(
                        AspectVersion::from_updates([(output_aspect, 2)]),
                    ))
                },
                EvaluationRequestMode::Default,
            )?;
            tx.evaluate_with_plan(
                throttle,
                &|_ctx| {
                    Ok::<NodeEvaluationResult, SignalError>(NodeEvaluationResult::from_version(
                        AspectVersion::from_updates([(output_aspect, 3)]),
                    ))
                },
                EvaluationRequestMode::Default,
            )?;
            Ok(())
        })
        .unwrap();
    assert_eq!(burst.temporal_evidence.rescheduled_wakes.len(), 1);
    assert_eq!(burst.temporal_evidence.reused_wakes.len(), 1);
    assert_eq!(
        runtime.temporal_wake_summary().scheduled_count(),
        5,
        "debounce supersession and throttle reuse must not widen the live frontier"
    );

    let main = runtime.current_branch();
    let feature = runtime.create_branch("phase9-temporal-feature").unwrap();
    let sibling = runtime.create_branch("phase9-temporal-sibling").unwrap();

    runtime.switch_branch(feature.clone()).unwrap();
    let feature = exercise_temporal_phase9_hostile_suffix_on_active_branch(&mut runtime);

    runtime.switch_branch(sibling.clone()).unwrap();
    let sibling = exercise_temporal_phase9_hostile_suffix_on_active_branch(&mut runtime);

    runtime.switch_branch(main).unwrap();
    let collapse = IntervalCondition::try_new(5)
        .unwrap()
        .with_missed_tick_policy(MissedTickPolicy::CollapseToOne);
    let skip = IntervalCondition::try_new(5)
        .unwrap()
        .with_missed_tick_policy(MissedTickPolicy::SkipToLatest);
    let catch_up = IntervalCondition::try_new(5)
        .unwrap()
        .with_missed_tick_policy(MissedTickPolicy::CatchUpAll);
    let bounded_wakes = [
        runtime
            .schedule_temporal_wake(TemporalCondition::interval(collapse), ClockTick::new(10))
            .unwrap(),
        runtime
            .schedule_temporal_wake(TemporalCondition::interval(skip), ClockTick::new(10))
            .unwrap(),
        runtime
            .schedule_temporal_wake(TemporalCondition::interval(catch_up), ClockTick::new(10))
            .unwrap(),
    ];
    runtime
        .advance_clock(ClockAdvanceRequest::new(
            ClockDomain::MonotonicExecution,
            ClockTick::new(1_010),
        ))
        .unwrap();
    runtime.promote_due_temporal_wakes_ready().unwrap();
    let mut wake_boundedness_evidence = TemporalTransactionEvidence::default();
    wake_boundedness_evidence.clock_basis = runtime.clock_basis();
    for wake in bounded_wakes {
        let regeneration = runtime.regenerate_interval_wake(wake.id()).unwrap();
        wake_boundedness_evidence
            .interval_regenerations
            .push(regeneration);
    }
    let boundedness_artifact = TemporalReconstructabilityArtifact::from_evidence(
        runtime.temporal_wake_summary(),
        &wake_boundedness_evidence,
    );

    let previous_wake = runtime
        .schedule_temporal_wake(TemporalCondition::after(1).unwrap(), ClockTick::new(1_011))
        .unwrap();
    runtime
        .advance_clock(ClockAdvanceRequest::new(
            ClockDomain::MonotonicExecution,
            ClockTick::new(1_011),
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
    let previous_value_artifact = TemporalReconstructabilityArtifact::from_evidence(
        runtime.temporal_wake_summary(),
        &previous_evidence,
    );

    let bundle = runtime
        .temporal_certification_builder()
        .with_temporal_eligibility_replay_parity(
            eligibility_artifact.clone(),
            eligibility_parity.clone(),
        )
        .unwrap()
        .with_temporal_branch_restore_equivalence(
            feature.reconstructability_after_restore.clone(),
            feature.restore_parity.clone(),
        )
        .unwrap()
        .with_temporal_wake_boundedness(boundedness_artifact.clone())
        .unwrap()
        .with_previous_value_time_gated_equivalence(previous_value_artifact.clone())
        .unwrap()
        .build()
        .unwrap();

    let diagnostics_operational = runtime
        .observe()
        .temporal_diagnostics_summary(DiagnosticsLevel::Operational);
    let diagnostics_forensic = runtime
        .observe()
        .temporal_diagnostics_summary(DiagnosticsLevel::Forensic);

    TemporalPhase9MixedWorkloadOutcome {
        bundle,
        eligibility_artifact,
        eligibility_parity,
        boundedness_artifact,
        previous_value_artifact,
        feature,
        sibling,
        diagnostics_operational,
        diagnostics_forensic,
        temporal_telemetry: runtime.telemetry().temporal,
    }
}
