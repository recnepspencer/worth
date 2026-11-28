use super::super::super::support::build_runtime;
use crate::facade::BridgeRouteRequest;
use crate::harness::fixtures::NamedTruth;
use crate::harness::fixtures::RecordingSignalBridgeSink;

#[test]
fn bridge_replay_accepts_versioned_canonical_route_record() {
    let serial_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let truth = NamedTruth::new();
    let (commit_a, _observation_a) = truth.commit_name(execution, "alice");
    let source = truth.source.clone();

    let runtime = build_runtime(
        source,
        RecordingSignalBridgeSink::default(),
        vec![NamedTruth::mapping()],
    );

    let route = runtime
        .plan_committed_patch(
            BridgeRouteRequest::for_commit(crate::truth_identity_fixtures::truth_commit_fixture(
                "commit-a",
            )),
            execution,
        )
        .expect("bridge should plan route before canonical replay capture");
    runtime
        .deliver_invalidation(route, execution)
        .expect("bridge should deliver route before canonical replay capture");
    let canonical_record = runtime
        .diagnostics()
        .last_canonical_route_record()
        .expect("bridge should expose a versioned canonical route record");

    let replay = runtime
        .replay_canonical_record(&canonical_record, execution)
        .expect("bridge should replay a supported canonical route record");

    assert_eq!(replay.source_commit().as_str(), commit_a.clone().as_str());
    assert_eq!(
        canonical_record.schema_version(),
        crate::facade::BRIDGE_CANONICAL_ROUTE_RECORD_SCHEMA_V3
    );
}
