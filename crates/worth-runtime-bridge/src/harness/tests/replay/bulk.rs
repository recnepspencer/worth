use super::*;
use crate::harness::fixtures::NamedTruth;

#[test]
fn replayed_bulk_plan_matches_original_canonical_artifact() {
    let serial_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let truth = NamedTruth::new();
    let (commit_a, _observation_a) = truth.commit_name(execution, "alice");
    let (commit_b, _observation_b) = truth.commit_name(execution, "bob");
    let source = truth.source.clone();

    let runtime = build_runtime(
        source,
        RecordingSignalBridgeSink::default(),
        vec![NamedTruth::mapping()],
    );

    let planned = runtime
        .plan_bulk_workload(
            BridgeBulkWorkloadRequest::new(vec![
                BridgeBulkWorkloadSegment::new(BridgeRouteRequest::for_commit(commit_a.clone())),
                BridgeBulkWorkloadSegment::new(BridgeRouteRequest::for_commit(commit_b.clone())),
            ]),
            execution,
        )
        .expect("bulk workload should plan before canonical bulk replay");
    let canonical = runtime.canonicalize_bulk_workload_plan(&planned);

    let replayed = runtime
        .replay_canonical_bulk_plan_record(&canonical, execution)
        .expect("bulk canonical replay should preserve the canonical plan");

    assert_eq!(
        canonical.schema_version(),
        BRIDGE_CANONICAL_BULK_PLAN_RECORD_SCHEMA_V1
    );
    assert_eq!(replayed.workload_identity(), planned.workload_identity());
    assert_eq!(
        replayed.canonical_planning_identity(),
        planned.canonical_planning_identity()
    );
    assert_eq!(
        replayed.packet_set().digest(),
        planned.packet_set().digest()
    );
    assert_eq!(
        replayed.execution_plan().digest(),
        planned.execution_plan().digest()
    );
}

#[test]
fn bulk_replay_rejects_drift_after_restart_shaped_truth_change() {
    let serial_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let original = ProfileTruth::new(execution);
    let (second_commit, _second_observation) = original.update_field(execution, "title", "bob");
    let original_runtime = build_runtime(
        original.native.source.clone(),
        RecordingSignalBridgeSink::default(),
        vec![ProfileTruth::widening_mapping()],
    );

    let planned = original_runtime
        .plan_bulk_workload(
            BridgeBulkWorkloadRequest::new(vec![
                BridgeBulkWorkloadSegment::new(BridgeRouteRequest::for_commit(
                    original.commit.clone(),
                )),
                BridgeBulkWorkloadSegment::new(BridgeRouteRequest::for_commit(
                    second_commit.clone(),
                )),
            ]),
            execution,
        )
        .expect("original bulk workload should plan");
    let canonical = original_runtime.canonicalize_bulk_workload_plan(&planned);

    let restarted = ProfileTruth::on_branch_with_field(execution, "main", "status");
    let (restarted_second, _second_observation) = restarted.update_field(execution, "title", "bob");
    assert_eq!(restarted.commit, original.commit);
    assert_eq!(restarted_second, second_commit);
    let restarted_runtime = build_runtime(
        restarted.native.source.clone(),
        RecordingSignalBridgeSink::default(),
        vec![ProfileTruth::widening_mapping()],
    );

    let error = restarted_runtime
        .replay_canonical_bulk_plan_record(&canonical, execution)
        .expect_err("bulk replay should reject drift after restart-shaped truth change");

    assert_eq!(
        error.kind(),
        crate::facade::BridgeReplayErrorKind::BulkPlanReplayMismatch
    );
    assert_eq!(canonical.workload_identity(), planned.workload_identity());
    assert_eq!(
        canonical.execution_plan_digest(),
        planned.execution_plan().digest()
    );
}

use crate::harness::fixtures::ProfileTruth;
