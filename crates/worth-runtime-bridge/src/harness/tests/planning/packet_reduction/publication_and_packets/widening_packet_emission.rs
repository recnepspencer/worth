use super::{
    build_runtime, BridgeBulkWorkloadRequest, BridgeBulkWorkloadSegment,
    BridgeMappingWideningClass, BridgeRouteRequest, RecordingSignalBridgeSink,
};

#[test]
fn bridge_bulk_packet_set_emits_widening_packets_for_widening_admitted_slices() {
    let serial_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let truth = ProfileTruth::new(execution);
    let runtime = build_runtime(
        truth.native.source.clone(),
        RecordingSignalBridgeSink::default(),
        vec![ProfileTruth::widening_mapping()],
    );

    let planned = runtime
        .plan_bulk_workload(
            BridgeBulkWorkloadRequest::new(vec![BridgeBulkWorkloadSegment::new(
                BridgeRouteRequest::for_commit(truth.commit.clone()),
            )]),
            execution,
        )
        .expect("widening workload should plan");

    assert_eq!(planned.packet_set().routing_packets().len(), 1);
    assert_eq!(planned.packet_set().widening_packets().len(), 1);
    assert_eq!(
        planned.packet_set().widening_packets()[0].originating_route_identity(),
        planned.packet_set().routing_packets()[0].route_identity()
    );
    assert_eq!(
        planned.packet_set().widening_packets()[0].widening_class(),
        BridgeMappingWideningClass::EntitySurface
    );
    assert!(planned.packet_set().widening_packets()[0]
        .bounded_scope_identity()
        .starts_with("truth-delta-surface:sha256:"));
    assert!(!planned.packet_set().widening_packets()[0]
        .bounded_scope_identity()
        .contains("committed-patch-target"));
    assert_eq!(planned.packet_set().truth_view_packets().len(), 1);
    assert_eq!(planned.packet_set().counters().bulk_packet_count(), 4);
    assert_eq!(
        planned
            .execution_plan()
            .reduced_artifact()
            .reduced_widenings()
            .len(),
        1
    );
    assert_eq!(
        planned.packet_set().widening_packets()[0].bounded_truth_delta_surface_identity(),
        planned
            .execution_plan()
            .reduced_artifact()
            .reduced_widenings()[0]
            .bounded_truth_delta_surface_identity()
    );
    assert_eq!(
        planned
            .execution_plan()
            .reduced_artifact()
            .reduced_widenings()[0]
            .reduced_route_identities(),
        &[planned.packet_set().routing_packets()[0]
            .route_identity()
            .clone()]
    );
    assert_eq!(
        planned
            .execution_plan()
            .reduced_artifact()
            .reduction_output_count(),
        3
    );
}

use crate::harness::fixtures::ProfileTruth;
