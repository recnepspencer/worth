use crate::facade::BridgeRouteRequest;
use crate::facade::{
    BridgeDiagnosticsTier, BridgeExecutionPolicyClass, BridgePolicyDeclaration,
    BridgePolicyDeclarationIdentity, BridgeRequestKind,
};

use super::super::support::{
    build_runtime_with_aspects, committed_patch, field_aspect_registration, field_slice_snapshot,
    registration,
};
use crate::harness::fixtures::{InMemoryRelationalBridgeSource, RecordingSignalBridgeSink};

#[test]
fn replayed_slice_route_matches_original_canonical_slice_artifact() {
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
    let runtime = build_runtime_with_aspects(
        source,
        RecordingSignalBridgeSink::default(),
        vec![registration()],
        vec![field_aspect_registration()],
    );

    let result = runtime
        .deliver_invalidation(
            runtime
                .plan_committed_patch(
                    BridgeRouteRequest::for_commit(
                        crate::truth_identity_fixtures::truth_commit_fixture("commit-a"),
                    ),
                    execution,
                )
                .expect("slice route should plan"),
            execution,
        )
        .expect("slice route should deliver");
    let canonical = runtime
        .diagnostics()
        .last_canonical_route_record()
        .expect("canonical route record should be retained");
    let replay = runtime
        .replay_canonical_record(&canonical, execution)
        .expect("canonical slice route should replay");

    assert_eq!(
        replay.subscription_slice_identity(),
        result.result_summary().subscription_slice_identity()
    );
    assert_eq!(
        replay.route_identity(),
        result.result_summary().route_identity()
    );
    assert_eq!(
        replay.invalidation_identity(),
        result.result_summary().invalidation_identity()
    );
}

#[test]
fn replayed_policy_scoped_route_preserves_route_policy_digest_in_route_record() {
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
    let runtime = build_runtime_with_aspects(
        source,
        RecordingSignalBridgeSink::default(),
        vec![registration()],
        vec![field_aspect_registration()],
    );
    let declaration = BridgePolicyDeclaration::new(
        BridgePolicyDeclarationIdentity::admit_bridge_owned("policy:route-record-visible"),
        BridgeRequestKind::Preview,
        BridgeExecutionPolicyClass::Optimized,
        BridgeDiagnosticsTier::Standard,
        false,
        true,
    );
    let contract = runtime
        .admit_policy_declaration(declaration)
        .expect("policy declaration should admit");
    let lowered = runtime.lower_admitted_policy(&contract);
    let route_policy = runtime
        .project_route_planning_policy(&lowered)
        .expect("route planning policy should project");

    let result = runtime
        .deliver_invalidation(
            runtime
                .plan_committed_patch_with_route_policy(
                    BridgeRouteRequest::for_commit(
                        crate::truth_identity_fixtures::truth_commit_fixture("commit-a"),
                    ),
                    &route_policy,
                    execution,
                )
                .expect("policy scoped route should plan"),
            execution,
        )
        .expect("policy scoped route should deliver");
    let canonical = runtime
        .diagnostics()
        .last_canonical_route_record()
        .expect("canonical route record should be retained");
    let replay = runtime
        .replay_canonical_record(&canonical, execution)
        .expect("policy scoped route should replay");
    let record = runtime
        .diagnostics()
        .last_route_record()
        .expect("route record should be retained");

    assert_eq!(
        result.result_summary().route_planning_policy_digest(),
        Some(route_policy.digest())
    );
    assert_eq!(
        record.route_planning_policy_digest(),
        Some(route_policy.digest())
    );
    assert_eq!(
        canonical
            .decode()
            .expect("canonical route record should decode")
            .route_planning_policy_digest(),
        Some(route_policy.digest())
    );
    assert_eq!(
        replay.route_identity(),
        result.result_summary().route_identity()
    );
}
