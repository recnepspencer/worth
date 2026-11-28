use crate::facade::BridgeRouteRequest;
use crate::facade::{
    BridgeDiagnosticsTier, BridgeExecutionPolicyClass, BridgePolicyDeclaration,
    BridgePolicyDeclarationIdentity, BridgeRequestKind,
};

use super::super::support::build_runtime_with_aspects;
use crate::harness::fixtures::{NamedTruth, RecordingSignalBridgeSink};

#[test]
fn replayed_slice_route_matches_original_canonical_slice_artifact() {
    let serial_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let truth = NamedTruth::new();
    let (commit, _observation) = truth.commit_name(execution, "alice");
    let runtime = build_runtime_with_aspects(
        truth.source,
        RecordingSignalBridgeSink::default(),
        vec![NamedTruth::mapping()],
        vec![NamedTruth::aspect_mapping()],
    );

    let result = runtime
        .deliver_invalidation(
            runtime
                .plan_committed_patch(BridgeRouteRequest::for_commit(commit), execution)
                .expect("slice route should plan"),
            execution,
        )
        .expect("slice route should deliver");
    assert_eq!(result.result_summary().subscription_slice_count(), 1);
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

    let truth = NamedTruth::new();
    let (commit, _observation) = truth.commit_name(execution, "alice");
    let runtime = build_runtime_with_aspects(
        truth.source,
        RecordingSignalBridgeSink::default(),
        vec![NamedTruth::mapping()],
        vec![NamedTruth::aspect_mapping()],
    );
    let declaration = BridgePolicyDeclaration::new(
        BridgePolicyDeclarationIdentity::admit_bridge_owned("policy:route-record-visible"),
        BridgeRequestKind::Preview,
        BridgeExecutionPolicyClass::Optimized,
        BridgeDiagnosticsTier::Standard,
        true,
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
                    BridgeRouteRequest::for_commit(commit),
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
