use super::super::{
    commit_a, committed_patch, patch_a, registration, snapshot, snapshot_a,
    CountingSnapshotReaderPool,
};
use crate::facade::{BridgeRouteRequest, RuntimeBridgeBuilder};
use crate::harness::fixtures::{InMemoryRelationalBridgeSource, RecordingSignalBridgeSink};

#[test]
fn bridge_snapshot_reader_pool_is_used_when_configured() {
    let serial_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let source = InMemoryRelationalBridgeSource::default();
    source.insert_committed_patch(committed_patch(
        commit_a(),
        patch_a(),
        snapshot_a(),
        worth_foundational::facade::FieldKey::new("name".to_owned())
            .expect("valid harness field key"),
    ));
    source.insert_snapshot(snapshot(snapshot_a(), "alice"));
    let pool = CountingSnapshotReaderPool::new(source.clone());
    let runtime = RuntimeBridgeBuilder::new()
        .with_relational_source(source)
        .with_snapshot_reader_pool(pool.clone())
        .with_signal_sink(RecordingSignalBridgeSink::default())
        .register_mapping(registration())
        .build()
        .expect("bridge runtime should build with a snapshot reader pool");

    runtime
        .deliver_invalidation(
            runtime
                .plan_committed_patch(BridgeRouteRequest::for_commit(commit_a()), execution)
                .expect("bridge should plan the route"),
            execution,
        )
        .expect("bridge delivery should succeed");

    assert_eq!(pool.acquire_count(), 1);
    assert_eq!(pool.release_count(), 1);
}
