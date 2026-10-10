use worth_runtime_bridge::facade::{
    BridgeDeliveryIntent, BridgeDiagnosticsTier, BridgeReplayMode, BridgeRuntimePolicy,
    BridgeTruthViewEvaluationRequest, BridgeTruthViewSelector, HistoricalEvaluationDeclaration,
    TruthBranchIdentity, TruthCommitIdentity, TruthSnapshotIdentity,
};

use super::super::bridge_lowering::{
    lower_materialization_from_artifact, lower_materialization_from_decision_log,
    lower_policy_resolution,
};
use super::super::contracts::HistoricalPathReuseDescriptor;
use super::super::planner::{
    admit_historical_evaluation_path, materialization_metadata_from_resolved,
    resolve_historical_materialization_path,
};
use super::super::request::HistoricalEvaluationRequest;
use super::super::{
    AdmittedHistoricalPathClass, HistoricalPathCompatibilityOutcome, RequestedHistoricalPathClass,
    ResolvedHistoricalPathClass,
};
use super::runtime_bridge_fixture::runtime;

#[test]
fn retained_snapshot_request_admits_and_resolves_retained_path() {
    let host_request = crate::bridge_host_request::serial_request();
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime(BridgeRuntimePolicy::default());
    let declaration = HistoricalEvaluationDeclaration::new(
        BridgeTruthViewSelector::branch_snapshot(
            TruthBranchIdentity::from_bridge_harness_label("analysis"),
            TruthSnapshotIdentity::from_bridge_harness_label("snapshot-a"),
        ),
        BridgeReplayMode::Disabled,
        BridgeDiagnosticsTier::Standard,
        BridgeDeliveryIntent::PrepareSignalEvaluation,
    );
    let request = HistoricalEvaluationRequest::retained_snapshot_for_test(
        declaration
            .declaration_identity()
            .bridge_admission_evidence()
            .terminal_projection_for_reporting(),
        1,
        1,
        HistoricalPathReuseDescriptor::retained_reuse(),
    );
    let capability = lower_policy_resolution(
        &declaration,
        &runtime.resolve_truth_view_policy(&declaration),
        None,
        request.requested_path_class(),
    )
    .expect("snapshot policy should lower");

    let admission =
        admit_historical_evaluation_path(request, capability).expect("retained path should admit");

    assert_eq!(
        admission.compatibility_outcome(),
        &HistoricalPathCompatibilityOutcome::Admitted
    );
    assert_eq!(
        admission.admitted_path().admitted_path_class(),
        &AdmittedHistoricalPathClass::AdmittedRetainedSnapshotPath
    );
    assert_eq!(
        admission.cost_posture().as_str(),
        "historical_retained_fast_path"
    );
    assert_eq!(
        admission.complexity_contract().contract_name(),
        "historical_retained_path"
    );

    let evaluation = runtime
        .evaluate(
            BridgeTruthViewEvaluationRequest::for_branch_snapshot(
                TruthBranchIdentity::from_bridge_harness_label("analysis"),
                TruthSnapshotIdentity::from_bridge_harness_label("snapshot-a"),
            )
            .with_replay_mode(BridgeReplayMode::Disabled),
            resource_request,
        )
        .expect("snapshot evaluation should succeed");
    let lowered = lower_materialization_from_artifact(
        &runtime.lower_historical_evaluation_artifact(evaluation.observation()),
        &RequestedHistoricalPathClass::RequestedRetainedSnapshotPath,
    )
    .expect("materialized artifact should lower");

    let resolved = resolve_historical_materialization_path(admission, lowered)
        .expect("retained path should resolve");
    let metadata = materialization_metadata_from_resolved(resolved.clone());

    assert_eq!(
        resolved.resolved_path_class(),
        &ResolvedHistoricalPathClass::ResolvedRetainedSnapshotPath
    );
    assert_eq!(
        metadata.requested_path_class(),
        &RequestedHistoricalPathClass::RequestedRetainedSnapshotPath
    );
    assert_eq!(
        metadata.admitted_path_class(),
        &AdmittedHistoricalPathClass::AdmittedRetainedSnapshotPath
    );
    assert_eq!(
        metadata.resolved_path_class(),
        &ResolvedHistoricalPathClass::ResolvedRetainedSnapshotPath
    );
    assert_eq!(
        resolved
            .counters()
            .history_work_avoided_by_retained_path_count(),
        0
    );
}

#[test]
fn replay_request_admits_and_resolves_replay_path() {
    let host_request = crate::bridge_host_request::serial_request();
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime(BridgeRuntimePolicy::default());
    let declaration = HistoricalEvaluationDeclaration::new(
        BridgeTruthViewSelector::historical_commit(
            TruthBranchIdentity::from_bridge_harness_label("analysis"),
            TruthCommitIdentity::from_bridge_harness_label("commit-a"),
        ),
        BridgeReplayMode::Required,
        BridgeDiagnosticsTier::Standard,
        BridgeDeliveryIntent::PrepareSignalEvaluation,
    );
    let request = HistoricalEvaluationRequest::delta_replay_for_test(
        declaration
            .declaration_identity()
            .bridge_admission_evidence()
            .terminal_projection_for_reporting(),
        4,
        8,
        HistoricalPathReuseDescriptor::with_replay_tail_reuse(),
    );
    let capability = lower_policy_resolution(
        &declaration,
        &runtime.resolve_truth_view_policy(&declaration),
        None,
        request.requested_path_class(),
    )
    .expect("historical commit policy should lower");

    let admission =
        admit_historical_evaluation_path(request, capability).expect("replay path should admit");
    assert_eq!(
        admission.admitted_path().admitted_path_class(),
        &AdmittedHistoricalPathClass::AdmittedDeltaReplayPath
    );
    assert_eq!(
        admission.cost_posture().as_str(),
        "historical_replay_bounded"
    );

    let evaluation = runtime
        .evaluate(
            BridgeTruthViewEvaluationRequest::for_historical_commit(
                TruthBranchIdentity::from_bridge_harness_label("analysis"),
                TruthCommitIdentity::from_bridge_harness_label("commit-a"),
            )
            .with_replay_mode(BridgeReplayMode::Required),
            resource_request,
        )
        .expect("historical evaluation should succeed");
    let lowered = lower_materialization_from_decision_log(evaluation.record().decision_log())
        .expect("decision log should lower");
    let resolved = resolve_historical_materialization_path(admission, lowered)
        .expect("replay path should resolve");

    assert_eq!(
        resolved.resolved_path_class(),
        &ResolvedHistoricalPathClass::ResolvedDeltaReplayPath
    );
    assert_eq!(
        resolved.complexity_contract().contract_name(),
        "historical_replay_path"
    );
}

#[test]
fn reconstruction_request_admits_full_reconstruction_path() {
    let runtime = runtime(BridgeRuntimePolicy::default());
    let declaration = HistoricalEvaluationDeclaration::new(
        BridgeTruthViewSelector::branch_head(TruthBranchIdentity::from_bridge_harness_label(
            "analysis",
        )),
        BridgeReplayMode::Enabled,
        BridgeDiagnosticsTier::Standard,
        BridgeDeliveryIntent::PrepareSignalEvaluation,
    );
    let request = HistoricalEvaluationRequest::full_reconstruction_for_test(
        declaration
            .declaration_identity()
            .bridge_admission_evidence()
            .terminal_projection_for_reporting(),
        3,
        10,
        HistoricalPathReuseDescriptor::no_reuse(),
    );
    let capability = lower_policy_resolution(
        &declaration,
        &runtime.resolve_truth_view_policy(&declaration),
        None,
        request.requested_path_class(),
    )
    .expect("branch-head policy should lower");

    let admission = admit_historical_evaluation_path(request, capability)
        .expect("full reconstruction path should admit");

    assert_eq!(
        admission.admitted_path().admitted_path_class(),
        &AdmittedHistoricalPathClass::AdmittedFullReconstructionPath
    );
    assert_eq!(
        admission.complexity_contract().contract_name(),
        "historical_reconstruction_path"
    );
}

#[test]
fn replay_request_is_denied_when_replay_mode_is_not_admitted() {
    let runtime = runtime(BridgeRuntimePolicy::default());
    let declaration = HistoricalEvaluationDeclaration::new(
        BridgeTruthViewSelector::branch_snapshot(
            TruthBranchIdentity::from_bridge_harness_label("analysis"),
            TruthSnapshotIdentity::from_bridge_harness_label("snapshot-a"),
        ),
        BridgeReplayMode::Disabled,
        BridgeDiagnosticsTier::Standard,
        BridgeDeliveryIntent::PrepareSignalEvaluation,
    );
    let request = HistoricalEvaluationRequest::delta_replay_for_test(
        declaration
            .declaration_identity()
            .bridge_admission_evidence()
            .terminal_projection_for_reporting(),
        2,
        2,
        HistoricalPathReuseDescriptor::no_reuse(),
    );
    let capability = lower_policy_resolution(
        &declaration,
        &runtime.resolve_truth_view_policy(&declaration),
        None,
        request.requested_path_class(),
    )
    .expect("snapshot policy should lower");

    let error = admit_historical_evaluation_path(request, capability)
        .expect_err("replay should be denied when replay mode is disabled");

    assert_eq!(
        error.failure_class(),
        super::super::error::HistoricalEvaluationFailureClass::ReplayNotPermitted
    );
}

#[test]
fn admitted_path_class_must_match_requested_lane_proof() {
    let request = HistoricalEvaluationRequest::delta_replay_for_test(
        "basis:mismatch",
        2,
        2,
        HistoricalPathReuseDescriptor::no_reuse(),
    );
    let capability = super::super::request::HistoricalCapabilityDescriptor::new_for_test(
        "basis:mismatch",
        Some(AdmittedHistoricalPathClass::AdmittedRetainedSnapshotPath),
        true,
        false,
        true,
        false,
        HistoricalPathReuseDescriptor::no_reuse(),
    );

    let error = admit_historical_evaluation_path(request, capability)
        .expect_err("mismatched admitted proof should fail");

    assert_eq!(
        error.failure_class(),
        super::super::error::HistoricalEvaluationFailureClass::UnsupportedHistoricalPathRequest
    );
}
