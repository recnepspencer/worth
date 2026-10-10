use super::*;

#[test]
fn bridge_continuity_replay_matches_original_canonical_artifact() {
    let serial_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let source = InMemoryRelationalBridgeSource::default();
    source.insert_committed_patch(committed_patch(
        crate::truth_identity_fixtures::truth_commit_fixture("commit-a"),
        crate::truth_identity_fixtures::truth_patch_fixture("patch-a"),
        crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
        worth_foundational::facade::FieldKey::new("name".to_owned())
            .expect("valid harness field key"),
    ));
    source.insert_snapshot(field_slice_snapshot(
        crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
        "alice",
    ));
    let runtime = crate::facade::RuntimeBridgeBuilder::new()
        .with_relational_source(source)
        .with_signal_sink(RecordingSignalBridgeSink::default())
        .with_continuity_lineage_source(ReplaySingleSuccessorLineageSource)
        .register_mapping(registration())
        .register_aspect_mapping(field_aspect_registration())
        .build()
        .expect("runtime should build");

    let route = runtime
        .plan_committed_patch_with_mapping_context(
            BridgeRouteRequest::for_commit(crate::truth_identity_fixtures::truth_commit_fixture(
                "commit-a",
            )),
            crate::facade::BridgeMappingContext::default().with_lineage_context(
                BridgeLineageContext::new(BridgeContinuityAuthorityBasis::new(
                    crate::truth_identity_fixtures::truth_branch_fixture("main"),
                    crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
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
        .plan_historical_lineage_packet(&requests)
        .expect("historical lineage packet should plan");
    let resolved = runtime
        .resolve_lineage_continuity(&packet)
        .expect("continuity should resolve");
    let artifact = runtime.lower_continuity_artifact(&resolved);
    let canonical = runtime.canonicalize_continuity_record(&route_record, &requests, &artifact);

    let replay = runtime
        .replay_canonical_continuity_record(&canonical)
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

    let original_source = InMemoryRelationalBridgeSource::default();
    original_source.insert_committed_patch(committed_patch(
        crate::truth_identity_fixtures::truth_commit_fixture("commit-a"),
        crate::truth_identity_fixtures::truth_patch_fixture("patch-a"),
        crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
        worth_foundational::facade::FieldKey::new("name".to_owned())
            .expect("valid harness field key"),
    ));
    original_source.insert_snapshot(field_slice_snapshot(
        crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
        "alice",
    ));
    let original_runtime = crate::facade::RuntimeBridgeBuilder::new()
        .with_relational_source(original_source)
        .with_signal_sink(RecordingSignalBridgeSink::default())
        .with_continuity_lineage_source(ReplaySingleSuccessorLineageSource)
        .register_mapping(registration())
        .register_aspect_mapping(field_aspect_registration())
        .build()
        .expect("runtime should build");

    let route = original_runtime
        .plan_committed_patch_with_mapping_context(
            BridgeRouteRequest::for_commit(crate::truth_identity_fixtures::truth_commit_fixture(
                "commit-a",
            )),
            crate::facade::BridgeMappingContext::default().with_lineage_context(
                BridgeLineageContext::new(BridgeContinuityAuthorityBasis::new(
                    crate::truth_identity_fixtures::truth_branch_fixture("main"),
                    crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
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
        .plan_historical_lineage_packet(&requests)
        .expect("historical lineage packet should plan");
    let resolved = original_runtime
        .resolve_lineage_continuity(&packet)
        .expect("continuity should resolve");
    let artifact = original_runtime.lower_continuity_artifact(&resolved);
    let canonical =
        original_runtime.canonicalize_continuity_record(&route_record, &requests, &artifact);

    let restarted_source = InMemoryRelationalBridgeSource::default();
    restarted_source.insert_committed_patch(committed_patch(
        crate::truth_identity_fixtures::truth_commit_fixture("commit-a"),
        crate::truth_identity_fixtures::truth_patch_fixture("patch-a"),
        crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
        worth_foundational::facade::FieldKey::new("name".to_owned())
            .expect("valid harness field key"),
    ));
    restarted_source.insert_snapshot(field_slice_snapshot(
        crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
        "alice",
    ));
    let restarted_runtime = crate::facade::RuntimeBridgeBuilder::new()
        .with_relational_source(restarted_source)
        .with_signal_sink(RecordingSignalBridgeSink::default())
        .with_continuity_lineage_source(ReplayDriftedSuccessorLineageSource)
        .register_mapping(registration())
        .register_aspect_mapping(field_aspect_registration())
        .build()
        .expect("runtime should build");

    let error = restarted_runtime
        .replay_canonical_continuity_record(&canonical)
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
