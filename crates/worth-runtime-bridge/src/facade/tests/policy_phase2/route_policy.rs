use crate::facade::tests::policy_phase2::admitted_bundle;
use crate::facade::tests::runtime;
use crate::facade::{
    BridgeBulkWorkloadRequest, BridgeBulkWorkloadSegment, BridgeDiagnosticsTier,
    BridgeExecutionPolicyClass, BridgePolicyDeclaration, BridgePolicyDeclarationIdentity,
    BridgeRequestKind, BridgeRouteErrorKind, BridgeRouteRequest, BridgeRuntimePolicy,
};

#[test]
fn runtime_projects_route_planning_policy_and_stamps_planned_route() {
    let serial_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let runtime = runtime(BridgeRuntimePolicy::default());
    let (contract, lowered, provenance, replay_bundle) = admitted_bundle(
        &runtime,
        BridgePolicyDeclaration::new(
            BridgePolicyDeclarationIdentity::admit_bridge_owned("policy:route-planning"),
            BridgeRequestKind::Preview,
            BridgeExecutionPolicyClass::Optimized,
            BridgeDiagnosticsTier::Standard,
            false,
            false,
        ),
    );
    let route_policy = runtime
        .project_route_planning_policy(&lowered)
        .expect("route policy projection should succeed");
    let planned = runtime
        .plan_committed_patch_with_route_policy(
            BridgeRouteRequest::for_commit(crate::truth_identity_fixtures::truth_commit_fixture(
                "commit-a",
            )),
            &route_policy,
            execution,
        )
        .expect("route planning under lowered policy should succeed");

    assert_eq!(
        planned.route_planning_policy_digest(),
        Some(route_policy.digest())
    );
    let row = runtime.summarize_policy_provenance_row(
        "route-planning",
        &contract,
        &lowered,
        &provenance,
        &replay_bundle,
    );
    assert_eq!(row.lowered_policy_digest(), lowered.digest());
}

#[test]
fn runtime_rejects_divergent_route_planning_policy_from_more_permissive_runtime() {
    let permissive = runtime(BridgeRuntimePolicy::development());
    let restrictive = runtime(BridgeRuntimePolicy::operational().with_replay_artifacts(false));
    let (_, lowered, _, _) = admitted_bundle(
        &permissive,
        BridgePolicyDeclaration::new(
            BridgePolicyDeclarationIdentity::admit_bridge_owned("policy:replay-required-for-route"),
            BridgeRequestKind::Authoritative,
            BridgeExecutionPolicyClass::DeterministicCanonical,
            BridgeDiagnosticsTier::Standard,
            true,
            true,
        ),
    );

    let error = restrictive
        .project_route_planning_policy(&lowered)
        .expect_err("restrictive runtime should reject divergent route policy");

    assert_eq!(error.kind(), BridgeRouteErrorKind::RoutePolicyMismatch);
}

#[test]
fn bulk_route_planning_policy_is_carried_by_every_planned_route() {
    let serial_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let runtime = runtime(BridgeRuntimePolicy::default());
    let (_, lowered, _, _) = admitted_bundle(
        &runtime,
        BridgePolicyDeclaration::new(
            BridgePolicyDeclarationIdentity::admit_bridge_owned("policy:bulk-route-planning"),
            BridgeRequestKind::Preview,
            BridgeExecutionPolicyClass::Optimized,
            BridgeDiagnosticsTier::Minimal,
            false,
            false,
        ),
    );
    let route_policy = runtime
        .project_route_planning_policy(&lowered)
        .expect("bulk route policy projection should succeed");
    let workload = BridgeBulkWorkloadRequest::new(vec![
        BridgeBulkWorkloadSegment::new(BridgeRouteRequest::for_commit(
            crate::truth_identity_fixtures::truth_commit_fixture("commit-a"),
        )),
        BridgeBulkWorkloadSegment::new(BridgeRouteRequest::for_commit(
            crate::truth_identity_fixtures::truth_commit_fixture("commit-a"),
        )),
    ]);

    let plan = runtime
        .plan_bulk_workload_with_route_policy(workload, &route_policy, execution)
        .expect("bulk planning under route policy should succeed");

    assert_eq!(plan.planned_routes().len(), 2);
    for route in plan.planned_routes() {
        assert_eq!(
            route.route_planning_policy_digest(),
            Some(route_policy.digest())
        );
    }
}

#[test]
fn policy_scoped_route_round_trips_through_canonical_replay() {
    let serial_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let runtime = runtime(BridgeRuntimePolicy::default());
    let (_, lowered, _, _) = admitted_bundle(
        &runtime,
        BridgePolicyDeclaration::new(
            BridgePolicyDeclarationIdentity::admit_bridge_owned("policy:route-replay-scope"),
            BridgeRequestKind::Preview,
            BridgeExecutionPolicyClass::Optimized,
            BridgeDiagnosticsTier::Standard,
            false,
            true,
        ),
    );
    let route_policy = runtime
        .project_route_planning_policy(&lowered)
        .expect("route replay policy projection should succeed");
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
    let canonical_record = runtime
        .diagnostics()
        .last_canonical_route_record()
        .expect("canonical route record should be retained");

    let replay = runtime
        .replay_canonical_record(&canonical_record, execution)
        .expect("policy scoped canonical route should replay");

    assert_eq!(
        result.result_summary().route_planning_policy_digest(),
        Some(route_policy.digest())
    );
    assert_eq!(
        canonical_record
            .decode()
            .expect("canonical route record should decode")
            .route_planning_policy_digest(),
        Some(route_policy.digest())
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
fn policy_scoped_route_without_route_artifacts_does_not_retain_canonical_record() {
    let serial_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let runtime = runtime(BridgeRuntimePolicy::default());
    let (_, lowered, _, _) = admitted_bundle(
        &runtime,
        BridgePolicyDeclaration::new(
            BridgePolicyDeclarationIdentity::admit_bridge_owned("policy:route-no-retention"),
            BridgeRequestKind::Preview,
            BridgeExecutionPolicyClass::Optimized,
            BridgeDiagnosticsTier::Standard,
            false,
            false,
        ),
    );
    let route_policy = runtime
        .project_route_planning_policy(&lowered)
        .expect("route policy projection should succeed");

    runtime
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

    assert!(runtime
        .diagnostics()
        .last_canonical_route_record()
        .is_none());
}
