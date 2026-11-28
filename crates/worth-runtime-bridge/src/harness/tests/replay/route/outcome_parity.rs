use super::super::super::support::build_runtime_with_aspects;
use crate::facade::BridgeRouteRequest;
use crate::harness::fixtures::{NamedTruth, RecordingSignalBridgeSink};

#[test]
fn bridge_replay_preserves_canonical_route_outcome_for_delivered_patch() {
    let serial_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let truth = NamedTruth::new();
    let (commit, _observation) = truth.commit_name(execution, "alice");
    let runtime = build_runtime_with_aspects(
        truth.source,
        RecordingSignalBridgeSink::default(),
        vec![NamedTruth::mapping()],
        vec![NamedTruth::aspect_mapping()],
    );

    let result = runtime
        .deliver_invalidation(
            runtime
                .plan_committed_patch(BridgeRouteRequest::for_commit(commit), execution)
                .expect("route should plan before replay parity certification"),
            execution,
        )
        .expect("route should deliver before replay parity certification");
    let canonical_record = runtime
        .diagnostics()
        .last_canonical_route_record()
        .expect("bridge should retain a canonical route record for replay parity certification");

    let replay = runtime
        .replay_canonical_record(&canonical_record, execution)
        .expect("bridge replay should preserve the canonical route outcome");

    assert_eq!(
        replay.route_identity(),
        result.result_summary().route_identity()
    );
    assert_eq!(
        replay.invalidation_identity(),
        result.result_summary().invalidation_identity()
    );
    assert_eq!(
        replay.subscription_slice_identity(),
        result.result_summary().subscription_slice_identity()
    );
    assert_eq!(
        replay.source_commit(),
        result.result_summary().source_commit()
    );
    assert_eq!(
        replay.source_patch(),
        result.result_summary().source_patch()
    );
    assert_eq!(
        replay.source_snapshot(),
        result.result_summary().snapshot_identity()
    );
}
