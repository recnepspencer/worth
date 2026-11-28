use super::*;
use crate::harness::fixtures::ProfileTruth;

#[test]
fn bridge_continuity_replay_matches_original_canonical_artifact() {
    let serial_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let truth = ProfileTruth::new(execution);
    let source = truth.native.source.clone();
    let runtime = crate::facade::RuntimeBridgeBuilder::new()
        .with_relational_source(source)
        .with_signal_sink(RecordingSignalBridgeSink::default())
        .with_continuity_lineage_source(ReplaySingleSuccessorLineageSource)
        .register_mapping(ProfileTruth::mapping())
        .register_aspect_mapping(ProfileTruth::aspect_mapping())
        .build()
        .expect("runtime should build");

    let route = runtime
        .plan_committed_patch_with_mapping_context(
            BridgeRouteRequest::for_commit(truth.commit.clone()),
            crate::facade::BridgeMappingContext::default().with_lineage_context(
                BridgeLineageContext::new(BridgeContinuityAuthorityBasis::new(
                    crate::truth_identity_fixtures::truth_branch_fixture("main"),
                    truth.observation.snapshot_identity().clone(),
                )),
            ),
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
    let canonical = runtime.canonicalize_continuity_record(&route_record, &requests, &artifact);

    let replay = runtime
        .replay_canonical_continuity_record(&canonical, execution)
        .expect("continuity replay should preserve canonical artifact");

    assert_eq!(replay.continuity_identity(), artifact.continuity_identity());
    assert_eq!(
        replay.remapped_subscription_slice_identity(),
        artifact.remapped_subscription_slice_identity()
    );
    assert_eq!(replay.remapped_slices(), artifact.remapped_slices());
}

#[test]
fn bridge_continuity_replay_rejects_artifact_drift() {
    let serial_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let truth = ProfileTruth::new(execution);
    let original_source = truth.native.source.clone();
    let original_runtime = crate::facade::RuntimeBridgeBuilder::new()
        .with_relational_source(original_source)
        .with_signal_sink(RecordingSignalBridgeSink::default())
        .with_continuity_lineage_source(ReplaySingleSuccessorLineageSource)
        .register_mapping(ProfileTruth::mapping())
        .register_aspect_mapping(ProfileTruth::aspect_mapping())
        .build()
        .expect("runtime should build");

    let route = original_runtime
        .plan_committed_patch_with_mapping_context(
            BridgeRouteRequest::for_commit(truth.commit.clone()),
            crate::facade::BridgeMappingContext::default().with_lineage_context(
                BridgeLineageContext::new(BridgeContinuityAuthorityBasis::new(
                    crate::truth_identity_fixtures::truth_branch_fixture("main"),
                    truth.observation.snapshot_identity().clone(),
                )),
            ),
            execution,
        )
        .expect("route should plan");
    let result = original_runtime
        .deliver_invalidation(route, execution)
        .expect("delivery should succeed");
    let route_record = original_runtime
        .diagnostics()
        .route_record_for_route_identity(result.result_summary().route_identity())
        .expect("route record should be retained");
    let requests = original_runtime
        .plan_continuity_requests(&route_record)
        .expect("continuity requests should plan");
    let packet = original_runtime
        .plan_historical_lineage_packet(&requests, execution)
        .expect("historical lineage packet should plan");
    let resolved = original_runtime
        .resolve_lineage_continuity(&packet)
        .expect("continuity should resolve");
    let artifact = original_runtime.lower_continuity_artifact(&resolved);
    let canonical =
        original_runtime.canonicalize_continuity_record(&route_record, &requests, &artifact);

    let restarted_source = truth.native.source.clone();
    let restarted_runtime = crate::facade::RuntimeBridgeBuilder::new()
        .with_relational_source(restarted_source)
        .with_signal_sink(RecordingSignalBridgeSink::default())
        .with_continuity_lineage_source(ReplayDriftedSuccessorLineageSource)
        .register_mapping(ProfileTruth::mapping())
        .register_aspect_mapping(ProfileTruth::aspect_mapping())
        .build()
        .expect("runtime should build");

    let error = restarted_runtime
        .replay_canonical_continuity_record(&canonical, execution)
        .expect_err("continuity replay should reject successor drift");

    assert_eq!(
        error.kind(),
        crate::facade::BridgeReplayErrorKind::ContinuityResolutionMismatch
    );
    assert_eq!(
        error.context().route_identity(),
        Some(canonical.route_record().route_identity())
    );
    assert_eq!(
        error.context().snapshot_identity(),
        Some(canonical.route_record().source_snapshot())
    );
    assert_eq!(
        error.context().subscription_slice_identity(),
        Some(canonical.remapped_subscription_slice_identity())
    );
}
