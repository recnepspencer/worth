use super::support::{committed_patch, registration, snapshot};
use crate::facade::{BridgeBulkWorkloadRequest, BridgeBulkWorkloadSegment, BridgeRouteRequest};
use crate::harness::fixtures::{InMemoryRelationalBridgeSource, RecordingSignalBridgeSink};
use crate::truth_identity_fixtures::{
    truth_commit_fixture, truth_patch_fixture, truth_snapshot_fixture,
};

#[test]
fn bridge_diagnostics_retain_queryable_bulk_records_by_workload_identity() {
    let serial_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let source = InMemoryRelationalBridgeSource::default();
    source.insert_committed_patch(committed_patch(
        truth_commit_fixture("commit-a"),
        truth_patch_fixture("patch-a"),
        truth_snapshot_fixture("snapshot-a"),
        worth_foundational::facade::FieldKey::new("name".to_owned())
            .expect("valid harness field key"),
    ));
    source.insert_committed_patch(committed_patch(
        truth_commit_fixture("commit-b"),
        truth_patch_fixture("patch-b"),
        truth_snapshot_fixture("snapshot-b"),
        worth_foundational::facade::FieldKey::new("name".to_owned())
            .expect("valid harness field key"),
    ));
    source.insert_snapshot(snapshot(truth_snapshot_fixture("snapshot-a"), "alice"));
    source.insert_snapshot(snapshot(truth_snapshot_fixture("snapshot-b"), "bob"));
    let runtime = crate::facade::RuntimeBridge::builder()
        .with_relational_source(source)
        .with_signal_sink(RecordingSignalBridgeSink::default())
        .with_policy(
            crate::facade::BridgeRuntimePolicy::development()
                .with_route_record_limit(2)
                .with_failure_record_limit(2),
        )
        .register_mapping(registration())
        .build()
        .expect("bridge runtime with bounded diagnostics retention");

    let plan = runtime
        .plan_bulk_workload(
            BridgeBulkWorkloadRequest::new(vec![
                BridgeBulkWorkloadSegment::new(BridgeRouteRequest::for_commit(
                    truth_commit_fixture("commit-a"),
                )),
                BridgeBulkWorkloadSegment::new(BridgeRouteRequest::for_commit(
                    truth_commit_fixture("commit-b"),
                )),
            ]),
            execution,
        )
        .expect("bulk workload should plan before diagnostics retention");
    let record = runtime.canonicalize_bulk_workload_plan(&plan);

    let diagnostics = runtime.diagnostics();
    let queried = diagnostics
        .bulk_record_for_workload_identity(record.workload_identity())
        .expect("bulk record should be queryable by workload identity");
    let handle = diagnostics.handle();

    assert_eq!(diagnostics.bulk_records().len(), 1);
    assert_eq!(queried.workload_identity(), record.workload_identity());
    assert_eq!(
        diagnostics
            .last_bulk_record()
            .expect("last bulk record")
            .execution_plan_digest(),
        record.execution_plan_digest()
    );
    assert_eq!(handle.bulk_records().len(), 1);
    assert_eq!(
        handle
            .bulk_record_for_workload_identity(record.workload_identity())
            .expect("handle should query bulk record by workload identity")
            .packet_set_digest(),
        record.packet_set_digest()
    );
}
