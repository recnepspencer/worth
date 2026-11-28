use crate::facade::{BridgeBulkWorkloadRequest, BridgeBulkWorkloadSegment, BridgeRouteRequest};
use crate::harness::fixtures::NamedTruth;
use crate::harness::fixtures::RecordingSignalBridgeSink;

#[test]
fn bridge_diagnostics_retain_queryable_bulk_records_by_workload_identity() {
    let serial_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let truth = NamedTruth::new();
    let (commit_a, _observation_a) = truth.commit_name(execution, "alice");
    let (commit_b, _observation_b) = truth.commit_name(execution, "bob");
    let source = truth.source.clone();

    let runtime = crate::facade::RuntimeBridge::builder()
        .with_relational_source(source)
        .with_signal_sink(RecordingSignalBridgeSink::default())
        .with_policy(
            crate::facade::BridgeRuntimePolicy::development()
                .with_route_record_limit(2)
                .with_failure_record_limit(2),
        )
        .register_mapping(NamedTruth::mapping())
        .build()
        .expect("bridge runtime with bounded diagnostics retention");

    let plan = runtime
        .plan_bulk_workload(
            BridgeBulkWorkloadRequest::new(vec![
                BridgeBulkWorkloadSegment::new(BridgeRouteRequest::for_commit(commit_a.clone())),
                BridgeBulkWorkloadSegment::new(BridgeRouteRequest::for_commit(commit_b.clone())),
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
