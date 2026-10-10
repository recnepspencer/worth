use worth_runtime_bridge::facade::{
    BridgeHistoricalMaterializationPath, BridgeReplayMode, BridgeRuntimePolicy,
    BridgeTruthViewEvaluationRequest, TruthBranchIdentity, TruthCommitIdentity,
};

use super::super::bridge_lowering::lower_materialization_from_decision_log;
use super::super::contracts::HistoricalPathReuseDescriptor;
use super::super::planner::{
    admit_historical_evaluation_path, resolve_historical_materialization_path,
};
use super::super::request::{HistoricalEvaluationRequest, HistoricalMaterializationDescriptor};
use super::super::{AdmittedHistoricalPathClass, ResolvedHistoricalPathClass};
use super::runtime_bridge_fixture::runtime;

#[test]
fn hidden_path_substitution_is_denied() {
    let runtime = runtime(BridgeRuntimePolicy::default());
    let declaration = worth_runtime_bridge::facade::HistoricalEvaluationDeclaration::new(
        worth_runtime_bridge::facade::BridgeTruthViewSelector::historical_commit(
            TruthBranchIdentity::from_bridge_harness_label("analysis"),
            TruthCommitIdentity::from_bridge_harness_label("commit-a"),
        ),
        BridgeReplayMode::Required,
        worth_runtime_bridge::facade::BridgeDiagnosticsTier::Standard,
        worth_runtime_bridge::facade::BridgeDeliveryIntent::PrepareSignalEvaluation,
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
    let capability = super::super::bridge_lowering::lower_policy_resolution(
        &declaration,
        &runtime.resolve_truth_view_policy(&declaration),
        None,
        request.requested_path_class(),
    )
    .expect("historical commit policy should lower");
    let admission =
        admit_historical_evaluation_path(request, capability).expect("replay path should admit");

    let wrong_path = HistoricalMaterializationDescriptor::new_for_test(
        declaration
            .declaration_identity()
            .bridge_admission_evidence()
            .terminal_projection_for_reporting(),
        ResolvedHistoricalPathClass::ResolvedFullReconstructionPath,
    );

    let error = resolve_historical_materialization_path(admission, wrong_path)
        .expect_err("hidden substitution should be denied");

    assert_eq!(
        error.failure_class(),
        super::super::error::HistoricalEvaluationFailureClass::HiddenPathSubstitutionDenied
    );
}

#[test]
fn resolved_historical_counters_preserve_admission_lane_and_metadata() {
    let request = HistoricalEvaluationRequest::delta_replay_for_test(
        "basis:counts",
        4,
        8,
        HistoricalPathReuseDescriptor::with_replay_tail_reuse(),
    );
    let capability = super::super::request::HistoricalCapabilityDescriptor::new_for_test(
        "basis:counts",
        Some(AdmittedHistoricalPathClass::AdmittedDeltaReplayPath),
        true,
        false,
        false,
        true,
        HistoricalPathReuseDescriptor::with_replay_tail_reuse(),
    );
    let admission =
        admit_historical_evaluation_path(request, capability).expect("admission should succeed");
    let resolved = resolve_historical_materialization_path(
        admission,
        HistoricalMaterializationDescriptor::new_for_test(
            "basis:counts",
            ResolvedHistoricalPathClass::ResolvedDeltaReplayPath,
        ),
    )
    .expect("resolution should succeed");

    assert_eq!(resolved.counters().historical_admitted_path_count(), 1);
    assert_eq!(resolved.counters().historical_resolved_path_count(), 1);
    assert_eq!(
        resolved.counters().historical_result_path_metadata_count(),
        1
    );
    assert_eq!(
        resolved
            .counters()
            .historical_delta_replay_admission_count(),
        1
    );
    assert_eq!(
        resolved
            .counters()
            .history_work_avoided_by_retained_path_count(),
        0
    );
}

#[test]
fn retained_reuse_counter_only_increments_when_capability_proves_reuse() {
    let request = HistoricalEvaluationRequest::retained_snapshot_for_test(
        "basis:retained-reuse",
        3,
        5,
        HistoricalPathReuseDescriptor::retained_reuse(),
    );
    let capability =
        super::super::request::HistoricalCapabilityDescriptor::retained_snapshot_for_test(
            "basis:retained-reuse",
            HistoricalPathReuseDescriptor::retained_reuse(),
        );
    let admission =
        admit_historical_evaluation_path(request, capability).expect("admission should succeed");
    let resolved = resolve_historical_materialization_path(
        admission,
        HistoricalMaterializationDescriptor::retained_snapshot_for_test("basis:retained-reuse"),
    )
    .expect("resolution should succeed");

    assert_eq!(
        resolved
            .counters()
            .history_work_avoided_by_retained_path_count(),
        8
    );
}

#[test]
fn bridge_lowering_preserves_decision_log_path_semantics() {
    let host_request = worth_runtime_bridge::facade::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime(BridgeRuntimePolicy::default());
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
    assert_eq!(
        evaluation.record().decision_log().materialization_path(),
        BridgeHistoricalMaterializationPath::CommitEnvelopeSnapshot
    );
    let lowered = lower_materialization_from_decision_log(evaluation.record().decision_log())
        .expect("decision log should lower");

    assert_eq!(
        lowered.resolved_path_class(),
        &ResolvedHistoricalPathClass::ResolvedDeltaReplayPath
    );
}
