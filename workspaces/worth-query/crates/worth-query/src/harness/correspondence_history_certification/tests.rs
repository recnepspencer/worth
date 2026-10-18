use super::{
    model::CorrespondenceHistoryFailureClass,
    MilestoneFivePointFourCorrespondenceHistoryCertificationAdapter,
};
use crate::harness::certification::HostileExpectation;

#[test]
fn correspondence_history_artifact_is_deterministic_and_offline_ready() {
    let left = MilestoneFivePointFourCorrespondenceHistoryCertificationAdapter::
        structural_correspondence_and_historical_materialization_path_artifact();
    let right = MilestoneFivePointFourCorrespondenceHistoryCertificationAdapter::
        structural_correspondence_and_historical_materialization_path_artifact();

    assert_eq!(
        left.certification_bundle_digest,
        right.certification_bundle_digest
    );
    assert_eq!(left.coverage_matrix_digest, right.coverage_matrix_digest);
    assert!(
        left.bundle_completeness_report
            .all_lanes_emit_required_outputs
    );
    assert_eq!(
        left.bundle_completeness_report.zero_rediscovery_lane_count,
        left.matrix.rows.len() * 3
    );
    assert!(left.bundle_completeness_report.offline_analysis_ready);
}

#[test]
fn correspondence_history_rows_preserve_expected_boundaries() {
    let matrix = MilestoneFivePointFourCorrespondenceHistoryCertificationAdapter::
        structural_correspondence_and_historical_materialization_path_test();

    let structural = matrix
        .rows
        .iter()
        .find(|row| row.row_name == "structural-correspondence-advisory")
        .expect("structural advisory row should exist");
    assert_eq!(
        structural.hostile_expectation,
        HostileExpectation::DistinctFromControl
    );
    assert_eq!(
        structural
            .control_lane
            .parity_bundle
            .parity_variant()
            .as_str(),
        "success"
    );
    assert_ne!(
        structural
            .control_lane
            .parity_bundle
            .correspondence_outcome_digest()
            .as_str(),
        structural
            .hostile_lane
            .parity_bundle
            .correspondence_outcome_digest()
            .as_str()
    );

    let replay = matrix
        .rows
        .iter()
        .find(|row| row.row_name == "historical-delta-replay-path")
        .expect("replay row should exist");
    assert_ne!(
        replay
            .control_lane
            .parity_bundle
            .resolved_path_digest()
            .expect("control path")
            .as_str(),
        replay
            .hostile_lane
            .parity_bundle
            .resolved_path_digest()
            .expect("hostile path")
            .as_str()
    );

    let correspondence_cost = matrix
        .rows
        .iter()
        .find(|row| row.row_name == "correspondence-cost-posture-parity")
        .expect("correspondence cost row should exist");
    assert_ne!(
        correspondence_cost
            .control_lane
            .parity_bundle
            .correspondence_cost_posture_digest()
            .as_str(),
        correspondence_cost
            .hostile_lane
            .parity_bundle
            .correspondence_cost_posture_digest()
            .as_str()
    );

    let historical_cost = matrix
        .rows
        .iter()
        .find(|row| row.row_name == "historical-cost-posture-parity")
        .expect("historical cost row should exist");
    assert_ne!(
        historical_cost
            .control_lane
            .parity_bundle
            .historical_cost_posture_digest()
            .expect("control posture")
            .as_str(),
        historical_cost
            .hostile_lane
            .parity_bundle
            .historical_cost_posture_digest()
            .expect("hostile posture")
            .as_str()
    );

    let drift = matrix
        .rows
        .iter()
        .find(|row| row.row_name == "prediction-drift-explicit")
        .expect("drift row should exist");
    assert_ne!(
        drift
            .control_lane
            .parity_bundle
            .performance_prediction_drift_outcome()
            .as_str(),
        "within_budget"
    );
}

#[test]
fn correspondence_history_rejections_bind_typed_failures() {
    let matrix = MilestoneFivePointFourCorrespondenceHistoryCertificationAdapter::
        structural_correspondence_and_historical_materialization_path_test();

    let substitution = matrix
        .rejection_rows
        .iter()
        .find(|row| row.row_name == "hidden-materialization-path-substitution-forbidden")
        .expect("substitution row should exist");
    assert_eq!(
        substitution.hostile_lane.failure_class,
        CorrespondenceHistoryFailureClass::HistoricalPathDenied
    );
    assert!(!substitution.hostile_lane.failure_digest.is_empty());
    assert!(substitution.hostile_lane.counter_snapshot_digest.is_some());

    let unsupported = matrix
        .rejection_rows
        .iter()
        .find(|row| row.row_name == "unsupported-correspondence-family")
        .expect("unsupported correspondence row should exist");
    assert_eq!(
        unsupported.hostile_lane.failure_class,
        CorrespondenceHistoryFailureClass::CorrespondenceDenied
    );
    assert!(unsupported.hostile_lane.counter_snapshot_digest.is_some());

    let host_cache = matrix
        .rejection_rows
        .iter()
        .find(|row| row.row_name == "host-cache-history-authority-forbidden")
        .expect("host cache authority row should exist");
    assert_eq!(
        host_cache.hostile_lane.failure_class,
        CorrespondenceHistoryFailureClass::HistoricalPathDenied
    );
    assert!(host_cache.hostile_lane.counter_snapshot_digest.is_some());
}
