use super::*;

#[test]
fn bridge_resolved_lineage_continuity_lowers_single_successor_artifact() {
    let serial_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let truth = ProfileTruth::new(execution);
    let source = truth.native.source.clone();
    let sink = RecordingSignalBridgeSink::default();
    let runtime = crate::facade::RuntimeBridgeBuilder::new()
        .with_relational_source(source)
        .with_signal_sink(sink)
        .with_continuity_lineage_source(TestContinuityLineageSource)
        .register_mapping(ProfileTruth::mapping())
        .register_aspect_mapping(ProfileTruth::aspect_mapping())
        .build()
        .expect("runtime should build");

    let route = runtime
        .plan_committed_patch_with_mapping_context(
            BridgeRouteRequest::for_commit(truth.commit.clone()),
            BridgeMappingContext::default().with_lineage_context(truth.lineage_context()),
            execution,
        )
        .expect("route should plan");
    let result = runtime
        .deliver_invalidation(route, execution)
        .expect("delivery should succeed");
    let route_record = runtime
        .diagnostics()
        .route_record_for_route_identity(result.result_summary().route_identity())
        .expect("route record should be retained");

    let requests = runtime
        .plan_continuity_requests(&route_record)
        .expect("continuity requests should plan");
    let packet = runtime
        .plan_historical_lineage_packet(&requests, execution)
        .expect("historical lineage packet should plan");
    let resolved = runtime
        .resolve_lineage_continuity(&packet)
        .expect("continuity should resolve");
    let artifact = runtime.lower_continuity_artifact(&resolved);

    assert_eq!(resolved.continuity_entries().len(), 1);
    assert_eq!(
        resolved.continuity_entries()[0].outcome_class(),
        crate::facade::BridgeContinuityOutcomeClass::ContinuesAsSingleSuccessor
    );
    assert_eq!(artifact.remapped_slices().len(), 1);
    assert_eq!(
        artifact.remapped_slices().slices()[0].entity_identity(),
        "entity:0:4:2"
    );
    assert_eq!(
        artifact.remapped_slices().slices()[0].aspect_key().as_str(),
        "profile"
    );
    assert!(artifact.remapped_slices().slices()[0]
        .field_locator()
        .is_some());
    assert!(!artifact.remapped_slices().slices()[0]
        .projection_mask()
        .is_whole_aspect());
    assert_eq!(artifact.counters().continuity_request_count(), 1);
    assert_eq!(artifact.counters().continuity_prior_slice_count(), 1);
    assert_eq!(artifact.counters().continuity_single_successor_count(), 1);
}

use crate::harness::fixtures::ProfileTruth;
