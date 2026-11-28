use crate::facade::BridgeRouteRequest;
use crate::harness::fixtures::NamedTruth;

use super::support::build_runtime_with_aspects;
use crate::facade::{
    BridgeDeliveryIntent, BridgeReplayMode, BridgeTruthViewSelector,
    HistoricalEvaluationDeclaration, SnapshotReadPacket, TruthBranchIdentity,
};
use crate::harness::fixtures::RecordingSignalBridgeSink;

#[test]
fn bridge_counters_expose_digest_input_bytes() {
    let serial_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let truth = NamedTruth::new();
    let (commit_a, _observation_a) = truth.commit_name(execution, "alice");
    let source = truth.source.clone();

    let runtime = build_runtime_with_aspects(
        source,
        RecordingSignalBridgeSink::default(),
        vec![NamedTruth::mapping()],
        vec![NamedTruth::aspect_mapping()],
    );

    let result = runtime
        .deliver_invalidation(
            runtime
                .plan_committed_patch(BridgeRouteRequest::for_commit(commit_a.clone()), execution)
                .expect("route should plan before digest budget capture"),
            execution,
        )
        .expect("delivery should succeed before digest budget capture");

    assert!(result.counters().digest_computation_count() >= 8);
    assert!(result.counters().digest_input_bytes() > 0);
}

#[test]
fn historical_evaluation_counters_capture_selector_branch_and_materialization_width() {
    let serial_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let truth = NamedTruth::new();
    let (commit_a, _observation_a) = truth.commit_name(execution, "alice");
    let _head = truth.bind_main_head();
    let source = truth.source.clone();

    let runtime = build_runtime_with_aspects(
        source,
        RecordingSignalBridgeSink::default(),
        vec![NamedTruth::mapping()],
        vec![NamedTruth::aspect_mapping()],
    );

    let declaration = HistoricalEvaluationDeclaration::new(
        BridgeTruthViewSelector::historical_commit(
            TruthBranchIdentity::from_relational_branch_id("main"),
            commit_a.clone(),
        ),
        BridgeReplayMode::Enabled,
        runtime.policy().diagnostics_tier(),
        BridgeDeliveryIntent::PrepareSignalEvaluation,
    );
    let observation = runtime
        .materialize_truth_view_observation(
            runtime
                .plan_truth_view_packet(declaration, SnapshotReadPacket::new(vec![]), execution)
                .expect("historical declaration should plan"),
            execution,
        )
        .expect("historical declaration should materialize");
    let record = runtime.canonicalize_historical_evaluation_record(&observation);

    assert_eq!(record.counters().truth_view_selector_count(), 1);
    assert_eq!(record.counters().historical_truth_view_count(), 1);
    assert_eq!(record.counters().branch_truth_view_count(), 0);
    assert_eq!(record.counters().planned_truth_view_packet_count(), 1);
    assert_eq!(record.counters().resolved_truth_view_policy_count(), 1);
    assert_eq!(record.counters().materialized_truth_view_count(), 1);
    assert_eq!(record.counters().truth_view_decision_log_count(), 1);
    assert_eq!(record.counters().selector_width(), 1);
    assert_eq!(record.counters().branch_width(), 1);
    assert_eq!(
        record.decision_log().materialization_path(),
        crate::facade::BridgeHistoricalMaterializationPath::PinnedCommitObservation
    );
    assert_eq!(record.counters().direct_snapshot_materialization_count(), 0);
    assert_eq!(record.counters().branch_head_materialization_count(), 0);
}
