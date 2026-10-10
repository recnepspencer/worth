use super::*;

#[test]
fn bridge_deliver_continuity_returns_delivered_result_and_canonical_record() {
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
    let sink = RecordingSignalBridgeSink::default();
    let runtime = crate::facade::RuntimeBridgeBuilder::new()
        .with_relational_source(source)
        .with_signal_sink(sink)
        .with_continuity_lineage_source(TestContinuityLineageSource)
        .register_mapping(registration())
        .register_aspect_mapping(field_aspect_registration())
        .build()
        .expect("runtime should build");

    let route = runtime
        .plan_committed_patch_with_mapping_context(
            BridgeRouteRequest::for_commit(crate::truth_identity_fixtures::truth_commit_fixture(
                "commit-a",
            )),
            BridgeMappingContext::default().with_lineage_context(BridgeLineageContext::new(
                BridgeContinuityAuthorityBasis::new(
                    crate::truth_identity_fixtures::truth_branch_fixture("main"),
                    crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
                ),
            )),
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

    let delivered = runtime
        .deliver_continuity(&route_record)
        .expect("continuity should deliver as a single bridge-owned result");

    assert_eq!(
        delivered.canonical_record().continuity_artifact_identity(),
        delivered.continuity_identity()
    );
    assert_eq!(
        delivered
            .canonical_record()
            .remapped_subscription_slice_identity(),
        delivered.remapped_subscription_slice_identity()
    );
    assert_eq!(
        delivered.canonical_record().remapped_slices(),
        delivered.remapped_slices()
    );
}

#[test]
fn bridge_continuity_truth_is_invariant_across_diagnostics_tiers() {
    let serial_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let standard_source = InMemoryRelationalBridgeSource::default();
    standard_source.insert_committed_patch(committed_patch(
        crate::truth_identity_fixtures::truth_commit_fixture("commit-a"),
        crate::truth_identity_fixtures::truth_patch_fixture("patch-a"),
        crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
        worth_foundational::facade::FieldKey::new("name".to_owned())
            .expect("valid harness field key"),
    ));
    standard_source.insert_snapshot(field_slice_snapshot(
        crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
        "alice",
    ));
    let forensic_source = standard_source.clone();

    let standard_runtime = crate::facade::RuntimeBridgeBuilder::new()
        .with_policy(BridgeRuntimePolicy::development())
        .with_relational_source(standard_source)
        .with_signal_sink(RecordingSignalBridgeSink::default())
        .with_continuity_lineage_source(TestContinuityLineageSource)
        .register_mapping(registration())
        .register_aspect_mapping(field_aspect_registration())
        .build()
        .expect("standard runtime should build");
    let forensic_runtime = crate::facade::RuntimeBridgeBuilder::new()
        .with_policy(BridgeRuntimePolicy::forensic())
        .with_relational_source(forensic_source)
        .with_signal_sink(RecordingSignalBridgeSink::default())
        .with_continuity_lineage_source(TestContinuityLineageSource)
        .register_mapping(registration())
        .register_aspect_mapping(field_aspect_registration())
        .build()
        .expect("forensic runtime should build");

    let standard_route = standard_runtime
        .plan_committed_patch_with_mapping_context(
            BridgeRouteRequest::for_commit(crate::truth_identity_fixtures::truth_commit_fixture(
                "commit-a",
            )),
            BridgeMappingContext::default().with_lineage_context(BridgeLineageContext::new(
                BridgeContinuityAuthorityBasis::new(
                    crate::truth_identity_fixtures::truth_branch_fixture("main"),
                    crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
                ),
            )),
            execution,
        )
        .expect("standard route should plan");
    let forensic_route = forensic_runtime
        .plan_committed_patch_with_mapping_context(
            BridgeRouteRequest::for_commit(crate::truth_identity_fixtures::truth_commit_fixture(
                "commit-a",
            )),
            BridgeMappingContext::default().with_lineage_context(BridgeLineageContext::new(
                BridgeContinuityAuthorityBasis::new(
                    crate::truth_identity_fixtures::truth_branch_fixture("main"),
                    crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
                ),
            )),
            execution,
        )
        .expect("forensic route should plan");

    let standard_result = standard_runtime
        .deliver_invalidation(standard_route, execution)
        .expect("standard delivery should succeed");
    let forensic_result = forensic_runtime
        .deliver_invalidation(forensic_route, execution)
        .expect("forensic delivery should succeed");

    let standard_route_record = standard_runtime
        .diagnostics()
        .route_record_for_route_identity(standard_result.result_summary().route_identity())
        .expect("standard route record should be retained");
    let forensic_route_record = forensic_runtime
        .diagnostics()
        .route_record_for_route_identity(forensic_result.result_summary().route_identity())
        .expect("forensic route record should be retained");

    let standard = standard_runtime
        .deliver_continuity(&standard_route_record)
        .expect("standard continuity should deliver");
    let forensic = forensic_runtime
        .deliver_continuity(&forensic_route_record)
        .expect("forensic continuity should deliver");

    assert_eq!(
        standard.continuity_identity(),
        forensic.continuity_identity()
    );
    assert_eq!(
        standard.canonical_record().continuity_resolution_digest(),
        forensic.canonical_record().continuity_resolution_digest()
    );
    assert_eq!(
        standard.remapped_subscription_slice_identity(),
        forensic.remapped_subscription_slice_identity()
    );
    assert_eq!(standard.remapped_slices(), forensic.remapped_slices());
}
