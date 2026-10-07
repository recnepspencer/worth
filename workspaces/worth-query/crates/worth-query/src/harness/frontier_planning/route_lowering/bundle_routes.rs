use crate::execution::{execute_preflight_bundle, execute_serial_fallback_route};
use crate::frontier_signal_adapter::SignalFrontierSurfaceEvidence;
use crate::planning::{
    admit_bounded_materialization_frontier_preflight, lower_execution_preflight_to_frontier_plan,
    lower_preflight_to_serial_fallback_route, FrontierPredictionDriftOutcome,
    FrontierRoutePlanningError, FrontierSurfaceDigest, SerialFallbackEvidence,
    SerialFallbackReason,
};

use super::super::fixtures::{sample_signal_execution_receipt, sample_signal_planning_estimate};

#[test]
fn denied_by_drift_blocks_serial_route_lowering() {
    let bounded = crate::harness::fixtures::execution_preflights::ordered_collection_preflight();
    let bounded_admitted = admit_bounded_materialization_frontier_preflight(bounded.clone())
        .expect("bounded materialization should admit");
    let denied = lower_preflight_to_serial_fallback_route(
        &bounded_admitted,
        &SerialFallbackEvidence::from_surface(
            bounded.basis().proof().digest().as_str(),
            FrontierSurfaceDigest::from_label("denied-by-drift-serial"),
            SerialFallbackReason::PredictionDriftRequiresSerialRoute,
            FrontierPredictionDriftOutcome::DeniedByDrift,
        ),
    )
    .expect_err("denied-by-drift must block serial fallback route lowering");

    match denied {
        FrontierRoutePlanningError::PredictionDriftDenied { .. } => {}
        other => panic!("expected prediction drift denial, got {other:?}"),
    }
}

#[test]
fn serial_fallback_required_allows_explicit_serial_fallback() {
    let bounded = crate::harness::fixtures::execution_preflights::ordered_collection_preflight();
    let bounded_admitted = admit_bounded_materialization_frontier_preflight(bounded.clone())
        .expect("bounded materialization should admit");
    let route = lower_preflight_to_serial_fallback_route(
        &bounded_admitted,
        &SerialFallbackEvidence::from_surface(
            bounded.basis().proof().digest().as_str(),
            FrontierSurfaceDigest::from_label("serial-fallback-required-serial"),
            SerialFallbackReason::PredictionDriftRequiresSerialRoute,
            FrontierPredictionDriftOutcome::SerialFallbackRequired,
        ),
    )
    .expect("serial fallback required should still allow serial route");
    assert_eq!(
        route.report().drift_outcome(),
        &FrontierPredictionDriftOutcome::SerialFallbackRequired
    );
    assert_eq!(
        route.reason(),
        &SerialFallbackReason::PredictionDriftRequiresSerialRoute
    );
    assert_eq!(route.report().serial_fallback_reason(), route.reason());
    assert_eq!(route.counters().route_nonbudget_drift_posture_count(), 1);
}

#[test]
fn bounded_materialization_lowers_into_serial_fallback_route_with_typed_executor_entrypoint() {
    let preflight = crate::harness::fixtures::execution_preflights::ordered_collection_preflight();
    let admitted = admit_bounded_materialization_frontier_preflight(preflight.clone())
        .expect("bounded materialization should admit on the serial fallback frontier lane");
    let evidence = SerialFallbackEvidence::from_surface(
        preflight.basis().proof().digest().as_str(),
        FrontierSurfaceDigest::from_label("bounded-materialization-overlap"),
        SerialFallbackReason::DeterministicAdmissionDenied,
        FrontierPredictionDriftOutcome::WithinBudget,
    );

    let route = lower_preflight_to_serial_fallback_route(&admitted, &evidence)
        .expect("bounded materialization should lower into the serial fallback route");
    let typed = execute_serial_fallback_route(&route)
        .expect("serial fallback route entrypoint should execute");
    let baseline = execute_preflight_bundle(&preflight).expect("baseline execution should succeed");

    assert_eq!(typed.rows(), baseline.rows());
    assert_eq!(
        typed.report().result_digest(),
        baseline.report().result_digest()
    );
    assert_eq!(
        route.reason(),
        &SerialFallbackReason::DeterministicAdmissionDenied
    );
    assert_eq!(
        route.report().serial_fallback_reason(),
        &SerialFallbackReason::DeterministicAdmissionDenied
    );
    assert_eq!(
        route.report().route_surface_digest(),
        evidence.surface_digest()
    );
    assert_eq!(route.counters().route_serial_fallback_count(), 1);
}

#[test]
fn route_posture_digest_binds_frontier_evidence() {
    let preflight = crate::harness::fixtures::execution_preflights::ordered_collection_preflight();
    let admitted = admit_bounded_materialization_frontier_preflight(preflight.clone())
        .expect("bounded materialization should admit on the serial frontier lane");
    let first_evidence = SerialFallbackEvidence::from_surface(
        preflight.basis().proof().digest().as_str(),
        FrontierSurfaceDigest::from_label("frontier-a"),
        SerialFallbackReason::SerialExecutor,
        FrontierPredictionDriftOutcome::WithinBudget,
    );
    let second_evidence = SerialFallbackEvidence::from_surface(
        preflight.basis().proof().digest().as_str(),
        FrontierSurfaceDigest::from_label("frontier-b"),
        SerialFallbackReason::SerialExecutor,
        FrontierPredictionDriftOutcome::WithinBudget,
    );

    let first = lower_preflight_to_serial_fallback_route(&admitted, &first_evidence)
        .expect("first frontier surface should admit");
    let second = lower_preflight_to_serial_fallback_route(&admitted, &second_evidence)
        .expect("second frontier surface should admit");

    assert_ne!(
        first.posture_digest(),
        second.posture_digest(),
        "route posture must bind explicit frontier evidence instead of family-only heuristics"
    );
}

#[test]
fn signal_planning_estimate_adapter_produces_stable_surface_digest() {
    let estimate = sample_signal_planning_estimate();

    let first = SignalFrontierSurfaceEvidence::from_planning_estimate(&estimate);
    let second = SignalFrontierSurfaceEvidence::from_planning_estimate(&estimate);

    assert_eq!(first.surface_digest(), second.surface_digest());
    assert_eq!(first.predicted_breadth(), second.predicted_breadth());
    assert_eq!(first.realized_breadth(), None);
}

#[test]
fn signal_frontier_surface_evidence_lowers_into_route_evidence() {
    let preflight = crate::harness::fixtures::execution_preflights::ordered_collection_preflight();
    let admitted = admit_bounded_materialization_frontier_preflight(preflight.clone())
        .expect("bounded materialization should admit on the serial frontier lane");
    let signal_surface =
        SignalFrontierSurfaceEvidence::from_execution_receipt(&sample_signal_execution_receipt());
    let evidence = signal_surface.to_serial_fallback_evidence(
        preflight.basis().proof().digest().as_str(),
        SerialFallbackReason::SerialExecutor,
        FrontierPredictionDriftOutcome::WithinBudget,
    );

    let route = lower_preflight_to_serial_fallback_route(&admitted, &evidence)
        .expect("signal-backed frontier evidence should lower the bounded materialization route");

    let frontier_plan = lower_execution_preflight_to_frontier_plan(&preflight)
        .expect("preflight should lower into a frontier plan");
    let expected_posture = crate::identity::hash_parts(&[
        format!(
            "frontier_plan_posture:{}",
            frontier_plan.report().posture_digest().as_str()
        ),
        format!(
            "evidence_basis:{}",
            preflight.basis().proof().digest().as_str()
        ),
        format!(
            "frontier_surface:{}",
            signal_surface.surface_digest().as_str()
        ),
        "drift_outcome:within_budget".into(),
        "serial_fallback_reason:serial_executor".into(),
    ]);
    assert_eq!(route.posture_digest().as_str(), expected_posture);
    assert_eq!(
        route.report().route_surface_digest(),
        signal_surface.surface_digest()
    );
    assert_eq!(
        route.report().route_surface_digest(),
        evidence.surface_digest()
    );
    assert_eq!(route.posture_digest(), route.report().posture_digest());
    assert!(signal_surface.realized_breadth().is_some());
}
