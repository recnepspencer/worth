use super::{
    build_runtime, BridgeBulkWorkloadRequest, BridgeBulkWorkloadSegment,
    BridgeInvalidationReductionFamily, BridgeParallelAdmissionReason, BridgeRouteRequest,
    RecordingSignalBridgeSink,
};

#[test]
fn bridge_bulk_packet_reduction_collapses_duplicate_slice_targets() {
    let serial_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let truth = ProfileTruth::new(execution);
    let runtime = build_runtime(
        truth.native.source.clone(),
        RecordingSignalBridgeSink::default(),
        vec![ProfileTruth::mapping()],
    );

    let planned = runtime
        .plan_bulk_workload(
            BridgeBulkWorkloadRequest::new(vec![
                BridgeBulkWorkloadSegment::new(BridgeRouteRequest::for_commit(
                    truth.commit.clone(),
                )),
                BridgeBulkWorkloadSegment::new(BridgeRouteRequest::for_commit(
                    truth.commit.clone(),
                )),
            ]),
            execution,
        )
        .expect("duplicate workload should plan");

    assert_eq!(planned.packet_set().routing_packets().len(), 2);
    assert_eq!(planned.packet_set().truth_view_packets().len(), 1);
    assert_eq!(planned.packet_set().reduction_packets().len(), 1);
    assert_eq!(
        planned.packet_set().reduction_packets()[0].reduction_family(),
        BridgeInvalidationReductionFamily::Publication
    );
    assert_eq!(
        planned.packet_set().reduction_packets()[0].reduced_subscription_slice_identity(),
        planned.packet_set().routing_packets()[0].subscription_slice_identity()
    );
    assert_eq!(planned.packet_set().counters().bulk_packet_count(), 6);
    assert_eq!(
        planned.execution_plan().parallel_admission().reason(),
        BridgeParallelAdmissionReason::SharedTruthViewMaterializationTarget
    );
    assert_eq!(
        planned
            .execution_plan()
            .locality_footprint()
            .publication_scope_count(),
        1
    );
}

use crate::harness::fixtures::ProfileTruth;
