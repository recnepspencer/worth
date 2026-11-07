use super::super::contracts::HistoricalPathComplexityContract;
use super::super::contracts::HistoricalPathReuseDescriptor;
use super::super::cost::{HistoricalPathCostPosture, PerformancePredictionDriftOutcome};
use super::super::report::HistoricalPathVocabularyReport;
use super::super::request::HistoricalEvaluationRequest;
use super::super::{
    AdmittedHistoricalPathClass, RequestedHistoricalPathClass, ResolvedHistoricalPathClass,
};

#[test]
fn closed_enum_names_are_stable() {
    assert_eq!(
        RequestedHistoricalPathClass::RequestedRetainedSnapshotPath.as_str(),
        "requested_retained_snapshot_path"
    );
    assert_eq!(
        AdmittedHistoricalPathClass::AdmittedDeltaReplayPath.as_str(),
        "admitted_delta_replay_path"
    );
    assert_eq!(
        ResolvedHistoricalPathClass::ResolvedFullReconstructionPath.as_str(),
        "resolved_full_reconstruction_path"
    );
    assert_eq!(
        HistoricalPathCostPosture::HistoricalReplayBounded.as_str(),
        "historical_replay_bounded"
    );
    assert_eq!(
        PerformancePredictionDriftOutcome::HistoricalReplaySpanDrift.as_str(),
        "historical_replay_span_drift"
    );
}

#[test]
fn requested_admitted_and_resolved_path_classes_are_distinct() {
    let requested = RequestedHistoricalPathClass::RequestedRetainedSnapshotPath;
    let admitted = AdmittedHistoricalPathClass::AdmittedRetainedSnapshotPath;
    let resolved = ResolvedHistoricalPathClass::ResolvedRetainedSnapshotPath;

    assert_ne!(requested.as_str(), admitted.as_str());
    assert_ne!(admitted.as_str(), resolved.as_str());
    assert_ne!(requested.as_str(), resolved.as_str());
}

#[test]
fn complexity_contract_names_are_deterministic() {
    assert_eq!(
        HistoricalPathComplexityContract::retained_path().contract_name(),
        "historical_retained_path"
    );
    assert_eq!(
        HistoricalPathComplexityContract::replay_path().contract_name(),
        "historical_replay_path"
    );
    assert_eq!(
        HistoricalPathComplexityContract::reconstruction_path().contract_name(),
        "historical_reconstruction_path"
    );
}

#[test]
fn vocabulary_report_preserves_requested_family_and_posture() {
    let request = HistoricalEvaluationRequest::full_reconstruction_for_test(
        "basis:reconstruction",
        5,
        9,
        HistoricalPathReuseDescriptor::no_reuse(),
    )
    .expect("fixture digest must fit");
    let report = HistoricalPathVocabularyReport::from_request(
        &request,
        HistoricalPathComplexityContract::reconstruction_path(),
        super::super::counters::HistoricalCounterSnapshot::vocabulary_baseline(),
    );

    assert_eq!(
        report.requested_path_class_name(),
        "requested_full_reconstruction_path"
    );
    assert_eq!(
        report.cost_posture().as_str(),
        "historical_reconstruction_expensive"
    );
}
