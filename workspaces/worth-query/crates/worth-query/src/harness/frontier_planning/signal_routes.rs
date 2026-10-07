use crate::frontier_signal_adapter::{
    SignalAdmissionEvidenceError, SignalFrontierBundleEvidence, SignalFrontierSurfaceEvidence,
};
use crate::planning::{
    admit_bounded_materialization_frontier_preflight,
    lower_preflight_bundle_to_serial_fallback_routes, lower_preflight_to_serial_fallback_route,
    FrontierBundleRoutePlanningError, FrontierPredictionDriftOutcome, FrontierRoutePlanningError,
    SerialFallbackEvidence, SerialFallbackReason,
};
use worth_signal::facade::adapters::FrontierRouteEvidenceReason;

use super::fixtures::{
    runtime_signal_stage_execution_record, sample_signal_execution_receipt,
    sample_stage_execution_record,
};

#[test]
fn signal_stage_record_serial_reason_maps_to_serial_fallback_evidence() {
    let preflight = crate::harness::fixtures::execution_preflights::ordered_collection_preflight();
    let admitted = admit_bounded_materialization_frontier_preflight(preflight.clone())
        .expect("bounded materialization should admit on the serial fallback frontier lane");
    let signal_surface =
        SignalFrontierSurfaceEvidence::from_execution_receipt(&sample_signal_execution_receipt());
    let evidence = signal_surface.to_serial_fallback_evidence(
        preflight.basis().proof().digest().as_str(),
        SerialFallbackReason::SerialExecutor,
        FrontierPredictionDriftOutcome::WithinBudget,
    );

    let route = lower_preflight_to_serial_fallback_route(&admitted, &evidence)
        .expect("serial stage evidence should lower into the serial fallback route");

    assert_eq!(route.reason(), &SerialFallbackReason::SerialExecutor);
    assert_eq!(
        route.report().serial_fallback_reason(),
        &SerialFallbackReason::SerialExecutor
    );
}

#[test]
fn signal_stage_record_preserves_specific_serial_admission_reasons() {
    let preflight = crate::harness::fixtures::execution_preflights::ordered_collection_preflight();
    let admitted = admit_bounded_materialization_frontier_preflight(preflight.clone())
        .expect("bounded materialization should admit on the serial fallback frontier lane");
    let signal_surface =
        SignalFrontierSurfaceEvidence::from_execution_receipt(&sample_signal_execution_receipt());

    let cases = [
        (
            FrontierRouteEvidenceReason::BelowMinStageWidth,
            SerialFallbackReason::BelowMinStageWidth,
        ),
        (
            FrontierRouteEvidenceReason::BelowPolicyWorkThreshold,
            SerialFallbackReason::BelowPolicyWorkThreshold,
        ),
        (
            FrontierRouteEvidenceReason::ValidationHeavyStage,
            SerialFallbackReason::ValidationHeavyStage,
        ),
        (
            FrontierRouteEvidenceReason::BelowFullParallelThreshold,
            SerialFallbackReason::BelowFullParallelThreshold,
        ),
        (
            FrontierRouteEvidenceReason::FullParallelUnsupportedByMutableEngine,
            SerialFallbackReason::FullParallelUnsupportedByMutableEngine,
        ),
    ];

    for (signal_reason, query_reason) in cases {
        let stage = sample_stage_execution_record(signal_reason);
        let evidence = signal_surface
            .to_route_evidence_from_stage_record(
                preflight.basis().proof().digest().as_str(),
                &stage,
            )
            .expect("serial Signal receipt should preserve its admission reason");
        let route = lower_preflight_to_serial_fallback_route(&admitted, &evidence)
            .expect("specific serial reason should lower into fallback route");

        assert_eq!(route.reason(), &query_reason);
        assert_eq!(route.report().serial_fallback_reason(), &query_reason);
        assert_eq!(
            route.report().drift_outcome(),
            &FrontierPredictionDriftOutcome::WithinBudget
        );
    }
}

#[test]
fn signal_parallel_stage_is_rejected_as_serial_fallback_evidence() {
    let preflight = crate::harness::fixtures::execution_preflights::ordered_collection_preflight();
    let surface =
        SignalFrontierSurfaceEvidence::from_execution_receipt(&sample_signal_execution_receipt());
    let stage = sample_stage_execution_record(
        FrontierRouteEvidenceReason::AdmittedProofSafeGroupedConcurrent,
    );
    assert!(stage.is_parallel_admitted());

    assert_eq!(
        surface.to_route_evidence_from_stage_record(
            preflight.basis().proof().digest().as_str(),
            &stage,
        ),
        Err(SignalAdmissionEvidenceError::ParallelStageUnsupported)
    );
}

#[test]
fn signal_backed_route_evidence_rejects_basis_mismatch() {
    let preflight = crate::harness::fixtures::execution_preflights::ordered_collection_preflight();
    let admitted = admit_bounded_materialization_frontier_preflight(preflight.clone())
        .expect("bounded materialization should admit on the serial fallback frontier lane");
    let signal_surface =
        SignalFrontierSurfaceEvidence::from_execution_receipt(&sample_signal_execution_receipt());
    let evidence = signal_surface.to_serial_fallback_evidence(
        "wrong-basis-digest",
        SerialFallbackReason::SerialExecutor,
        FrontierPredictionDriftOutcome::WithinBudget,
    );

    let error = lower_preflight_to_serial_fallback_route(&admitted, &evidence)
        .expect_err("serial fallback route should reject frontier evidence bound to another basis");

    match error {
        FrontierRoutePlanningError::SerialFallbackUnavailable { .. } => {}
        other => panic!("expected serial fallback unavailability, got {other:?}"),
    }
}

#[test]
fn same_basis_bundle_lowers_into_signal_backed_serial_fallback_routes() {
    let first = admit_bounded_materialization_frontier_preflight(
        crate::harness::fixtures::execution_preflights::ordered_collection_preflight(),
    )
    .expect("bounded materialization should admit on the serial fallback frontier lane");
    let second = admit_bounded_materialization_frontier_preflight(
        crate::harness::fixtures::execution_preflights::ordered_collection_preflight(),
    )
    .expect("bounded materialization should admit on the serial fallback frontier lane");
    let route_surface =
        SignalFrontierSurfaceEvidence::from_execution_receipt(&sample_signal_execution_receipt());
    let bundle_evidence = SignalFrontierBundleEvidence::from_route_evidences(vec![
        route_surface.to_serial_fallback_evidence(
            first.as_preflight().basis().proof().digest().as_str(),
            SerialFallbackReason::SerialExecutor,
            FrontierPredictionDriftOutcome::WithinBudget,
        ),
        route_surface.to_serial_fallback_evidence(
            second.as_preflight().basis().proof().digest().as_str(),
            SerialFallbackReason::SerialExecutor,
            FrontierPredictionDriftOutcome::WithinBudget,
        ),
    ]);

    let bundle = lower_preflight_bundle_to_serial_fallback_routes(
        &[first.clone(), second.clone()],
        &bundle_evidence
            .bind_to_basis(first.as_preflight().basis().proof().digest().as_str())
            .expect("signal bundle evidence should bind to one shared basis"),
    )
    .expect("same-basis bounded bundle should lower into serial fallback routes");

    assert_eq!(
        bundle.bundle_basis_digest(),
        first.as_preflight().basis().proof().digest().as_str()
    );
    assert_eq!(bundle.routes().len(), 2);
    assert_eq!(
        bundle.routes()[0].reason(),
        &SerialFallbackReason::SerialExecutor
    );
    assert_eq!(
        bundle.routes()[1].reason(),
        &SerialFallbackReason::SerialExecutor
    );
    assert!(!bundle.bundle_posture_digest().as_str().is_empty());
    assert!(!bundle_evidence.bundle_surface_digest().as_str().is_empty());
}

#[test]
fn signal_backed_serial_bundle_rejects_evidence_count_mismatch() {
    let preflight = admit_bounded_materialization_frontier_preflight(
        crate::harness::fixtures::execution_preflights::ordered_collection_preflight(),
    )
    .expect("bounded materialization should admit on the serial fallback frontier lane");
    let signal_surface =
        SignalFrontierSurfaceEvidence::from_execution_receipt(&sample_signal_execution_receipt());
    let runtime_stage = runtime_signal_stage_execution_record();
    let bundle_evidence = SignalFrontierBundleEvidence::from_stage_records(
        preflight.as_preflight().basis().proof().digest().as_str(),
        &[signal_surface],
        &[runtime_stage],
    )
    .expect("single-route signal bundle evidence should compose");

    let error = lower_preflight_bundle_to_serial_fallback_routes(
        &[preflight.clone(), preflight],
        &bundle_evidence
            .bind_to_basis("signal-bundle-evidence")
            .expect("single-route signal bundle evidence should bind"),
    )
    .expect_err("bundle lowering should reject missing second evidence");

    assert_eq!(
        error,
        FrontierBundleRoutePlanningError::EvidenceCountMismatch {
            expected: 2,
            found: 1,
        }
    );
}

#[test]
fn signal_backed_serial_bundle_rejects_basis_mismatch() {
    let route_surface =
        SignalFrontierSurfaceEvidence::from_execution_receipt(&sample_signal_execution_receipt());
    let bundle_evidence = SignalFrontierBundleEvidence::from_route_evidences(vec![
        SerialFallbackEvidence::from_surface(
            "basis-a",
            route_surface.surface_digest().clone(),
            SerialFallbackReason::SerialExecutor,
            FrontierPredictionDriftOutcome::WithinBudget,
        ),
        SerialFallbackEvidence::from_surface(
            "basis-b",
            route_surface.surface_digest().clone(),
            SerialFallbackReason::SerialExecutor,
            FrontierPredictionDriftOutcome::WithinBudget,
        ),
    ]);

    let error = crate::frontier_planning::SerialFallbackBundleEvidence::from_routes(
        bundle_evidence.bundle_surface_digest().clone(),
        vec![
            SerialFallbackEvidence::from_surface(
                "basis-a",
                route_surface.surface_digest().clone(),
                SerialFallbackReason::SerialExecutor,
                FrontierPredictionDriftOutcome::WithinBudget,
            ),
            SerialFallbackEvidence::from_surface(
                "basis-b",
                route_surface.surface_digest().clone(),
                SerialFallbackReason::SerialExecutor,
                FrontierPredictionDriftOutcome::WithinBudget,
            ),
        ],
    )
    .expect_err("mixed route bases must be rejected before bundle evidence is admitted");

    match error {
        crate::frontier_planning::SerialFallbackBundleEvidenceError::MixedBasisDigest {
            expected_basis_digest,
            found_basis_digest,
        } => {
            assert_eq!(expected_basis_digest, "basis-a");
            assert_eq!(found_basis_digest, "basis-b");
        }
        other => panic!("expected mixed basis evidence denial, got {other:?}"),
    }
}

#[test]
fn signal_bundle_surface_digest_binds_member_basis_evidence() {
    let route_surface =
        SignalFrontierSurfaceEvidence::from_execution_receipt(&sample_signal_execution_receipt());
    let left = SignalFrontierBundleEvidence::from_route_evidences(vec![
        SerialFallbackEvidence::from_surface(
            "basis-a",
            route_surface.surface_digest().clone(),
            SerialFallbackReason::SerialExecutor,
            FrontierPredictionDriftOutcome::WithinBudget,
        ),
        SerialFallbackEvidence::from_surface(
            "basis-a",
            route_surface.surface_digest().clone(),
            SerialFallbackReason::SerialExecutor,
            FrontierPredictionDriftOutcome::WithinBudget,
        ),
    ]);
    let right = SignalFrontierBundleEvidence::from_route_evidences(vec![
        SerialFallbackEvidence::from_surface(
            "basis-a",
            route_surface.surface_digest().clone(),
            SerialFallbackReason::SerialExecutor,
            FrontierPredictionDriftOutcome::WithinBudget,
        ),
        SerialFallbackEvidence::from_surface(
            "basis-b",
            route_surface.surface_digest().clone(),
            SerialFallbackReason::SerialExecutor,
            FrontierPredictionDriftOutcome::WithinBudget,
        ),
    ]);

    assert_ne!(
        left.bundle_surface_digest(),
        right.bundle_surface_digest(),
        "bundle surface digest must bind member basis evidence, not only surface and fallback shape"
    );
}
