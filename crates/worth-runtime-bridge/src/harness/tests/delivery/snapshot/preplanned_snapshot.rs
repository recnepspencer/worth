use super::super::{
    build_runtime, commit_a, commit_b, committed_patch, patch_a, patch_b, registration, snapshot,
    snapshot_a, snapshot_b,
};
use crate::facade::BridgeRouteRequest;
use crate::harness::adapter::{BridgeHarnessAdapter, BridgeHarnessMutation, BridgeHarnessTargetId};
use crate::harness::fixtures::{
    BridgeHarnessFixture, InMemoryRelationalBridgeSource, RecordingSignalBridgeSink,
};
use worth_harness::facade::{ExecutionProfile, ExecutionRequest, MutationBatch, ScenarioPlan};
use worth_harness::runtime::HarnessAdapter;

#[test]
fn bridge_snapshot_delivery_remains_stable_after_newer_truth_arrives() {
    let adapter = BridgeHarnessAdapter;
    let fixture = ScenarioPlan::new(
        "bridge-snapshot-stability",
        BridgeHarnessFixture::new(vec![registration()])
            .with_committed_patch(committed_patch(
                commit_a(),
                patch_a(),
                snapshot_a(),
                worth_foundational::facade::FieldKey::new("name".to_owned())
                    .expect("valid harness field key"),
            ))
            .with_snapshot(snapshot(snapshot_a(), "alice")),
    )
    .declare_input("commit-a")
    .declare_observation("route")
    .compile();
    let mutation = MutationBatch::new("publish-newer-truth")
        .push(BridgeHarnessMutation::PublishCommittedPatch(
            committed_patch(
                commit_b(),
                patch_b(),
                snapshot_b(),
                worth_foundational::facade::FieldKey::new("name".to_owned())
                    .expect("valid harness field key"),
            ),
        ))
        .push(BridgeHarnessMutation::PublishSnapshot(snapshot(
            snapshot_b(),
            "bob",
        )));
    let request = ExecutionRequest::target(
        "deliver-commit-a",
        BridgeHarnessTargetId::committed_route(commit_a()),
    );
    let profile = ExecutionProfile::development("development");

    let mut session = adapter.create_runtime().expect("bridge harness runtime");
    adapter
        .prepare_runtime(&mut session, &profile)
        .expect("bridge harness prepare");
    adapter
        .load_fixture(&mut session, &fixture)
        .expect("bridge harness load fixture");
    adapter
        .apply_mutation_batch(&mut session, &mutation)
        .expect("bridge harness mutation should publish newer truth");
    adapter
        .execute(&mut session, &fixture, &request, &profile)
        .expect("bridge snapshot-stability execution should succeed");
    let route_record = session
        .runtime
        .as_ref()
        .expect("bridge runtime")
        .diagnostics()
        .last_route_record()
        .expect("bridge should retain typed route record");

    assert_eq!(
        route_record.source_snapshot().relational_snapshot_parts(),
        Some(crate::facade::RelationalBridgeSnapshotIdentityParts::new(
            1, 1
        ))
    );
}

#[test]
fn bridge_delivery_keeps_preplanned_snapshot_after_newer_truth_arrives_during_delivery() {
    let source = InMemoryRelationalBridgeSource::default();
    source.insert_committed_patch(committed_patch(
        commit_a(),
        patch_a(),
        snapshot_a(),
        worth_foundational::facade::FieldKey::new("name".to_owned())
            .expect("valid harness field key"),
    ));
    source.insert_snapshot(snapshot(snapshot_a(), "alice"));
    let sink = RecordingSignalBridgeSink::default();
    let runtime = build_runtime(source.clone(), sink.clone(), vec![registration()]);

    let route = runtime
        .plan_committed_patch(BridgeRouteRequest::for_commit(commit_a()))
        .expect("bridge should plan from the original committed artifact");

    source.insert_committed_patch(committed_patch(
        commit_b(),
        patch_b(),
        snapshot_b(),
        worth_foundational::facade::FieldKey::new("name".to_owned())
            .expect("valid harness field key"),
    ));
    source.insert_snapshot(snapshot(snapshot_b(), "bob"));

    let result = runtime
        .deliver_invalidation(route)
        .expect("bridge should deliver the preplanned route against its original snapshot");

    assert_eq!(
        result
            .result_summary()
            .snapshot_identity()
            .relational_snapshot_parts(),
        Some(crate::facade::RelationalBridgeSnapshotIdentityParts::new(
            1, 1
        ))
    );
    assert_eq!(
        result
            .receipt()
            .snapshot_identity()
            .relational_snapshot_parts(),
        Some(crate::facade::RelationalBridgeSnapshotIdentityParts::new(
            1, 1
        ))
    );
    let delivered = sink
        .last_delivery()
        .expect("bridge sink should record the delivered artifact");
    assert!(!delivered.leased);
    assert_eq!(
        delivered
            .delivery
            .source_snapshot()
            .relational_snapshot_parts(),
        Some(crate::facade::RelationalBridgeSnapshotIdentityParts::new(
            1, 1
        ))
    );
}

#[test]
fn leased_facade_delivery_carries_the_lease_to_the_signal_sink() {
    let source = InMemoryRelationalBridgeSource::default();
    source.insert_committed_patch(committed_patch(
        commit_a(),
        patch_a(),
        snapshot_a(),
        worth_foundational::facade::FieldKey::new("name".to_owned()).unwrap(),
    ));
    source.insert_snapshot(snapshot(snapshot_a(), "alice"));
    let sink = RecordingSignalBridgeSink::default();
    let runtime = build_runtime(source, sink.clone(), vec![registration()]);
    let route = runtime
        .plan_committed_patch(BridgeRouteRequest::for_commit(commit_a()))
        .unwrap();
    let lease = crate::snapshot::test_execution_lease(worth_execution::CancellationToken::new());

    runtime
        .deliver_invalidation_with_lease(route, &lease)
        .expect("leased route should reach the Signal sink");
    assert!(sink.last_delivery().unwrap().leased);
}

#[test]
fn bridge_prepares_signal_evaluation_with_snapshot_context_without_sink_delivery() {
    let source = InMemoryRelationalBridgeSource::default();
    source.insert_committed_patch(committed_patch(
        commit_a(),
        patch_a(),
        snapshot_a(),
        worth_foundational::facade::FieldKey::new("name".to_owned())
            .expect("valid harness field key"),
    ));
    source.insert_snapshot(snapshot(snapshot_a(), "alice"));
    let sink = RecordingSignalBridgeSink::default();
    let runtime = build_runtime(source, sink.clone(), vec![registration()]);

    let route = runtime
        .plan_committed_patch(BridgeRouteRequest::for_commit(commit_a()))
        .expect("bridge should plan the route");
    let evaluation = runtime
        .prepare_signal_evaluation(route)
        .expect("bridge should prepare signal evaluation");

    assert_eq!(
        evaluation
            .snapshot()
            .snapshot_identity()
            .relational_snapshot_parts(),
        Some(crate::facade::RelationalBridgeSnapshotIdentityParts::new(
            1, 1
        ))
    );
    assert!(sink.last_delivery().is_none());
}

#[test]
fn bridge_prepared_signal_evaluation_keeps_preplanned_snapshot_after_newer_truth_arrives() {
    let source = InMemoryRelationalBridgeSource::default();
    source.insert_committed_patch(committed_patch(
        commit_a(),
        patch_a(),
        snapshot_a(),
        worth_foundational::facade::FieldKey::new("name".to_owned())
            .expect("valid harness field key"),
    ));
    source.insert_snapshot(snapshot(snapshot_a(), "alice"));
    let sink = RecordingSignalBridgeSink::default();
    let runtime = build_runtime(source.clone(), sink, vec![registration()]);

    let route = runtime
        .plan_committed_patch(BridgeRouteRequest::for_commit(commit_a()))
        .expect("bridge should plan the route");

    source.insert_committed_patch(committed_patch(
        commit_b(),
        patch_b(),
        snapshot_b(),
        worth_foundational::facade::FieldKey::new("name".to_owned())
            .expect("valid harness field key"),
    ));
    source.insert_snapshot(snapshot(snapshot_b(), "bob"));

    let evaluation = runtime
        .prepare_signal_evaluation(route)
        .expect("bridge should prepare signal evaluation");

    assert_eq!(
        evaluation
            .snapshot()
            .snapshot_identity()
            .relational_snapshot_parts(),
        Some(crate::facade::RelationalBridgeSnapshotIdentityParts::new(
            1, 1
        ))
    );
}
